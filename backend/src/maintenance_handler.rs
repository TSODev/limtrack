// src/maintenance_handler.rs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Local, NaiveDate};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;

use common::{
    CreateMaintenanceEntryPayload, CreateMaintenanceTypePayload, MaintenanceEntry,
    MaintenanceStatus, MaintenanceType, UpdateMaintenanceTypePayload,
};

// ─── Limites métier ──────────────────────────────────────────────

const MAX_MAINTENANCE_TYPES_PER_VEHICLE: i64 = 20;
const MAX_TYPES_PER_ENTRY: usize = 10;
const MAX_LEN_LABEL: usize = 100;
const MAX_LEN_PROVIDER: usize = 200;
const MAX_LEN_NOTES: usize = 2000;

// ─── Erreur unifiée ──────────────────────────────────────────────

#[derive(Serialize)]
struct ApiError {
    error: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

// ─── Helper : vérifie owner ou editor ────────────────────────────

async fn require_editor(
    db: &sqlx::PgPool,
    vehicle_id: Uuid,
    user_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let role = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access
         WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données"))?;

    match role.as_deref() {
        Some("owner") | Some("editor") => Ok(()),
        Some(_) => Err(err(
            StatusCode::FORBIDDEN,
            "Droits insuffisants (owner ou editor requis)",
        )),
        None => Err(err(
            StatusCode::NOT_FOUND,
            "Véhicule introuvable ou accès refusé",
        )),
    }
}

async fn check_read_access(
    db: &sqlx::PgPool,
    vehicle_id: Uuid,
    user_id: Uuid,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    let access = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access
         WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(db)
    .await;

    if matches!(access, Ok(None) | Err(_)) {
        return Err(err(
            StatusCode::NOT_FOUND,
            "Véhicule introuvable ou accès refusé",
        ));
    }
    Ok(())
}

// ─── Validation des types ─────────────────────────────────────────

fn validate_type_fields(
    label: &str,
    interval_km: Option<i32>,
    interval_months: Option<i32>,
) -> Result<(), (StatusCode, Json<ApiError>)> {
    if label.trim().is_empty() {
        return Err(err(StatusCode::UNPROCESSABLE_ENTITY, "label ne peut pas être vide"));
    }
    if label.len() > MAX_LEN_LABEL {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("label : {MAX_LEN_LABEL} caractères max"),
        ));
    }
    if interval_km.map(|v| v <= 0).unwrap_or(false) {
        return Err(err(StatusCode::UNPROCESSABLE_ENTITY, "interval_km doit être positif"));
    }
    if interval_months.map(|v| v <= 0).unwrap_or(false) {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "interval_months doit être positif",
        ));
    }
    if interval_km.is_none() && interval_months.is_none() {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "au moins un intervalle (km ou mois) est requis",
        ));
    }
    Ok(())
}

// ─── POST /vehicles/:vehicle_id/maintenance-types ────────────────

pub async fn create_maintenance_type(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateMaintenanceTypePayload>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    let type_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.maintenance_types WHERE vehicle_id = $1",
        vehicle_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    if type_count >= MAX_MAINTENANCE_TYPES_PER_VEHICLE {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Limite de {} types d'entretien par véhicule atteinte", MAX_MAINTENANCE_TYPES_PER_VEHICLE),
        )
        .into_response();
    }

    if let Err(e) = validate_type_fields(&payload.label, payload.interval_km, payload.interval_months) {
        return e.into_response();
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO public.maintenance_types (vehicle_id, label, interval_km, interval_months)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        vehicle_id,
        payload.label.trim(),
        payload.interval_km,
        payload.interval_months,
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(row) => (StatusCode::CREATED, Json(serde_json::json!({ "id": row.id }))).into_response(),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur création type d'entretien : {}", e),
        )
        .into_response(),
    }
}

// ─── GET /vehicles/:vehicle_id/maintenance-types ─────────────────

pub async fn list_maintenance_types(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = check_read_access(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    let rows = sqlx::query_as!(
        MaintenanceType,
        r#"
        SELECT id, vehicle_id, label, interval_km, interval_months, active, created_at
        FROM public.maintenance_types
        WHERE vehicle_id = $1
        ORDER BY created_at ASC
        "#,
        vehicle_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(types) => (StatusCode::OK, Json(types)).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── PATCH /vehicles/:vehicle_id/maintenance-types/:type_id ─────

pub async fn update_maintenance_type(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, type_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateMaintenanceTypePayload>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    let current = sqlx::query!(
        r#"SELECT label, interval_km, interval_months, active
           FROM public.maintenance_types WHERE id = $1 AND vehicle_id = $2"#,
        type_id,
        vehicle_id,
    )
    .fetch_optional(&state.db)
    .await;

    let current = match current {
        Ok(Some(c)) => c,
        Ok(None) => return err(StatusCode::NOT_FOUND, "Type d'entretien introuvable").into_response(),
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

    let label = payload.label.clone().unwrap_or(current.label);
    let interval_km = payload.interval_km.or(current.interval_km);
    let interval_months = payload.interval_months.or(current.interval_months);
    let active = payload.active.unwrap_or(current.active);

    if let Err(e) = validate_type_fields(&label, interval_km, interval_months) {
        return e.into_response();
    }

    match sqlx::query!(
        r#"
        UPDATE public.maintenance_types SET
            label = $1, interval_km = $2, interval_months = $3, active = $4
        WHERE id = $5 AND vehicle_id = $6
        "#,
        label.trim(),
        interval_km,
        interval_months,
        active,
        type_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => err(StatusCode::NOT_FOUND, "Type d'entretien introuvable").into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── DELETE /vehicles/:vehicle_id/maintenance-types/:type_id ────

pub async fn delete_maintenance_type(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, type_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }
    match sqlx::query!(
        "DELETE FROM public.maintenance_types WHERE id = $1 AND vehicle_id = $2",
        type_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => err(StatusCode::NOT_FOUND, "Type d'entretien introuvable").into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── Résolution du label d'une entrée (création et modification) ─
// Fourni librement (ex. "Révision 30 000 km"), ou généré depuis les types sélectionnés
// (snapshot) s'il est absent. Requis si aucun type n'est sélectionné (entrée "Autre" libre).

async fn resolve_entry_label(
    db: &sqlx::PgPool,
    vehicle_id: Uuid,
    type_ids: &[Uuid],
    label_override: Option<&str>,
) -> Result<String, (StatusCode, Json<ApiError>)> {
    if !type_ids.is_empty() {
        let rows = sqlx::query!(
            "SELECT id, label FROM public.maintenance_types WHERE vehicle_id = $1 AND id = ANY($2)",
            vehicle_id,
            type_ids,
        )
        .fetch_all(db)
        .await
        .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données"))?;

        if rows.len() != type_ids.len() {
            return Err(err(
                StatusCode::UNPROCESSABLE_ENTITY,
                "maintenance_type_ids invalide ou n'appartient pas à ce véhicule",
            ));
        }

        match label_override.map(str::trim) {
            Some(l) if !l.is_empty() => {
                if l.len() > MAX_LEN_LABEL {
                    return Err(err(StatusCode::UNPROCESSABLE_ENTITY, format!("label : {MAX_LEN_LABEL} caractères max")));
                }
                Ok(l.to_string())
            }
            _ => {
                let mut labels_by_id: std::collections::HashMap<Uuid, String> =
                    rows.into_iter().map(|r| (r.id, r.label)).collect();
                Ok(type_ids
                    .iter()
                    .filter_map(|id| labels_by_id.remove(id))
                    .collect::<Vec<_>>()
                    .join(", "))
            }
        }
    } else {
        match label_override.map(str::trim) {
            Some(l) if !l.is_empty() => {
                if l.len() > MAX_LEN_LABEL {
                    return Err(err(StatusCode::UNPROCESSABLE_ENTITY, format!("label : {MAX_LEN_LABEL} caractères max")));
                }
                Ok(l.to_string())
            }
            _ => Err(err(
                StatusCode::UNPROCESSABLE_ENTITY,
                "label requis quand maintenance_type_ids est vide",
            )),
        }
    }
}

// ─── POST /vehicles/:vehicle_id/maintenance-entries ──────────────

pub async fn create_maintenance_entry(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateMaintenanceEntryPayload>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    if payload.km_at_service < 0 {
        return err(StatusCode::UNPROCESSABLE_ENTITY, "km_at_service ne peut pas être négatif").into_response();
    }
    if payload.cost.map(|c| c < 0.0).unwrap_or(false) {
        return err(StatusCode::UNPROCESSABLE_ENTITY, "cost ne peut pas être négatif").into_response();
    }
    if payload.provider.as_deref().map(|s| s.len()).unwrap_or(0) > MAX_LEN_PROVIDER {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("provider : {MAX_LEN_PROVIDER} caractères max")).into_response();
    }
    if payload.notes.as_deref().map(|s| s.len()).unwrap_or(0) > MAX_LEN_NOTES {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("notes : {MAX_LEN_NOTES} caractères max")).into_response();
    }

    // Dédoublonne en conservant l'ordre de sélection (sert à générer le label auto ci-dessous)
    let mut type_ids: Vec<Uuid> = Vec::with_capacity(payload.maintenance_type_ids.len());
    for id in &payload.maintenance_type_ids {
        if !type_ids.contains(id) {
            type_ids.push(*id);
        }
    }
    if type_ids.len() > MAX_TYPES_PER_ENTRY {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Maximum {} types par entretien", MAX_TYPES_PER_ENTRY),
        )
        .into_response();
    }

    // Label : fourni librement (ex. "Révision 30 000 km"), ou généré depuis les types
    // sélectionnés (snapshot) s'il est absent. Requis si aucun type n'est sélectionné
    // (entrée "Autre" libre).
    let label = match resolve_entry_label(&state.db, vehicle_id, &type_ids, payload.label.as_deref()).await {
        Ok(l) => l,
        Err(e) => return e.into_response(),
    };

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

    let entry_id = match sqlx::query_scalar!(
        r#"
        INSERT INTO public.maintenance_entries
            (vehicle_id, label, performed_at, km_at_service, cost, provider, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        vehicle_id,
        label,
        payload.performed_at,
        payload.km_at_service,
        payload.cost,
        payload.provider.as_deref().map(str::trim),
        payload.notes.as_deref().map(str::trim),
    )
    .fetch_one(&mut *tx)
    .await
    {
        Ok(id) => id,
        Err(e) => {
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur création entretien : {}", e),
            )
            .into_response()
        }
    };

    for type_id in &type_ids {
        if let Err(e) = sqlx::query!(
            "INSERT INTO public.maintenance_entry_types (entry_id, maintenance_type_id) VALUES ($1, $2)",
            entry_id,
            type_id,
        )
        .execute(&mut *tx)
        .await
        {
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur rattachement type d'entretien : {}", e),
            )
            .into_response();
        }
    }

    if tx.commit().await.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response();
    }

    (StatusCode::CREATED, Json(serde_json::json!({ "id": entry_id }))).into_response()
}

// ─── GET /vehicles/:vehicle_id/maintenance-entries ───────────────

pub async fn list_maintenance_entries(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = check_read_access(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    let rows = sqlx::query_as!(
        MaintenanceEntry,
        r#"
        SELECT
            e.id, e.vehicle_id, e.label, e.performed_at,
            e.km_at_service, e.cost, e.provider, e.notes, e.created_at,
            COALESCE(
                ARRAY_AGG(met.maintenance_type_id) FILTER (WHERE met.maintenance_type_id IS NOT NULL),
                '{}'
            ) AS "type_ids!: Vec<Uuid>",
            (SELECT COUNT(*) FROM public.maintenance_attachments a WHERE a.entry_id = e.id) AS "attachment_count!"
        FROM public.maintenance_entries e
        LEFT JOIN public.maintenance_entry_types met ON met.entry_id = e.id
        WHERE e.vehicle_id = $1
        GROUP BY e.id
        ORDER BY e.performed_at DESC, e.created_at DESC
        "#,
        vehicle_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(entries) => (StatusCode::OK, Json(entries)).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── DELETE /vehicles/:vehicle_id/maintenance-entries/:entry_id ─

pub async fn delete_maintenance_entry(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, entry_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    // Récupère les fichiers attachés avant suppression (la CASCADE SQL nettoie la table
    // mais pas le filesystem — cleanup best-effort après le DELETE).
    let attachment_paths = sqlx::query_scalar!(
        "SELECT file_path FROM public.maintenance_attachments WHERE entry_id = $1",
        entry_id
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    match sqlx::query!(
        "DELETE FROM public.maintenance_entries WHERE id = $1 AND vehicle_id = $2",
        entry_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => err(StatusCode::NOT_FOUND, "Entretien introuvable").into_response(),
        Ok(_) => {
            for path in attachment_paths {
                if let Err(e) = tokio::fs::remove_file(&path).await {
                    tracing::error!("Erreur suppression fichier {} : {}", path, e);
                }
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── PATCH /vehicles/:vehicle_id/maintenance-entries/:entry_id ───
// Permet de corriger une entrée après coup (ex. saisie rapide "Vidange" au moment du
// rendez-vous, puis complétée plus tard avec le prix et la date exacts une fois la
// facture en main) — même validation que la création, types remplacés intégralement
// (delete + re-insert du pivot, plus simple qu'un diff pour au plus MAX_TYPES_PER_ENTRY lignes).

#[derive(serde::Deserialize)]
pub struct UpdateMaintenanceEntryPayload {
    #[serde(default)]
    pub maintenance_type_ids: Vec<Uuid>,
    pub label: Option<String>,
    pub performed_at: chrono::NaiveDate,
    pub km_at_service: i32,
    pub cost: Option<f64>,
    pub provider: Option<String>,
    pub notes: Option<String>,
}

pub async fn update_maintenance_entry(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, entry_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateMaintenanceEntryPayload>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    if payload.km_at_service < 0 {
        return err(StatusCode::UNPROCESSABLE_ENTITY, "km_at_service ne peut pas être négatif").into_response();
    }
    if payload.cost.map(|c| c < 0.0).unwrap_or(false) {
        return err(StatusCode::UNPROCESSABLE_ENTITY, "cost ne peut pas être négatif").into_response();
    }
    if payload.provider.as_deref().map(|s| s.len()).unwrap_or(0) > MAX_LEN_PROVIDER {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("provider : {MAX_LEN_PROVIDER} caractères max")).into_response();
    }
    if payload.notes.as_deref().map(|s| s.len()).unwrap_or(0) > MAX_LEN_NOTES {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("notes : {MAX_LEN_NOTES} caractères max")).into_response();
    }

    let mut type_ids: Vec<Uuid> = Vec::with_capacity(payload.maintenance_type_ids.len());
    for id in &payload.maintenance_type_ids {
        if !type_ids.contains(id) {
            type_ids.push(*id);
        }
    }
    if type_ids.len() > MAX_TYPES_PER_ENTRY {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Maximum {} types par entretien", MAX_TYPES_PER_ENTRY),
        )
        .into_response();
    }

    let label = match resolve_entry_label(&state.db, vehicle_id, &type_ids, payload.label.as_deref()).await {
        Ok(l) => l,
        Err(e) => return e.into_response(),
    };

    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

    let updated = sqlx::query!(
        r#"
        UPDATE public.maintenance_entries
        SET label = $1, performed_at = $2, km_at_service = $3, cost = $4, provider = $5, notes = $6
        WHERE id = $7 AND vehicle_id = $8
        "#,
        label,
        payload.performed_at,
        payload.km_at_service,
        payload.cost,
        payload.provider.as_deref().map(str::trim),
        payload.notes.as_deref().map(str::trim),
        entry_id,
        vehicle_id,
    )
    .execute(&mut *tx)
    .await;

    match updated {
        Ok(r) if r.rows_affected() == 0 => {
            return err(StatusCode::NOT_FOUND, "Entretien introuvable").into_response();
        }
        Ok(_) => {}
        Err(e) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur mise à jour entretien : {}", e)).into_response();
        }
    }

    if let Err(e) = sqlx::query!(
        "DELETE FROM public.maintenance_entry_types WHERE entry_id = $1",
        entry_id,
    )
    .execute(&mut *tx)
    .await
    {
        return err(StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur mise à jour des types : {}", e)).into_response();
    }

    for type_id in &type_ids {
        if let Err(e) = sqlx::query!(
            "INSERT INTO public.maintenance_entry_types (entry_id, maintenance_type_id) VALUES ($1, $2)",
            entry_id,
            type_id,
        )
        .execute(&mut *tx)
        .await
        {
            return err(StatusCode::INTERNAL_SERVER_ERROR, format!("Erreur rattachement type d'entretien : {}", e)).into_response();
        }
    }

    if tx.commit().await.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}

// ─── Projection : date estimée d'atteinte d'un kilométrage cible ─
// Basé sur le rythme moyen (premier → dernier relevé km) de tout l'historique
// du véhicule — contrairement à estimate_limit_date (contracts_handler.rs) qui
// est borné à un contrat, ici on utilise l'ensemble du mileage_log.

fn estimate_date_for_km(
    first_date: NaiveDate,
    first_km: i32,
    last_date: NaiveDate,
    last_km: i32,
    target_km: i32,
) -> Option<NaiveDate> {
    if target_km <= last_km {
        return Some(last_date);
    }
    let days_elapsed = (last_date - first_date).num_days();
    if days_elapsed <= 0 {
        return None;
    }
    let km_per_day = (last_km - first_km) as f64 / days_elapsed as f64;
    if km_per_day <= 0.0 {
        return None;
    }
    let days_to_target = ((target_km - last_km) as f64 / km_per_day).ceil() as i64;
    last_date.checked_add_signed(Duration::days(days_to_target))
}

// ─── GET /vehicles/:vehicle_id/maintenance-status ────────────────

// Calcul pur (sans vérification d'accès) — réutilisé par le handler propriétaire
// (`maintenance_status`, après `check_read_access`) et par le dashboard admin
// (`admin_handler.rs::get_vehicle_summary_admin`, après `AdminUser`).
pub async fn compute_maintenance_status(
    db: &sqlx::PgPool,
    vehicle_id: Uuid,
) -> Result<Vec<MaintenanceStatus>, ()> {
    let today = Local::now().date_naive();

    let mileage_bounds = sqlx::query!(
        r#"
        SELECT
            (SELECT value FROM public.mileage_log WHERE vehicle_id = $1 ORDER BY recorded_at ASC, created_at ASC LIMIT 1) AS first_km,
            (SELECT recorded_at FROM public.mileage_log WHERE vehicle_id = $1 ORDER BY recorded_at ASC, created_at ASC LIMIT 1) AS first_date,
            (SELECT value FROM public.mileage_log WHERE vehicle_id = $1 ORDER BY recorded_at DESC, created_at DESC LIMIT 1) AS last_km,
            (SELECT recorded_at FROM public.mileage_log WHERE vehicle_id = $1 ORDER BY recorded_at DESC, created_at DESC LIMIT 1) AS last_date
        "#,
        vehicle_id
    )
    .fetch_one(db)
    .await;

    let mileage_bounds = mileage_bounds.map_err(|_| ())?;

    let types = sqlx::query_as!(
        MaintenanceType,
        r#"
        SELECT id, vehicle_id, label, interval_km, interval_months, active, created_at
        FROM public.maintenance_types
        WHERE vehicle_id = $1 AND active = true
        ORDER BY created_at ASC
        "#,
        vehicle_id
    )
    .fetch_all(db)
    .await;

    let types = types.map_err(|_| ())?;

    let mut statuses = Vec::with_capacity(types.len());

    for t in types {
        let last_entry = sqlx::query!(
            r#"SELECT e.performed_at, e.km_at_service FROM public.maintenance_entries e
               JOIN public.maintenance_entry_types met ON met.entry_id = e.id
               WHERE met.maintenance_type_id = $1
               ORDER BY e.performed_at DESC, e.created_at DESC LIMIT 1"#,
            t.id,
        )
        .fetch_optional(db)
        .await;

        let last_entry = last_entry.map_err(|_| ())?;

        let Some(last) = last_entry else {
            statuses.push(MaintenanceStatus {
                type_id: t.id,
                label: t.label,
                interval_km: t.interval_km,
                interval_months: t.interval_months,
                last_performed_at: None,
                last_km: None,
                next_due_km: None,
                next_due_date: None,
                overdue: false,
            });
            continue;
        };

        let next_due_km = t.interval_km.map(|i| last.km_at_service + i);
        let next_due_date_by_time = t
            .interval_months
            .and_then(|m| {
                last.performed_at
                    .checked_add_months(chrono::Months::new(m as u32))
            });

        let next_due_date_by_km = match (next_due_km, mileage_bounds.first_date, mileage_bounds.first_km, mileage_bounds.last_date, mileage_bounds.last_km) {
            (Some(target), Some(fd), Some(fk), Some(ld), Some(lk)) => {
                estimate_date_for_km(fd, fk, ld, lk, target)
            }
            _ => None,
        };

        let next_due_date = match (next_due_date_by_time, next_due_date_by_km) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        let overdue_by_km = match (next_due_km, mileage_bounds.last_km) {
            (Some(target), Some(current)) => current >= target,
            _ => false,
        };
        let overdue_by_time = next_due_date_by_time.map(|d| today >= d).unwrap_or(false);

        statuses.push(MaintenanceStatus {
            type_id: t.id,
            label: t.label,
            interval_km: t.interval_km,
            interval_months: t.interval_months,
            last_performed_at: Some(last.performed_at),
            last_km: Some(last.km_at_service),
            next_due_km,
            next_due_date,
            overdue: overdue_by_km || overdue_by_time,
        });
    }

    Ok(statuses)
}

pub async fn maintenance_status(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = check_read_access(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    match compute_maintenance_status(&state.db, vehicle_id).await {
        Ok(statuses) => (StatusCode::OK, Json(statuses)).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}
