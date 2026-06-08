// src/contracts_handler.rs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;

use common::{ContractInsurance, ContractLoa, CreateInsurancePayload, CreateLoaPayload};

// ─── Limites métier ──────────────────────────────────────────────

const MAX_LOA_PER_VEHICLE: i64 = 5;
const MAX_INSURANCE_PER_VEHICLE: i64 = 5;
const MAX_LEN_INSURER: usize = 200;

// ─── Erreur unifiée ──────────────────────────────────────────────

#[derive(Serialize)]
struct ApiError {
    error: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

// ─── Helper : date estimée d'atteinte de la limite ───────────────

fn estimate_limit_date(
    today: chrono::NaiveDate,
    start_date: chrono::NaiveDate,
    km_consumed: i32,
    km_remaining: i32,
) -> Option<chrono::NaiveDate> {
    let days_elapsed = (today - start_date).num_days();
    if days_elapsed <= 0 || km_consumed <= 0 {
        return None;
    }
    let km_per_day = km_consumed as f64 / days_elapsed as f64;
    if km_per_day <= 0.0 {
        return None;
    }
    let days_to_limit = (km_remaining as f64 / km_per_day).ceil() as i64;
    today.checked_add_signed(chrono::Duration::days(days_to_limit))
}

// ─── Helper : vérifie que l'utilisateur est owner du véhicule ────

async fn require_owner(
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
        Some("owner") => Ok(()),
        Some(_) => Err(err(
            StatusCode::FORBIDDEN,
            "Réservé au propriétaire du véhicule",
        )),
        None => Err(err(
            StatusCode::NOT_FOUND,
            "Véhicule introuvable ou accès refusé",
        )),
    }
}

// ─── POST /vehicles/:vehicle_id/contracts/loa ────────────────────

pub async fn create_loa(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateLoaPayload>,
) -> impl IntoResponse {
    if let Err(e) = require_owner(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    // Limite : MAX_LOA_PER_VEHICLE contrats LOA par véhicule
    let loa_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.contracts_loa WHERE vehicle_id = $1",
        vehicle_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    if loa_count >= MAX_LOA_PER_VEHICLE {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Limite de {} contrats LOA par véhicule atteinte", MAX_LOA_PER_VEHICLE),
        )
        .into_response();
    }

    if payload.km_allowed <= 0 {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "km_allowed doit être positif",
        )
        .into_response();
    }
    if payload.end_date <= payload.start_date {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "end_date doit être postérieure à start_date",
        )
        .into_response();
    }
    if payload.km_start < 0 {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "km_start ne peut pas être négatif",
        )
        .into_response();
    }

    // Un seul contrat LOA actif par période — vérifie les chevauchements de dates
    let overlap = sqlx::query_scalar!(
        r#"SELECT EXISTS(
            SELECT 1 FROM public.contracts_loa
            WHERE vehicle_id = $1
              AND start_date < $3
              AND end_date   > $2
        )"#,
        vehicle_id,
        payload.start_date,
        payload.end_date,
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(false))
    .unwrap_or(false);

    if overlap {
        return err(
            StatusCode::CONFLICT,
            "Un contrat LOA existe déjà sur cette période",
        )
        .into_response();
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO public.contracts_loa
            (vehicle_id, km_allowed, km_start, start_date, end_date, price_per_extra_km)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        vehicle_id,
        payload.km_allowed,
        payload.km_start,
        payload.start_date,
        payload.end_date,
        payload.price_per_extra_km,
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(row) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": row.id })),
        )
            .into_response(),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur création LOA : {}", e),
        )
        .into_response(),
    }
}

// ─── GET /vehicles/:vehicle_id/contracts/loa ─────────────────────

pub async fn list_loa(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let access = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access
         WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    if matches!(access, Ok(None) | Err(_)) {
        return err(
            StatusCode::NOT_FOUND,
            "Véhicule introuvable ou accès refusé",
        )
        .into_response();
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            l.id,
            l.vehicle_id,
            l.km_allowed,
            l.km_start,
            l.start_date,
            l.end_date,
            l.status,
            l.price_per_extra_km,
            COALESCE(
                (SELECT value FROM public.mileage_log m
                 WHERE m.vehicle_id = l.vehicle_id
                 ORDER BY recorded_at DESC, created_at DESC
                 LIMIT 1),
                l.km_start
            ) AS "km_current!"
        FROM public.contracts_loa l
        WHERE l.vehicle_id = $1
        ORDER BY l.start_date DESC
        "#,
        vehicle_id
    )
    .fetch_all(&state.db)
    .await;

    let rows = match rows {
        Ok(r) => r,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
    };

    let today = Local::now().date_naive();

    let contracts: Vec<ContractLoa> = rows
        .into_iter()
        .map(|r| {
            let km_consumed = (r.km_current - r.km_start).max(0);
            let km_remaining = (r.km_allowed - km_consumed).max(0);
            let days_total = (r.end_date - r.start_date).num_days().max(1);
            let days_elapsed = (today - r.start_date).num_days().max(0);
            let days_remaining = (r.end_date - today).num_days().max(0);

            let forecast_km = if days_elapsed > 0 {
                (km_consumed as f64 / days_elapsed as f64 * days_total as f64) as i32
            } else {
                0
            };

            let estimated_limit_date =
                estimate_limit_date(today, r.start_date, km_consumed, km_remaining);

            let overage_risk = forecast_km > r.km_allowed;

            let status = if km_consumed >= r.km_allowed {
                "exceeded".to_string()
            } else if today > r.end_date {
                "closed".to_string()
            } else {
                "active".to_string()
            };

            ContractLoa {
                id: r.id,
                vehicle_id: r.vehicle_id,
                km_allowed: r.km_allowed,
                km_start: r.km_start,
                start_date: r.start_date,
                end_date: r.end_date,
                price_per_extra_km: r.price_per_extra_km,
                km_current: r.km_current,
                km_consumed,
                km_remaining,
                status,
                days_remaining,
                forecast_km,
                overage_risk,
                estimated_limit_date,
            }
        })
        .collect();

    (StatusCode::OK, Json(contracts)).into_response()
}

// ─── DELETE /vehicles/:vehicle_id/contracts/loa/:contract_id ────

pub async fn delete_loa(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, contract_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = require_owner(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }
    match sqlx::query!(
        "DELETE FROM public.contracts_loa WHERE id = $1 AND vehicle_id = $2",
        contract_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, "Contrat introuvable").into_response()
        }
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── PATCH /vehicles/:vehicle_id/contracts/loa/:contract_id ─────

#[derive(serde::Deserialize)]
pub struct UpdateLoaPayload {
    pub price_per_extra_km: Option<f64>,
}

pub async fn update_loa(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, contract_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateLoaPayload>,
) -> impl IntoResponse {
    if let Err(e) = require_owner(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    match sqlx::query!(
        "UPDATE public.contracts_loa SET price_per_extra_km = $1
         WHERE id = $2 AND vehicle_id = $3",
        payload.price_per_extra_km,
        contract_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 =>
            err(StatusCode::NOT_FOUND, "Contrat introuvable").into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── POST /vehicles/:vehicle_id/contracts/insurance ──────────────

pub async fn create_insurance(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateInsurancePayload>,
) -> impl IntoResponse {
    if let Err(e) = require_owner(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    // Limite : MAX_INSURANCE_PER_VEHICLE contrats assurance par véhicule
    let ins_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.contracts_insurance WHERE vehicle_id = $1",
        vehicle_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    if ins_count >= MAX_INSURANCE_PER_VEHICLE {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Limite de {} contrats assurance par véhicule atteinte", MAX_INSURANCE_PER_VEHICLE),
        )
        .into_response();
    }

    if payload.km_annual_limit <= 0 {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "km_annual_limit doit être positif",
        )
        .into_response();
    }
    if payload.end_date <= payload.start_date {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "end_date doit être postérieure à start_date",
        )
        .into_response();
    }
    if payload.km_start < 0 {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "km_start ne peut pas être négatif",
        )
        .into_response();
    }
    if payload.insurer.as_deref().map(|s| s.len()).unwrap_or(0) > MAX_LEN_INSURER {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("insurer : {MAX_LEN_INSURER} caractères max")).into_response();
    }

    // Un seul contrat assurance actif par période — vérifie les chevauchements de dates
    let overlap = sqlx::query_scalar!(
        r#"SELECT EXISTS(
            SELECT 1 FROM public.contracts_insurance
            WHERE vehicle_id = $1
              AND start_date < $3
              AND end_date   > $2
        )"#,
        vehicle_id,
        payload.start_date,
        payload.end_date,
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(false))
    .unwrap_or(false);

    if overlap {
        return err(
            StatusCode::CONFLICT,
            "Un contrat assurance existe déjà sur cette période",
        )
        .into_response();
    }

    let result = sqlx::query!(
        r#"
        INSERT INTO public.contracts_insurance
            (vehicle_id, km_annual_limit, km_start, start_date, end_date, insurer, auto_renew)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        vehicle_id,
        payload.km_annual_limit,
        payload.km_start,
        payload.start_date,
        payload.end_date,
        payload.insurer.as_deref(),
        payload.auto_renew.unwrap_or(false),
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(row) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": row.id })),
        )
            .into_response(),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur création assurance : {}", e),
        )
        .into_response(),
    }
}

// ─── DELETE /vehicles/:vehicle_id/contracts/insurance/:contract_id

pub async fn delete_insurance(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, contract_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = require_owner(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }
    match sqlx::query!(
        "DELETE FROM public.contracts_insurance WHERE id = $1 AND vehicle_id = $2",
        contract_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, "Contrat introuvable").into_response()
        }
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── GET /vehicles/:vehicle_id/contracts/insurance ───────────────

pub async fn list_insurance(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let access = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access
         WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    if matches!(access, Ok(None) | Err(_)) {
        return err(
            StatusCode::NOT_FOUND,
            "Véhicule introuvable ou accès refusé",
        )
        .into_response();
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            i.id,
            i.vehicle_id,
            i.km_annual_limit,
            i.km_start,
            i.start_date,
            i.end_date,
            i.insurer,
            i.status,
            i.auto_renew,
            COALESCE(
                (SELECT value FROM public.mileage_log m
                 WHERE m.vehicle_id = i.vehicle_id
                 ORDER BY recorded_at DESC, created_at DESC
                 LIMIT 1),
                i.km_start
            ) AS "km_current!"
        FROM public.contracts_insurance i
        WHERE i.vehicle_id = $1
        ORDER BY i.start_date DESC
        "#,
        vehicle_id
    )
    .fetch_all(&state.db)
    .await;

    let rows = match rows {
        Ok(r) => r,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
    };

    let today = Local::now().date_naive();

    let contracts: Vec<ContractInsurance> = rows
        .into_iter()
        .map(|r| {
            let km_consumed = (r.km_current - r.km_start).max(0);
            let km_remaining = (r.km_annual_limit - km_consumed).max(0);
            let days_total = (r.end_date - r.start_date).num_days().max(1);
            let days_elapsed = (today - r.start_date).num_days().max(0);
            let days_remaining = (r.end_date - today).num_days().max(0);

            let forecast_km = if days_elapsed > 0 {
                (km_consumed as f64 / days_elapsed as f64 * days_total as f64) as i32
            } else {
                0
            };

            let estimated_limit_date =
                estimate_limit_date(today, r.start_date, km_consumed, km_remaining);

            let overage_risk = forecast_km > r.km_annual_limit;

            let status = if km_consumed >= r.km_annual_limit {
                "exceeded".to_string()
            } else if today > r.end_date {
                "closed".to_string()
            } else {
                "active".to_string()
            };

            ContractInsurance {
                id: r.id,
                vehicle_id: r.vehicle_id,
                km_annual_limit: r.km_annual_limit,
                km_start: r.km_start,
                start_date: r.start_date,
                end_date: r.end_date,
                insurer: r.insurer,
                auto_renew: r.auto_renew,
                km_current: r.km_current,
                km_consumed,
                km_remaining,
                status,
                days_remaining,
                forecast_km,
                overage_risk,
                estimated_limit_date,
            }
        })
        .collect();

    (StatusCode::OK, Json(contracts)).into_response()
}

// ─── PATCH /vehicles/:vehicle_id/contracts/insurance/:contract_id ─

#[derive(serde::Deserialize)]
pub struct UpdateInsurancePayload {
    pub auto_renew: Option<bool>,
}

pub async fn update_insurance(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, contract_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateInsurancePayload>,
) -> impl IntoResponse {
    if let Err(e) = require_owner(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    match sqlx::query!(
        "UPDATE public.contracts_insurance
         SET auto_renew = COALESCE($1, auto_renew)
         WHERE id = $2 AND vehicle_id = $3",
        payload.auto_renew,
        contract_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, "Contrat introuvable").into_response()
        }
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── POST /vehicles/:vehicle_id/contracts/insurance/:contract_id/renew

pub async fn renew_insurance(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, contract_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = require_owner(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    let contract = sqlx::query!(
        "SELECT km_annual_limit, insurer, start_date, end_date
         FROM public.contracts_insurance
         WHERE id = $1 AND vehicle_id = $2",
        contract_id,
        vehicle_id,
    )
    .fetch_optional(&state.db)
    .await;

    let c = match contract {
        Ok(Some(c)) => c,
        Ok(None) => return err(StatusCode::NOT_FOUND, "Contrat introuvable").into_response(),
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response()
        }
    };

    let successor_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM public.contracts_insurance
         WHERE vehicle_id = $1 AND start_date = $2)",
        vehicle_id,
        c.end_date,
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(false))
    .unwrap_or(false);

    if successor_exists {
        return err(
            StatusCode::CONFLICT,
            "Un contrat de renouvellement existe déjà pour cette période",
        )
        .into_response();
    }

    match do_renew(&state.db, vehicle_id, c.km_annual_limit, c.insurer.as_deref(), c.start_date, c.end_date).await {
        Ok(new_id) => (StatusCode::CREATED, Json(serde_json::json!({ "id": new_id }))).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur création renouvellement").into_response(),
    }
}

// ─── Logique de renouvellement partagée ──────────────────────────
// Utilisée par renew_insurance (manuel) et run_insurance_renewals (fond)

async fn do_renew(
    db: &sqlx::PgPool,
    vehicle_id: Uuid,
    km_annual_limit: i32,
    insurer: Option<&str>,
    old_start: chrono::NaiveDate,
    old_end: chrono::NaiveDate,
) -> Result<Uuid, sqlx::Error> {
    let new_start = old_end;
    let new_end = old_end + old_end.signed_duration_since(old_start);

    let km_start = sqlx::query_scalar!(
        "SELECT value FROM public.mileage_log WHERE vehicle_id = $1
         ORDER BY recorded_at DESC, created_at DESC LIMIT 1",
        vehicle_id
    )
    .fetch_optional(db)
    .await?
    .unwrap_or(0);

    let row = sqlx::query!(
        r#"INSERT INTO public.contracts_insurance
           (vehicle_id, km_annual_limit, km_start, start_date, end_date, insurer, auto_renew)
           VALUES ($1, $2, $3, $4, $5, $6, true)
           RETURNING id"#,
        vehicle_id,
        km_annual_limit,
        km_start,
        new_start,
        new_end,
        insurer,
    )
    .fetch_one(db)
    .await?;

    Ok(row.id)
}

// ─── Tâche de fond : renouvellements automatiques J-7 ────────────

pub async fn run_insurance_renewals(pool: &sqlx::PgPool) {
    let today = chrono::Local::now().date_naive();
    let window_end = today + chrono::Duration::days(7);

    let contracts = match sqlx::query!(
        r#"
        SELECT c.id, c.vehicle_id, c.km_annual_limit, c.insurer, c.start_date, c.end_date
        FROM public.contracts_insurance c
        WHERE c.auto_renew = true
          AND c.end_date <= $1
          AND NOT EXISTS (
              SELECT 1 FROM public.contracts_insurance s
              WHERE s.vehicle_id = c.vehicle_id
                AND s.start_date = c.end_date
          )
        "#,
        window_end
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!("run_insurance_renewals: erreur chargement — {}", e);
            return;
        }
    };

    for c in contracts {
        match do_renew(pool, c.vehicle_id, c.km_annual_limit, c.insurer.as_deref(), c.start_date, c.end_date).await {
            Ok(new_id) => tracing::info!(
                "Assurance {} → renouvellement {} créé (à partir du {})",
                c.id, new_id, c.end_date
            ),
            Err(e) => tracing::error!("Erreur renouvellement assurance {} : {}", c.id, e),
        }
    }
}
