// src/vehicles_handler.rs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::state::AppState; // <-- AppState vient de state.rs, plus défini ici
use common::JoinVehiclePayload;
use common::Vehicle;

// ─── Payloads ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateVehiclePayload {
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub year: Option<i16>,
    pub vin: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVehiclePayload {
    pub make: Option<String>,
    pub model: Option<String>,
    pub plate_number: Option<String>,
    pub year: Option<i16>,
    pub vin: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteVehiclePayload {
    pub plate_number: String,
}
// ─── Limites métier ──────────────────────────────────────────────

const MAX_VEHICLES_PER_USER: i64 = 10;

const MAX_LEN_MAKE: usize = 100;
const MAX_LEN_MODEL: usize = 100;
const MAX_LEN_PLATE: usize = 20;
const MAX_LEN_VIN: usize = 17;

// ─── Erreur unifiée ──────────────────────────────────────────────

#[derive(serde::Serialize)]
struct ApiError {
    error: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

// ─── GET /vehicles ───────────────────────────────────────────────

pub async fn list_vehicles(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rows = sqlx::query_as!(
        Vehicle,
        r#"
        SELECT
            v.id,
            v.owner_id,
            v.make,
            v.model,
            v.plate_number,
            v.year,
            v.vin,
            v.created_at,
            v.archived_at,
            va.role,
            CASE
                -- danger : km consommés >= km autorisés (indépendamment de la date)
                WHEN EXISTS (
                    SELECT 1 FROM public.contracts_loa l
                    WHERE l.vehicle_id = v.id
                      AND COALESCE(
                          (SELECT value FROM public.mileage_log WHERE vehicle_id = v.id ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                          l.km_start
                      ) - l.km_start >= l.km_allowed
                ) OR EXISTS (
                    SELECT 1 FROM public.contracts_insurance i
                    WHERE i.vehicle_id = v.id
                      AND COALESCE(
                          (SELECT value FROM public.mileage_log WHERE vehicle_id = v.id ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                          i.km_start
                      ) - i.km_start >= i.km_annual_limit
                ) THEN 'danger'
                -- warning : contrat actif expirant dans ≤30j OU projection de km dépasse le plafond
                WHEN EXISTS (
                    SELECT 1 FROM public.contracts_loa l
                    WHERE l.vehicle_id = v.id
                      AND l.end_date >= CURRENT_DATE
                      AND (
                          l.end_date <= CURRENT_DATE + 30
                          OR (
                              (COALESCE(
                                  (SELECT value FROM public.mileage_log WHERE vehicle_id = v.id ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                                  l.km_start
                              ) - l.km_start)::FLOAT
                              / GREATEST(CURRENT_DATE - l.start_date, 1)
                              * (l.end_date - l.start_date) > l.km_allowed
                          )
                      )
                ) OR EXISTS (
                    SELECT 1 FROM public.contracts_insurance i
                    WHERE i.vehicle_id = v.id
                      AND i.end_date >= CURRENT_DATE
                      AND (
                          i.end_date <= CURRENT_DATE + 30
                          OR (
                              (COALESCE(
                                  (SELECT value FROM public.mileage_log WHERE vehicle_id = v.id ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                                  i.km_start
                              ) - i.km_start)::FLOAT
                              / GREATEST(CURRENT_DATE - i.start_date, 1)
                              * (i.end_date - i.start_date) > i.km_annual_limit
                          )
                      )
                ) THEN 'warning'
                -- ok : au moins un contrat en cours, non dépassé
                WHEN EXISTS (
                    SELECT 1 FROM public.contracts_loa l WHERE l.vehicle_id = v.id AND l.end_date >= CURRENT_DATE
                ) OR EXISTS (
                    SELECT 1 FROM public.contracts_insurance i WHERE i.vehicle_id = v.id AND i.end_date >= CURRENT_DATE
                ) THEN 'ok'
                ELSE NULL
            END AS "contract_status?"
        FROM public.vehicles v
        JOIN public.vehicle_access va
          ON va.vehicle_id = v.id
         AND va.user_id = $1
        WHERE v.archived_at IS NULL
        ORDER BY v.created_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(vehicles) => (StatusCode::OK, Json(vehicles)).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── GET /vehicles/:id ───────────────────────────────────────────

pub async fn get_vehicle(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let row = sqlx::query_as!(
        Vehicle,
        r#"
        SELECT
            v.id,
            v.owner_id,
            v.make,
            v.model,
            v.plate_number,
            v.year,
            v.vin,
            v.created_at,
            v.archived_at,
            va.role,
            CASE
                WHEN EXISTS (
                    SELECT 1 FROM public.contracts_loa l
                    WHERE l.vehicle_id = v.id
                      AND COALESCE(
                          (SELECT value FROM public.mileage_log WHERE vehicle_id = v.id ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                          l.km_start
                      ) - l.km_start >= l.km_allowed
                ) OR EXISTS (
                    SELECT 1 FROM public.contracts_insurance i
                    WHERE i.vehicle_id = v.id
                      AND COALESCE(
                          (SELECT value FROM public.mileage_log WHERE vehicle_id = v.id ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                          i.km_start
                      ) - i.km_start >= i.km_annual_limit
                ) THEN 'danger'
                WHEN EXISTS (
                    SELECT 1 FROM public.contracts_loa l
                    WHERE l.vehicle_id = v.id
                      AND l.end_date >= CURRENT_DATE
                      AND (
                          l.end_date <= CURRENT_DATE + 30
                          OR (
                              (COALESCE(
                                  (SELECT value FROM public.mileage_log WHERE vehicle_id = v.id ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                                  l.km_start
                              ) - l.km_start)::FLOAT
                              / GREATEST(CURRENT_DATE - l.start_date, 1)
                              * (l.end_date - l.start_date) > l.km_allowed
                          )
                      )
                ) OR EXISTS (
                    SELECT 1 FROM public.contracts_insurance i
                    WHERE i.vehicle_id = v.id
                      AND i.end_date >= CURRENT_DATE
                      AND (
                          i.end_date <= CURRENT_DATE + 30
                          OR (
                              (COALESCE(
                                  (SELECT value FROM public.mileage_log WHERE vehicle_id = v.id ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                                  i.km_start
                              ) - i.km_start)::FLOAT
                              / GREATEST(CURRENT_DATE - i.start_date, 1)
                              * (i.end_date - i.start_date) > i.km_annual_limit
                          )
                      )
                ) THEN 'warning'
                WHEN EXISTS (
                    SELECT 1 FROM public.contracts_loa l WHERE l.vehicle_id = v.id AND l.end_date >= CURRENT_DATE
                ) OR EXISTS (
                    SELECT 1 FROM public.contracts_insurance i WHERE i.vehicle_id = v.id AND i.end_date >= CURRENT_DATE
                ) THEN 'ok'
                ELSE NULL
            END AS "contract_status?"
        FROM public.vehicles v
        JOIN public.vehicle_access va
          ON va.vehicle_id = v.id
         AND va.user_id = $1
        WHERE v.id = $2
        "#,
        user_id,
        vehicle_id
    )
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(vehicle)) => (StatusCode::OK, Json(vehicle)).into_response(),
        Ok(None) => err(
            StatusCode::NOT_FOUND,
            "véhicule introuvable ou accès refusé",
        )
        .into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── POST /vehicles ──────────────────────────────────────────────

pub async fn create_vehicle(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateVehiclePayload>,
) -> impl IntoResponse {
    if payload.make.trim().is_empty()
        || payload.model.trim().is_empty()
        || payload.plate_number.trim().is_empty()
    {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "make, model et plate_number sont requis",
        )
        .into_response();
    }
    if payload.make.len() > MAX_LEN_MAKE {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("make : {MAX_LEN_MAKE} caractères max")).into_response();
    }
    if payload.model.len() > MAX_LEN_MODEL {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("model : {MAX_LEN_MODEL} caractères max")).into_response();
    }
    if payload.plate_number.len() > MAX_LEN_PLATE {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("plate_number : {MAX_LEN_PLATE} caractères max")).into_response();
    }
    if payload.vin.as_deref().map(|v| v.len()).unwrap_or(0) > MAX_LEN_VIN {
        return err(StatusCode::UNPROCESSABLE_ENTITY, format!("vin : {MAX_LEN_VIN} caractères max")).into_response();
    }

    // Limite : MAX_VEHICLES_PER_USER véhicules actifs par propriétaire
    let owned_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM public.vehicles v
           JOIN public.vehicle_access va ON va.vehicle_id = v.id
           WHERE va.user_id = $1 AND va.role = 'owner' AND v.archived_at IS NULL"#,
        user_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    if owned_count >= MAX_VEHICLES_PER_USER {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Limite de {} véhicules actifs atteinte. Archivez un véhicule pour en ajouter un nouveau.", MAX_VEHICLES_PER_USER),
        )
        .into_response();
    }

    // Normalisation de la plaque AVANT la macro (évite le temporary value dropped)
    let plate = payload.plate_number.trim().to_uppercase();

    let row = sqlx::query_as!(
        Vehicle,
        r#"
        INSERT INTO public.vehicles (owner_id, make, model, plate_number, year, vin)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id,
            owner_id,
            make,
            model,
            plate_number,
            year,
            vin,
            created_at,
            archived_at,
            'owner' AS role,
            NULL::TEXT AS "contract_status?"
        "#,
        user_id,
        payload.make.trim(),
        payload.model.trim(),
        plate, // <-- variable, pas temporaire
        payload.year,
        payload.vin.as_deref().map(str::trim),
    )
    .fetch_one(&state.db)
    .await;

    match row {
        Ok(vehicle) => (StatusCode::CREATED, Json(vehicle)).into_response(),
        Err(sqlx::Error::Database(e)) if e.constraint() == Some("vehicles_plate_number_key") => {
            err(
                StatusCode::CONFLICT,
                "cette plaque d'immatriculation existe déjà",
            )
            .into_response()
        }
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── PATCH /vehicles/:id ─────────────────────────────────────────

pub async fn update_vehicle(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateVehiclePayload>,
) -> impl IntoResponse {
    let access = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access
         WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    let role = match access {
        Ok(Some(r)) => r,
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                "véhicule introuvable ou accès refusé",
            )
            .into_response()
        }
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if role == "viewer" {
        return err(StatusCode::FORBIDDEN, "droits insuffisants").into_response();
    }

    // Normalisation de la plaque AVANT la macro
    let plate = payload
        .plate_number
        .as_deref()
        .map(|s| s.trim().to_uppercase());

    let row = sqlx::query_as!(
        Vehicle,
        r#"
        UPDATE public.vehicles SET
            make         = COALESCE($1, make),
            model        = COALESCE($2, model),
            plate_number = COALESCE($3, plate_number),
            year         = COALESCE($4, year),
            vin          = COALESCE($5, vin)
        WHERE id = $6
        RETURNING
            id,
            owner_id,
            make,
            model,
            plate_number,
            year,
            vin,
            created_at,
            archived_at,
            $7 AS role,
            NULL::TEXT AS "contract_status?"
        "#,
        payload.make.as_deref().map(str::trim),
        payload.model.as_deref().map(str::trim),
        plate.as_deref(), // <-- variable, pas temporaire
        payload.year,
        payload.vin.as_deref().map(str::trim),
        vehicle_id,
        role,
    )
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some(vehicle)) => (StatusCode::OK, Json(vehicle)).into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "véhicule introuvable").into_response(),
        Err(sqlx::Error::Database(e)) if e.constraint() == Some("vehicles_plate_number_key") => {
            err(
                StatusCode::CONFLICT,
                "cette plaque d'immatriculation existe déjà",
            )
            .into_response()
        }
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── DELETE /vehicles/:id ────────────────────────────────────────

// ─── DELETE /vehicles/:id ────────────────────────────────────────

pub async fn delete_vehicle(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<DeleteVehiclePayload>,
) -> impl IntoResponse {
    // 1. Vérifie le rôle
    let access = sqlx::query!(
        "SELECT va.role, v.plate_number
         FROM public.vehicle_access va
         JOIN public.vehicles v ON v.id = va.vehicle_id
         WHERE va.vehicle_id = $1 AND va.user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    let row = match access {
        Ok(Some(r)) => r,
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                "véhicule introuvable ou accès refusé",
            )
            .into_response()
        }
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if row.role != "owner" {
        return err(
            StatusCode::FORBIDDEN,
            "seul le propriétaire peut supprimer ce véhicule",
        )
        .into_response();
    }

    // 2. Vérifie la plaque de confirmation
    let plate_normalized = payload.plate_number.trim().to_uppercase();
    if plate_normalized != row.plate_number {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "La plaque d'immatriculation ne correspond pas",
        )
        .into_response();
    }

    // 3. Supprime — les cascades nettoient tout le reste
    match sqlx::query!("DELETE FROM public.vehicles WHERE id = $1", vehicle_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── GET /vehicles/archived ──────────────────────────────────────

pub async fn list_archived_vehicles(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rows = sqlx::query_as!(
        Vehicle,
        r#"
        SELECT
            v.id,
            v.owner_id,
            v.make,
            v.model,
            v.plate_number,
            v.year,
            v.vin,
            v.created_at,
            v.archived_at,
            va.role,
            NULL::TEXT AS "contract_status?"
        FROM public.vehicles v
        JOIN public.vehicle_access va
          ON va.vehicle_id = v.id
         AND va.user_id = $1
        WHERE v.archived_at IS NOT NULL
        ORDER BY v.archived_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(vehicles) => (StatusCode::OK, Json(vehicles)).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── PATCH /vehicles/:id/archive ─────────────────────────────────

pub async fn archive_vehicle(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let role = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    match role {
        Ok(Some(r)) if r == "owner" => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "seul le propriétaire peut archiver ce véhicule",
            )
            .into_response()
        }
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                "véhicule introuvable ou accès refusé",
            )
            .into_response()
        }
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    }

    match sqlx::query!(
        "UPDATE public.vehicles SET archived_at = NOW() WHERE id = $1",
        vehicle_id
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── PATCH /vehicles/:id/unarchive ───────────────────────────────

pub async fn unarchive_vehicle(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let role = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    match role {
        Ok(Some(r)) if r == "owner" => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "seul le propriétaire peut désarchiver ce véhicule",
            )
            .into_response()
        }
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                "véhicule introuvable ou accès refusé",
            )
            .into_response()
        }
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    }

    match sqlx::query!(
        "UPDATE public.vehicles SET archived_at = NULL WHERE id = $1",
        vehicle_id
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}
