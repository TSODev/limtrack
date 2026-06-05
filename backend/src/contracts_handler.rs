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

    let result = sqlx::query!(
        r#"
        INSERT INTO public.contracts_insurance
            (vehicle_id, km_annual_limit, km_start, start_date, end_date, insurer)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        vehicle_id,
        payload.km_annual_limit,
        payload.km_start,
        payload.start_date,
        payload.end_date,
        payload.insurer.as_deref(),
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
