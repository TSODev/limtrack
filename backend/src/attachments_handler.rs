// src/attachments_handler.rs

use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;

use common::MaintenanceAttachment;

// ─── Limites métier ──────────────────────────────────────────────

const MAX_ATTACHMENTS_PER_ENTRY: i64 = 5;
const MAX_FILE_SIZE: usize = 8 * 1024 * 1024; // 8 Mo
const ALLOWED_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp", "application/pdf"];

// ─── Erreur unifiée ──────────────────────────────────────────────

#[derive(Serialize)]
struct ApiError {
    error: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

async fn require_editor(
    db: &sqlx::PgPool,
    vehicle_id: Uuid,
    user_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let role = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données"))?;

    match role.as_deref() {
        Some("owner") | Some("editor") => Ok(()),
        Some(_) => Err(err(StatusCode::FORBIDDEN, "Droits insuffisants (owner ou editor requis)")),
        None => Err(err(StatusCode::NOT_FOUND, "Véhicule introuvable ou accès refusé")),
    }
}

async fn check_read_access(
    db: &sqlx::PgPool,
    vehicle_id: Uuid,
    user_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let access = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(db)
    .await;

    if matches!(access, Ok(None) | Err(_)) {
        return Err(err(StatusCode::NOT_FOUND, "Véhicule introuvable ou accès refusé"));
    }
    Ok(())
}

fn sanitize_extension(original_filename: &str) -> String {
    std::path::Path::new(original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.chars().filter(|c| c.is_ascii_alphanumeric()).take(8).collect::<String>())
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| "bin".to_string())
}

fn sanitize_filename_for_header(name: &str) -> String {
    name.chars().filter(|c| c.is_ascii_graphic() || *c == ' ').take(150).collect()
}

// ─── POST /vehicles/:vehicle_id/maintenance-entries/:entry_id/attachments

pub async fn upload_attachments(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, entry_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    let entry_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM public.maintenance_entries WHERE id = $1 AND vehicle_id = $2)",
        entry_id,
        vehicle_id,
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(false))
    .unwrap_or(false);

    if !entry_exists {
        return err(StatusCode::NOT_FOUND, "Entretien introuvable").into_response();
    }

    let mut count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.maintenance_attachments WHERE entry_id = $1",
        entry_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    let mut created: Vec<MaintenanceAttachment> = Vec::new();

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => return err(StatusCode::UNPROCESSABLE_ENTITY, format!("Erreur de lecture du formulaire : {}", e)).into_response(),
        };

        if count >= MAX_ATTACHMENTS_PER_ENTRY {
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Limite de {} pièces jointes par entretien atteinte", MAX_ATTACHMENTS_PER_ENTRY),
            )
            .into_response();
        }

        let original_filename = field.file_name().unwrap_or("fichier").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();

        if !ALLOWED_CONTENT_TYPES.contains(&content_type.as_str()) {
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Type de fichier non autorisé : {} (jpeg, png, webp, pdf uniquement)", content_type),
            )
            .into_response();
        }

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(e) => return err(StatusCode::UNPROCESSABLE_ENTITY, format!("Erreur de lecture du fichier : {}", e)).into_response(),
        };

        if data.len() > MAX_FILE_SIZE {
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Fichier trop volumineux (max {} Mo)", MAX_FILE_SIZE / (1024 * 1024)),
            )
            .into_response();
        }

        let ext = sanitize_extension(&original_filename);
        let stored_name = format!("{}.{}", Uuid::new_v4(), ext);
        let rel_dir = format!("uploads/{}/{}", vehicle_id, entry_id);
        let rel_path = format!("{}/{}", rel_dir, stored_name);

        if let Err(e) = tokio::fs::create_dir_all(&rel_dir).await {
            tracing::error!("Erreur création répertoire uploads {} : {}", rel_dir, e);
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur de stockage du fichier").into_response();
        }
        if let Err(e) = tokio::fs::write(&rel_path, &data).await {
            tracing::error!("Erreur écriture fichier {} : {}", rel_path, e);
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur de stockage du fichier").into_response();
        }

        let size_bytes = data.len() as i32;
        let insert = sqlx::query_as!(
            MaintenanceAttachment,
            r#"
            INSERT INTO public.maintenance_attachments
                (entry_id, vehicle_id, file_path, original_filename, content_type, size_bytes)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, entry_id, vehicle_id, original_filename, content_type, size_bytes, created_at
            "#,
            entry_id,
            vehicle_id,
            rel_path,
            original_filename,
            content_type,
            size_bytes,
        )
        .fetch_one(&state.db)
        .await;

        match insert {
            Ok(att) => {
                created.push(att);
                count += 1;
            }
            Err(e) => {
                let _ = tokio::fs::remove_file(&rel_path).await;
                return err(StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur d'enregistrement de la pièce jointe : {}", e)).into_response();
            }
        }
    }

    (StatusCode::CREATED, Json(created)).into_response()
}

// ─── GET /vehicles/:vehicle_id/maintenance-entries/:entry_id/attachments

pub async fn list_attachments(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, entry_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = check_read_access(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    let rows = sqlx::query_as!(
        MaintenanceAttachment,
        r#"
        SELECT id, entry_id, vehicle_id, original_filename, content_type, size_bytes, created_at
        FROM public.maintenance_attachments
        WHERE entry_id = $1
        ORDER BY created_at ASC
        "#,
        entry_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(list) => (StatusCode::OK, Json(list)).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── GET /vehicles/:vehicle_id/attachments/:attachment_id

pub async fn download_attachment(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, attachment_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = check_read_access(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    let row = sqlx::query!(
        "SELECT file_path, content_type, original_filename FROM public.maintenance_attachments WHERE id = $1 AND vehicle_id = $2",
        attachment_id,
        vehicle_id,
    )
    .fetch_optional(&state.db)
    .await;

    let row = match row {
        Ok(Some(r)) => r,
        Ok(None) => return err(StatusCode::NOT_FOUND, "Pièce jointe introuvable").into_response(),
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

    match tokio::fs::read(&row.file_path).await {
        Ok(bytes) => {
            let disposition = format!("inline; filename=\"{}\"", sanitize_filename_for_header(&row.original_filename));
            let headers = [
                (header::CONTENT_TYPE, row.content_type),
                (header::CONTENT_DISPOSITION, disposition),
            ];
            (StatusCode::OK, headers, bytes).into_response()
        }
        Err(e) => {
            tracing::error!("Erreur lecture fichier {} : {}", row.file_path, e);
            err(StatusCode::INTERNAL_SERVER_ERROR, "Fichier introuvable sur le serveur").into_response()
        }
    }
}

// ─── DELETE /vehicles/:vehicle_id/attachments/:attachment_id

pub async fn delete_attachment(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, attachment_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    let row = sqlx::query!(
        "SELECT file_path FROM public.maintenance_attachments WHERE id = $1 AND vehicle_id = $2",
        attachment_id,
        vehicle_id,
    )
    .fetch_optional(&state.db)
    .await;

    let file_path = match row {
        Ok(Some(r)) => r.file_path,
        Ok(None) => return err(StatusCode::NOT_FOUND, "Pièce jointe introuvable").into_response(),
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

    match sqlx::query!("DELETE FROM public.maintenance_attachments WHERE id = $1", attachment_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => {
            if let Err(e) = tokio::fs::remove_file(&file_path).await {
                tracing::error!("Erreur suppression fichier {} : {}", file_path, e);
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}
