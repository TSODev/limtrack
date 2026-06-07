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
            va.role
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
            va.role
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
            'owner' AS role
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
            $7 AS role
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
            va.role
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
