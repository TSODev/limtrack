// src/trips_handler.rs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Datelike, Duration, Local, NaiveDate};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;

use common::{CreateTripPayload, ForecastPoint, PlannedTrip, UpdateTripPayload, UsageForecast};

// ─── Limites métier ──────────────────────────────────────────────

const MAX_TRIPS_PER_VEHICLE: i64 = 20;
const MAX_LEN_LABEL: usize = 100;
const MAX_OCCURRENCES_GUARD: i32 = 3000; // garde-fou anti-boucle infinie
const HORIZON_YEARS: i64 = 3; // plafond dur d'expansion si aucun contrat/recurrence_end_date

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

// ─── Validation des champs de récurrence ─────────────────────────

fn validate_trip_fields(
    label: &str,
    estimated_km: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
    recurrence: &str,
    recurrence_interval: i32,
    days_of_week: Option<&[i16]>,
    day_of_month: Option<i16>,
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
    if estimated_km <= 0 {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "estimated_km doit être positif",
        ));
    }
    if end_date < start_date {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "end_date doit être postérieure ou égale à start_date",
        ));
    }
    if !matches!(recurrence, "none" | "daily" | "weekly" | "monthly") {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "recurrence doit être : none, daily, weekly ou monthly",
        ));
    }
    if recurrence_interval < 1 {
        return Err(err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "recurrence_interval doit être ≥ 1",
        ));
    }
    if recurrence == "weekly" {
        match days_of_week {
            Some(days) if !days.is_empty() => {
                if days.iter().any(|d| !(0..=6).contains(d)) {
                    return Err(err(
                        StatusCode::UNPROCESSABLE_ENTITY,
                        "days_of_week : valeurs attendues entre 0 (lundi) et 6 (dimanche)",
                    ));
                }
            }
            _ => {
                return Err(err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "days_of_week requis pour une récurrence hebdomadaire",
                ))
            }
        }
    }
    if recurrence == "monthly" {
        match day_of_month {
            Some(d) if (1..=31).contains(&d) => {}
            _ => {
                return Err(err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "day_of_month requis (1-31) pour une récurrence mensuelle",
                ))
            }
        }
    }
    Ok(())
}

// ─── POST /vehicles/:vehicle_id/trips ────────────────────────────

pub async fn create_trip(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateTripPayload>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    let trip_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.planned_trips WHERE vehicle_id = $1",
        vehicle_id
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    if trip_count >= MAX_TRIPS_PER_VEHICLE {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("Limite de {} voyages planifiés par véhicule atteinte", MAX_TRIPS_PER_VEHICLE),
        )
        .into_response();
    }

    let recurrence = payload.recurrence.clone().unwrap_or_else(|| "none".to_string());
    let recurrence_interval = payload.recurrence_interval.unwrap_or(1);

    if let Err(e) = validate_trip_fields(
        &payload.label,
        payload.estimated_km,
        payload.start_date,
        payload.end_date,
        &recurrence,
        recurrence_interval,
        payload.days_of_week.as_deref(),
        payload.day_of_month,
    ) {
        return e.into_response();
    }

    let days_of_week = if recurrence == "weekly" { payload.days_of_week.clone() } else { None };
    let day_of_month = if recurrence == "monthly" { payload.day_of_month } else { None };

    let result = sqlx::query!(
        r#"
        INSERT INTO public.planned_trips
            (vehicle_id, label, estimated_km, start_date, end_date,
             recurrence, recurrence_interval, days_of_week, day_of_month, recurrence_end_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
        vehicle_id,
        payload.label.trim(),
        payload.estimated_km,
        payload.start_date,
        payload.end_date,
        recurrence,
        recurrence_interval,
        days_of_week.as_deref(),
        day_of_month,
        payload.recurrence_end_date,
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(row) => (StatusCode::CREATED, Json(serde_json::json!({ "id": row.id }))).into_response(),
        Err(e) => err(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur création voyage planifié : {}", e),
        )
        .into_response(),
    }
}

// ─── GET /vehicles/:vehicle_id/trips ──────────────────────────────

pub async fn list_trips(
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
        return err(StatusCode::NOT_FOUND, "Véhicule introuvable ou accès refusé").into_response();
    }

    let rows = sqlx::query_as!(
        PlannedTrip,
        r#"
        SELECT
            id, vehicle_id, label, estimated_km, start_date, end_date,
            recurrence, recurrence_interval, days_of_week, day_of_month,
            recurrence_end_date, active, created_at
        FROM public.planned_trips
        WHERE vehicle_id = $1
        ORDER BY start_date ASC
        "#,
        vehicle_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(trips) => (StatusCode::OK, Json(trips)).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── PATCH /vehicles/:vehicle_id/trips/:trip_id ──────────────────

pub async fn update_trip(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, trip_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateTripPayload>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }

    // Validation uniquement sur les champs fournis, en les combinant avec les valeurs actuelles.
    // Limite connue : `recurrence_end_date` ne peut pas être explicitement effacée via PATCH
    // (Option<T> ne distingue pas "absent" de "null" côté serde) — supprimer/recréer le voyage
    // pour repasser en récurrence sans date de fin.
    let current = sqlx::query!(
        r#"SELECT label, estimated_km, start_date, end_date, recurrence,
                  recurrence_interval, days_of_week, day_of_month,
                  recurrence_end_date, active
           FROM public.planned_trips WHERE id = $1 AND vehicle_id = $2"#,
        trip_id,
        vehicle_id,
    )
    .fetch_optional(&state.db)
    .await;

    let current = match current {
        Ok(Some(c)) => c,
        Ok(None) => return err(StatusCode::NOT_FOUND, "Voyage introuvable").into_response(),
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

    let label = payload.label.clone().unwrap_or(current.label);
    let estimated_km = payload.estimated_km.unwrap_or(current.estimated_km);
    let start_date = payload.start_date.unwrap_or(current.start_date);
    let end_date = payload.end_date.unwrap_or(current.end_date);
    let recurrence = payload.recurrence.clone().unwrap_or(current.recurrence);
    let recurrence_interval = payload.recurrence_interval.unwrap_or(current.recurrence_interval);
    let days_of_week = payload.days_of_week.clone().or(current.days_of_week);
    let day_of_month = payload.day_of_month.or(current.day_of_month);
    let recurrence_end_date = payload.recurrence_end_date.or(current.recurrence_end_date);
    let active = payload.active.unwrap_or(current.active);

    if let Err(e) = validate_trip_fields(
        &label,
        estimated_km,
        start_date,
        end_date,
        &recurrence,
        recurrence_interval,
        days_of_week.as_deref(),
        day_of_month,
    ) {
        return e.into_response();
    }

    let days_of_week = if recurrence == "weekly" { days_of_week } else { None };
    let day_of_month = if recurrence == "monthly" { day_of_month } else { None };

    match sqlx::query!(
        r#"
        UPDATE public.planned_trips SET
            label                = $1,
            estimated_km         = $2,
            start_date           = $3,
            end_date             = $4,
            recurrence           = $5,
            recurrence_interval  = $6,
            days_of_week         = $7,
            day_of_month         = $8,
            recurrence_end_date  = $9,
            active               = $10
        WHERE id = $11 AND vehicle_id = $12
        "#,
        label.trim(),
        estimated_km,
        start_date,
        end_date,
        recurrence,
        recurrence_interval,
        days_of_week.as_deref(),
        day_of_month,
        recurrence_end_date,
        active,
        trip_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => err(StatusCode::NOT_FOUND, "Voyage introuvable").into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── DELETE /vehicles/:vehicle_id/trips/:trip_id ─────────────────

pub async fn delete_trip(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((vehicle_id, trip_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(e) = require_editor(&state.db, vehicle_id, user_id).await {
        return e.into_response();
    }
    match sqlx::query!(
        "DELETE FROM public.planned_trips WHERE id = $1 AND vehicle_id = $2",
        trip_id,
        vehicle_id,
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => err(StatusCode::NOT_FOUND, "Voyage introuvable").into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    }
}

// ─── Expansion des occurrences d'un voyage planifié ──────────────
// Retourne, pour chaque jour couvert entre `from` et `horizon_end`, le km
// attribué à ce jour (estimated_km réparti uniformément sur la durée d'une occurrence).

struct TripRow {
    estimated_km: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
    recurrence: String,
    recurrence_interval: i32,
    days_of_week: Option<Vec<i16>>,
    day_of_month: Option<i16>,
    recurrence_end_date: Option<NaiveDate>,
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .day()
}

fn expand_occurrences(trip: &TripRow, from: NaiveDate, horizon_end: NaiveDate) -> Vec<(NaiveDate, f64)> {
    let mut daily_km: Vec<(NaiveDate, f64)> = Vec::new();

    if trip.end_date < trip.start_date || horizon_end < from {
        return daily_km;
    }

    let occurrence_len_days = (trip.end_date - trip.start_date).num_days() + 1;
    let per_day_km = trip.estimated_km as f64 / occurrence_len_days as f64;
    let recurrence_cap = trip
        .recurrence_end_date
        .map(|d| d.min(horizon_end))
        .unwrap_or(horizon_end);

    let mut add_occurrence = |occ_start: NaiveDate| {
        let occ_end = occ_start + Duration::days(occurrence_len_days - 1);
        let range_start = occ_start.max(from);
        let range_end = occ_end.min(horizon_end);
        let mut d = range_start;
        while d <= range_end {
            daily_km.push((d, per_day_km));
            d += Duration::days(1);
        }
    };

    match trip.recurrence.as_str() {
        "none" => {
            if trip.start_date <= recurrence_cap {
                add_occurrence(trip.start_date);
            }
        }
        "daily" => {
            let interval = trip.recurrence_interval.max(1) as i64;
            let mut occ_start = trip.start_date;
            let mut guard = 0;
            while occ_start <= recurrence_cap && guard < MAX_OCCURRENCES_GUARD {
                add_occurrence(occ_start);
                occ_start += Duration::days(interval);
                guard += 1;
            }
        }
        "weekly" => {
            let interval_weeks = trip.recurrence_interval.max(1) as i64;
            let days: Vec<i64> = trip
                .days_of_week
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|d| d as i64)
                .collect();
            if !days.is_empty() {
                let start_weekday = trip.start_date.weekday().num_days_from_monday() as i64;
                let week_start = trip.start_date - Duration::days(start_weekday);
                let mut week = week_start;
                let mut guard = 0;
                while week <= recurrence_cap && guard < MAX_OCCURRENCES_GUARD {
                    for &dow in &days {
                        let occ_start = week + Duration::days(dow);
                        if occ_start >= trip.start_date && occ_start <= recurrence_cap {
                            add_occurrence(occ_start);
                        }
                    }
                    week += Duration::days(7 * interval_weeks);
                    guard += 1;
                }
            }
        }
        "monthly" => {
            let interval_months = trip.recurrence_interval.max(1) as i32;
            let dom = trip
                .day_of_month
                .map(|d| d as u32)
                .unwrap_or_else(|| trip.start_date.day())
                .clamp(1, 31);
            let mut year = trip.start_date.year();
            let mut month = trip.start_date.month();
            let mut guard = 0;
            while guard < MAX_OCCURRENCES_GUARD {
                let last_day = last_day_of_month(year, month);
                let day = dom.min(last_day);
                let Some(occ_start) = NaiveDate::from_ymd_opt(year, month, day) else { break };
                if occ_start > recurrence_cap {
                    break;
                }
                if occ_start >= trip.start_date {
                    add_occurrence(occ_start);
                }
                let m0 = month as i32 - 1 + interval_months;
                year += m0.div_euclid(12);
                month = (m0.rem_euclid(12) + 1) as u32;
                guard += 1;
            }
        }
        _ => {}
    }

    daily_km
}

// ─── GET /vehicles/:vehicle_id/usage-forecast ────────────────────

struct ContractMetrics {
    km_current: i32,
    km_start: i32,
    km_allowed: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
}

fn build_forecast(metrics: &ContractMetrics, trips: &[TripRow], today: NaiveDate) -> UsageForecast {
    let km_consumed = (metrics.km_current - metrics.km_start).max(0);
    let days_elapsed = (today - metrics.start_date).num_days().max(0);
    let daily_rate = if days_elapsed > 0 { km_consumed as f64 / days_elapsed as f64 } else { 0.0 };
    let days_remaining = (metrics.end_date - today).num_days().max(0);

    let horizon_end = metrics.end_date.min(today + Duration::days(365 * HORIZON_YEARS));
    let from = today + Duration::days(1);

    let mut planned_by_day: HashMap<NaiveDate, f64> = HashMap::new();
    for trip in trips {
        for (date, km) in expand_occurrences(trip, from, horizon_end) {
            *planned_by_day.entry(date).or_insert(0.0) += km;
        }
    }
    let sum_future_planned_km: f64 = planned_by_day.values().sum();

    let km_per_day_available = if days_remaining > 0 {
        Some(((metrics.km_allowed - km_consumed) as f64 - sum_future_planned_km) / days_remaining as f64)
    } else {
        None
    };

    let mut unavailable_from: Option<NaiveDate> = None;
    let mut cumulative = 0.0_f64;
    let mut points = Vec::new();
    let mut d = from;
    let mut day_index = 0_i64;
    while d <= metrics.end_date {
        cumulative += daily_rate + planned_by_day.get(&d).copied().unwrap_or(0.0);
        if unavailable_from.is_none()
            && (km_consumed as f64 + cumulative) >= metrics.km_allowed as f64
        {
            unavailable_from = Some(d);
        }
        if day_index % 7 == 0 {
            points.push(ForecastPoint {
                date: d,
                cumulative_km: (metrics.km_current as f64 + cumulative).round() as i32,
            });
        }
        d += Duration::days(1);
        day_index += 1;
    }

    let unavailable_days = unavailable_from.map(|d| (metrics.end_date - d).num_days().max(0));

    UsageForecast {
        km_per_day_available,
        unavailable_from,
        unavailable_days,
        points,
    }
}

pub async fn usage_forecast(
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
        return err(StatusCode::NOT_FOUND, "Véhicule introuvable ou accès refusé").into_response();
    }

    let today = Local::now().date_naive();

    let loa = sqlx::query!(
        r#"
        SELECT l.km_allowed, l.km_start, l.start_date, l.end_date,
            COALESCE(
                (SELECT value FROM public.mileage_log m
                 WHERE m.vehicle_id = l.vehicle_id
                 ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                l.km_start
            ) AS "km_current!"
        FROM public.contracts_loa l
        WHERE l.vehicle_id = $1 AND l.start_date <= $2 AND l.end_date >= $2
        ORDER BY l.start_date DESC
        LIMIT 1
        "#,
        vehicle_id,
        today,
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None)
    .map(|r| ContractMetrics {
        km_current: r.km_current,
        km_start: r.km_start,
        km_allowed: r.km_allowed,
        start_date: r.start_date,
        end_date: r.end_date,
    });

    let insurance = sqlx::query!(
        r#"
        SELECT i.km_annual_limit, i.km_start, i.start_date, i.end_date,
            COALESCE(
                (SELECT value FROM public.mileage_log m
                 WHERE m.vehicle_id = i.vehicle_id
                 ORDER BY recorded_at DESC, created_at DESC LIMIT 1),
                i.km_start
            ) AS "km_current!"
        FROM public.contracts_insurance i
        WHERE i.vehicle_id = $1 AND i.start_date <= $2 AND i.end_date >= $2
        ORDER BY i.start_date DESC
        LIMIT 1
        "#,
        vehicle_id,
        today,
    )
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None)
    .map(|r| ContractMetrics {
        km_current: r.km_current,
        km_start: r.km_start,
        km_allowed: r.km_annual_limit,
        start_date: r.start_date,
        end_date: r.end_date,
    });

    if loa.is_none() && insurance.is_none() {
        return (
            StatusCode::OK,
            Json(UsageForecast {
                km_per_day_available: None,
                unavailable_from: None,
                unavailable_days: None,
                points: vec![],
            }),
        )
            .into_response();
    }

    let trips = sqlx::query!(
        r#"
        SELECT estimated_km, start_date, end_date, recurrence, recurrence_interval,
               days_of_week, day_of_month, recurrence_end_date
        FROM public.planned_trips
        WHERE vehicle_id = $1 AND active = true
        "#,
        vehicle_id
    )
    .fetch_all(&state.db)
    .await;

    let trips: Vec<TripRow> = match trips {
        Ok(rows) => rows
            .into_iter()
            .map(|r| TripRow {
                estimated_km: r.estimated_km,
                start_date: r.start_date,
                end_date: r.end_date,
                recurrence: r.recurrence,
                recurrence_interval: r.recurrence_interval,
                days_of_week: r.days_of_week,
                day_of_month: r.day_of_month,
                recurrence_end_date: r.recurrence_end_date,
            })
            .collect(),
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "Erreur base de données").into_response(),
    };

    let forecasts: Vec<UsageForecast> = [loa, insurance]
        .into_iter()
        .flatten()
        .map(|metrics| build_forecast(&metrics, &trips, today))
        .collect();

    // Le contrat le plus restrictif : date d'indisponibilité la plus proche,
    // puis à égalité le km/jour disponible le plus bas.
    let most_restrictive = forecasts.into_iter().min_by(|a, b| {
        match (a.unavailable_from, b.unavailable_from) {
            (Some(da), Some(db)) => da.cmp(&db),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a
                .km_per_day_available
                .partial_cmp(&b.km_per_day_available)
                .unwrap_or(std::cmp::Ordering::Equal),
        }
    });

    (StatusCode::OK, Json(most_restrictive.unwrap())).into_response()
}
