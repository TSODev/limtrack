// src/mileage_handler.rs

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

use common::MileageEntry;
use common::{CreateMileagePayload, MileageLog};

// ─── Limites métier ──────────────────────────────────────────────

const MAX_MILEAGE_ENTRIES_PER_DAY: i64 = 5;
const MAX_KM_PER_DAY: i32 = 1500;

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

// ─── POST /vehicles/:vehicle_id/mileage ──────────────────────────

pub async fn create_mileage(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateMileagePayload>,
) -> impl IntoResponse {
    // 1. Vérification du rôle
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    // 2. Validation du kilométrage
    if payload.value < 0 {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "La valeur du compteur ne peut pas être négative",
        )
        .into_response();
    }

    // 2b. Maximum MAX_MILEAGE_ENTRIES_PER_DAY relevés par date
    let recorded_at_check = payload.recorded_at.unwrap_or_else(|| Local::now().date_naive());
    let entries_this_day = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.mileage_log
         WHERE vehicle_id = $1 AND recorded_at = $2",
        vehicle_id,
        recorded_at_check
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    if entries_this_day >= MAX_MILEAGE_ENTRIES_PER_DAY {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!(
                "Maximum {} relevés par jour atteint pour cette date",
                MAX_MILEAGE_ENTRIES_PER_DAY
            ),
        )
        .into_response();
    }

    // 3. Vérifie cohérence vs le dernier relevé connu (valeur croissante + taux km/jour)
    let last_entry = sqlx::query!(
        "SELECT value, recorded_at FROM public.mileage_log
         WHERE vehicle_id = $1
         ORDER BY recorded_at DESC, created_at DESC
         LIMIT 1",
        vehicle_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données"));

    match last_entry {
        Ok(Some(last)) => {
            if payload.value < last.value {
                return err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!(
                        "La valeur ({} km) est inférieure au dernier relevé ({} km)",
                        payload.value, last.value
                    ),
                )
                .into_response();
            }
            let days_between = (recorded_at_check - last.recorded_at).num_days().max(1);
            let km_diff = payload.value - last.value;
            if km_diff / days_between as i32 > MAX_KM_PER_DAY {
                return err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!(
                        "Taux journalier trop élevé : {} km/j sur {} jour(s) (max {} km/jour)",
                        km_diff / days_between as i32,
                        days_between,
                        MAX_KM_PER_DAY
                    ),
                )
                .into_response();
            }
        }
        Ok(None) => {}
        Err(e) => return e.into_response(),
    }

    // 4. Validation de la source
    let source = payload.source.as_deref().unwrap_or("manual");
    if !matches!(source, "manual" | "import" | "api") {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "source doit être : manual, import ou api",
        )
        .into_response();
    }

    // 5. Vérifie que les contrats passés appartiennent bien au véhicule
    if let Some(loa_id) = payload.contract_loa_id {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM public.contracts_loa
             WHERE id = $1 AND vehicle_id = $2)",
            loa_id,
            vehicle_id
        )
        .fetch_one(&state.db)
        .await
        .unwrap_or(Some(false));

        if !exists.unwrap_or(false) {
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                "contract_loa_id invalide ou n'appartient pas à ce véhicule",
            )
            .into_response();
        }
    }

    if let Some(ins_id) = payload.contract_insurance_id {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM public.contracts_insurance
             WHERE id = $1 AND vehicle_id = $2)",
            ins_id,
            vehicle_id
        )
        .fetch_one(&state.db)
        .await
        .unwrap_or(Some(false));

        if !exists.unwrap_or(false) {
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                "contract_insurance_id invalide ou n'appartient pas à ce véhicule",
            )
            .into_response();
        }
    }

    // 6. Insertion
    let result = sqlx::query!(
        r#"
        INSERT INTO public.mileage_log
            (vehicle_id, contract_loa_id, contract_insurance_id, value, recorded_at, source)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, recorded_at, created_at
        "#,
        vehicle_id,
        payload.contract_loa_id,
        payload.contract_insurance_id,
        payload.value,
        recorded_at_check,
        source,
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(row) => (
            StatusCode::CREATED,
            Json(MileageLog {
                id: row.id,
                vehicle_id,
                contract_loa_id: payload.contract_loa_id,
                contract_insurance_id: payload.contract_insurance_id,
                value: payload.value,
                recorded_at: row.recorded_at,
                source: source.to_string(),
                created_at: row.created_at,
            }),
        )
            .into_response(),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur insertion relevé : {}", e),
        )
        .into_response(),
    }
}

// ─── DELETE /vehicles/:vehicle_id/mileage/:entry_id ─────────────

pub async fn delete_mileage(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, entry_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }
    match sqlx::query!(
        "DELETE FROM public.mileage_log WHERE id = $1 AND vehicle_id = $2",
        entry_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, "Relevé introuvable").into_response()
        }
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── GET /vehicles/:vehicle_id/mileage ───────────────────────────
// Retourne l'historique complet des relevés, du plus récent au plus ancien

pub async fn list_mileage(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Lecture : tout rôle suffit
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

    let rows = sqlx::query_as!(
        MileageLog,
        r#"
        SELECT
            id,
            vehicle_id,
            contract_loa_id,
            contract_insurance_id,
            value,
            recorded_at,
            source,
            created_at
        FROM public.mileage_log
        WHERE vehicle_id = $1
        ORDER BY recorded_at DESC, created_at DESC
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
