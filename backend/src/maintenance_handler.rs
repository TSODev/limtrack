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

    // Label : copié depuis le type sélectionné (snapshot), ou fourni librement pour une entrée "Autre"
    let label = if let Some(type_id) = payload.maintenance_type_id {
        let type_label = sqlx::query_scalar!(
            "SELECT label FROM public.maintenance_types WHERE id = $1 AND vehicle_id = $2",
            type_id,
            vehicle_id,
        )
        .fetch_optional(&state.db)
        .await;

        match type_label {
            Ok(Some(l)) => l,
            Ok(None) => {
                return err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "maintenance_type_id invalide ou n'appartient pas à ce véhicule",
                )
                .into_response()
            }
            Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
        }
    } else {
        match payload.label.as_deref().map(str::trim) {
            Some(l) if !l.is_empty() => {
                if l.len() > MAX_LEN_LABEL {
                    return err(StatusCode::UNPROCESSABLE_ENTITY, format!("label : {MAX_LEN_LABEL} caractères max")).into_response();
                }
                l.to_string()
            }
            _ => {
                return err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "label requis quand maintenance_type_id est absent",
                )
                .into_response()
            }
        }
    };

    let result = sqlx::query!(
        r#"
        INSERT INTO public.maintenance_entries
            (vehicle_id, maintenance_type_id, label, performed_at, km_at_service, cost, provider, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
        vehicle_id,
        payload.maintenance_type_id,
        label,
        payload.performed_at,
        payload.km_at_service,
        payload.cost,
        payload.provider.as_deref().map(str::trim),
        payload.notes.as_deref().map(str::trim),
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(row) => (StatusCode::CREATED, Json(serde_json::json!({ "id": row.id }))).into_response(),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur création entretien : {}", e),
        )
        .into_response(),
    }
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
        SELECT id, vehicle_id, maintenance_type_id, label, performed_at, km_at_service, cost, provider, notes, created_at
        FROM public.maintenance_entries
        WHERE vehicle_id = $1
        ORDER BY performed_at DESC, created_at DESC
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
    match sqlx::query!(
        "DELETE FROM public.maintenance_entries WHERE id = $1 AND vehicle_id = $2",
        entry_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => err(StatusCode::NOT_FOUND, "Entretien introuvable").into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
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

pub async fn maintenance_status(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = check_read_access(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

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
    .fetch_one(&state.db)
    .await;

    let mileage_bounds = match mileage_bounds {
        Ok(r) => r,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

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
    .fetch_all(&state.db)
    .await;

    let types = match types {
        Ok(t) => t,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

    let mut statuses = Vec::with_capacity(types.len());

    for t in types {
        let last_entry = sqlx::query!(
            r#"SELECT performed_at, km_at_service FROM public.maintenance_entries
               WHERE maintenance_type_id = $1 ORDER BY performed_at DESC, created_at DESC LIMIT 1"#,
            t.id,
        )
        .fetch_optional(&state.db)
        .await;

        let last_entry = match last_entry {
            Ok(e) => e,
            Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
        };

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

    (StatusCode::OK, Json(statuses)).into_response()
}
