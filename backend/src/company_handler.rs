// src/company_handler.rs — Gestion des entreprises et de la flotte

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use crate::state::AppState;
use common::{
    AddMemberPayload, AssignFleetRolePayload, AssignVehicleFleetPayload, Company, CompanyMember,
    CompanyWithStats, CreateCompanyPayload, CreateOrganizationPayload, FleetVehicle, Organization,
};

#[derive(serde::Serialize)]
struct ApiError {
    error: String,
}

fn err(status: StatusCode, msg: impl Into<String>) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: msg.into() }))
}

// ─── Helpers ─────────────────────────────────────────────────────

/// Renvoie le rôle fleet global de l'utilisateur (org_id IS NULL).
/// Si org_id est fourni, vérifie aussi ce rôle local en cas d'absence de rôle global.
async fn get_fleet_role(
    db: &sqlx::PgPool,
    user_id: Uuid,
    company_id: Uuid,
    org_id: Option<Uuid>,
) -> Result<Option<String>, sqlx::Error> {
    let global = sqlx::query_scalar!(
        "SELECT role FROM public.fleet_roles
         WHERE user_id = $1 AND company_id = $2 AND org_id IS NULL",
        user_id,
        company_id
    )
    .fetch_optional(db)
    .await?;

    if global.is_some() {
        return Ok(global);
    }

    if let Some(oid) = org_id {
        let local = sqlx::query_scalar!(
            "SELECT role FROM public.fleet_roles
             WHERE user_id = $1 AND company_id = $2 AND org_id = $3",
            user_id,
            company_id,
            oid
        )
        .fetch_optional(db)
        .await?;
        return Ok(local);
    }

    Ok(None)
}

async fn is_company_member(
    db: &sqlx::PgPool,
    user_id: Uuid,
    company_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM public.company_members
         WHERE user_id = $1 AND company_id = $2",
        user_id,
        company_id
    )
    .fetch_one(db)
    .await?;
    Ok(count.unwrap_or(0) > 0)
}

// ─── GET /api/companies ──────────────────────────────────────────

pub async fn list_companies(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let rows = sqlx::query!(
        r#"
        SELECT
            c.id,
            c.name,
            c.siret,
            c.created_by,
            c.created_at,
            (SELECT COUNT(*) FROM public.company_members cm WHERE cm.company_id = c.id)
                AS member_count,
            (SELECT COUNT(*) FROM public.vehicles v WHERE v.company_id = c.id)
                AS vehicle_count,
            (SELECT role FROM public.fleet_roles fr
             WHERE fr.user_id = $1 AND fr.company_id = c.id AND fr.org_id IS NULL
             LIMIT 1) AS my_role
        FROM public.companies c
        JOIN public.company_members cm2
          ON cm2.company_id = c.id AND cm2.user_id = $1
        ORDER BY c.created_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let companies: Vec<CompanyWithStats> = rows
                .into_iter()
                .map(|r| CompanyWithStats {
                    id: r.id,
                    name: r.name,
                    siret: r.siret,
                    created_by: r.created_by,
                    created_at: r.created_at,
                    member_count: r.member_count.unwrap_or(0),
                    vehicle_count: r.vehicle_count.unwrap_or(0),
                    my_role: r.my_role,
                })
                .collect();
            (StatusCode::OK, Json(companies)).into_response()
        }
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── POST /api/companies ─────────────────────────────────────────

pub async fn create_company(
    AuthenticatedUser(user_id): AuthenticatedUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateCompanyPayload>,
) -> impl IntoResponse {
    if payload.name.trim().is_empty() {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "le nom de l'entreprise est requis",
        )
        .into_response();
    }

    let result = sqlx::query!(
        "INSERT INTO public.companies (name, siret, created_by)
         VALUES ($1, $2, $3)
         RETURNING id, name, siret, created_by, created_at",
        payload.name.trim(),
        payload.siret.as_deref().map(str::trim),
        user_id,
    )
    .fetch_one(&state.db)
    .await;

    let company = match result {
        Ok(r) => Company {
            id: r.id,
            name: r.name,
            siret: r.siret,
            created_by: r.created_by,
            created_at: r.created_at,
        },
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    // Le créateur devient automatiquement membre et fleet_admin global
    let _ = sqlx::query!(
        "INSERT INTO public.company_members (user_id, company_id) VALUES ($1, $2)",
        user_id,
        company.id
    )
    .execute(&state.db)
    .await;

    let _ = sqlx::query!(
        "INSERT INTO public.fleet_roles (user_id, company_id, org_id, role, granted_by)
         VALUES ($1, $2, NULL, 'fleet_admin', $1)",
        user_id,
        company.id
    )
    .execute(&state.db)
    .await;

    (StatusCode::CREATED, Json(company)).into_response()
}

// ─── GET /api/companies/:id ──────────────────────────────────────

pub async fn get_company(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let is_member = match is_company_member(&state.db, user_id, company_id).await {
        Ok(v) => v,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if !is_member {
        return err(
            StatusCode::NOT_FOUND,
            "entreprise introuvable ou accès refusé",
        )
        .into_response();
    }

    let result = sqlx::query!(
        r#"
        SELECT
            c.id, c.name, c.siret, c.created_by, c.created_at,
            (SELECT COUNT(*) FROM public.company_members cm WHERE cm.company_id = c.id)
                AS member_count,
            (SELECT COUNT(*) FROM public.vehicles v WHERE v.company_id = c.id)
                AS vehicle_count,
            (SELECT role FROM public.fleet_roles fr
             WHERE fr.user_id = $2 AND fr.company_id = c.id AND fr.org_id IS NULL
             LIMIT 1) AS my_role
        FROM public.companies c
        WHERE c.id = $1
        "#,
        company_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(r)) => {
            let company = CompanyWithStats {
                id: r.id,
                name: r.name,
                siret: r.siret,
                created_by: r.created_by,
                created_at: r.created_at,
                member_count: r.member_count.unwrap_or(0),
                vehicle_count: r.vehicle_count.unwrap_or(0),
                my_role: r.my_role,
            };
            (StatusCode::OK, Json(company)).into_response()
        }
        Ok(None) => err(StatusCode::NOT_FOUND, "entreprise introuvable").into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── DELETE /api/companies/:id ───────────────────────────────────

pub async fn delete_company(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let role = match get_fleet_role(&state.db, user_id, company_id, None).await {
        Ok(r) => r,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if role.as_deref() != Some("fleet_admin") {
        return err(
            StatusCode::FORBIDDEN,
            "réservé au fleet_admin global de l'entreprise",
        )
        .into_response();
    }

    match sqlx::query!("DELETE FROM public.companies WHERE id = $1", company_id)
        .execute(&state.db)
        .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── GET /api/companies/:id/organizations ────────────────────────

pub async fn list_organizations(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let is_member = match is_company_member(&state.db, user_id, company_id).await {
        Ok(v) => v,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if !is_member {
        return err(
            StatusCode::NOT_FOUND,
            "entreprise introuvable ou accès refusé",
        )
        .into_response();
    }

    let rows = sqlx::query_as!(
        Organization,
        "SELECT id, company_id, parent_org_id, name, created_at
         FROM public.organizations
         WHERE company_id = $1
         ORDER BY parent_org_id NULLS FIRST, name",
        company_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(orgs) => (StatusCode::OK, Json(orgs)).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── POST /api/companies/:id/organizations ───────────────────────

pub async fn create_organization(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateOrganizationPayload>,
) -> impl IntoResponse {
    let role = match get_fleet_role(&state.db, user_id, company_id, None).await {
        Ok(r) => r,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if role.as_deref() != Some("fleet_admin") {
        return err(
            StatusCode::FORBIDDEN,
            "réservé au fleet_admin global de l'entreprise",
        )
        .into_response();
    }

    if payload.name.trim().is_empty() {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "le nom de l'organisation est requis",
        )
        .into_response();
    }

    // Contrainte 2 niveaux max : si parent fourni, vérifier qu'il n'a pas lui-même de parent
    if let Some(parent_id) = payload.parent_org_id {
        let parent_check = sqlx::query!(
            "SELECT parent_org_id FROM public.organizations
             WHERE id = $1 AND company_id = $2",
            parent_id,
            company_id
        )
        .fetch_optional(&state.db)
        .await;

        match parent_check {
            Ok(None) => {
                return err(
                    StatusCode::NOT_FOUND,
                    "organisation parente introuvable dans cette entreprise",
                )
                .into_response()
            }
            Ok(Some(r)) if r.parent_org_id.is_some() => {
                return err(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "profondeur maximale de 2 niveaux dépassée",
                )
                .into_response()
            }
            Ok(Some(_)) => {} // OK : parent est niveau 1, enfant sera niveau 2
            Err(_) => {
                return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données")
                    .into_response()
            }
        }
    }

    let result = sqlx::query_as!(
        Organization,
        "INSERT INTO public.organizations (company_id, parent_org_id, name)
         VALUES ($1, $2, $3)
         RETURNING id, company_id, parent_org_id, name, created_at",
        company_id,
        payload.parent_org_id,
        payload.name.trim(),
    )
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(org) => (StatusCode::CREATED, Json(org)).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── DELETE /api/companies/:id/organizations/:oid ────────────────

pub async fn delete_organization(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((company_id, org_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let role = match get_fleet_role(&state.db, user_id, company_id, None).await {
        Ok(r) => r,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if role.as_deref() != Some("fleet_admin") {
        return err(
            StatusCode::FORBIDDEN,
            "réservé au fleet_admin global de l'entreprise",
        )
        .into_response();
    }

    match sqlx::query!(
        "DELETE FROM public.organizations WHERE id = $1 AND company_id = $2",
        org_id,
        company_id
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, "organisation introuvable").into_response()
        }
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── GET /api/companies/:id/members ──────────────────────────────

pub async fn list_members(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let is_member = match is_company_member(&state.db, user_id, company_id).await {
        Ok(v) => v,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if !is_member {
        return err(
            StatusCode::NOT_FOUND,
            "entreprise introuvable ou accès refusé",
        )
        .into_response();
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            u.id        AS user_id,
            cm.company_id,
            u.username,
            u.email,
            cm.joined_at,
            (SELECT role FROM public.fleet_roles fr
             WHERE fr.user_id = u.id AND fr.company_id = cm.company_id
               AND fr.org_id IS NULL
             LIMIT 1) AS fleet_role
        FROM public.company_members cm
        JOIN public.users u ON u.id = cm.user_id
        WHERE cm.company_id = $1
        ORDER BY cm.joined_at ASC
        "#,
        company_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let members: Vec<CompanyMember> = rows
                .into_iter()
                .map(|r| CompanyMember {
                    user_id: r.user_id,
                    company_id: r.company_id,
                    username: r.username,
                    email: r.email,
                    joined_at: r.joined_at,
                    fleet_role: r.fleet_role,
                })
                .collect();
            (StatusCode::OK, Json(members)).into_response()
        }
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── POST /api/companies/:id/members ─────────────────────────────

pub async fn add_member(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<AddMemberPayload>,
) -> impl IntoResponse {
    let role = match get_fleet_role(&state.db, user_id, company_id, None).await {
        Ok(r) => r,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if role.as_deref() != Some("fleet_admin") {
        return err(
            StatusCode::FORBIDDEN,
            "réservé au fleet_admin de l'entreprise",
        )
        .into_response();
    }

    let target = sqlx::query!(
        "SELECT id, username, email FROM public.users
         WHERE LOWER(email) = LOWER($1)",
        payload.email.trim()
    )
    .fetch_optional(&state.db)
    .await;

    let target = match target {
        Ok(Some(u)) => u,
        Ok(None) => {
            return err(
                StatusCode::NOT_FOUND,
                "aucun utilisateur trouvé avec cet email",
            )
            .into_response()
        }
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    match sqlx::query!(
        "INSERT INTO public.company_members (user_id, company_id) VALUES ($1, $2)",
        target.id,
        company_id
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            let member = CompanyMember {
                user_id: target.id,
                company_id,
                username: target.username,
                email: target.email,
                joined_at: chrono::Utc::now(),
                fleet_role: None,
            };
            (StatusCode::CREATED, Json(member)).into_response()
        }
        Err(sqlx::Error::Database(e)) if e.constraint() == Some("company_members_pkey") => {
            err(
                StatusCode::CONFLICT,
                "cet utilisateur est déjà membre de l'entreprise",
            )
            .into_response()
        }
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── DELETE /api/companies/:id/members/:uid ──────────────────────

pub async fn remove_member(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((company_id, target_uid)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let role = match get_fleet_role(&state.db, user_id, company_id, None).await {
        Ok(r) => r,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if role.as_deref() != Some("fleet_admin") {
        return err(
            StatusCode::FORBIDDEN,
            "réservé au fleet_admin de l'entreprise",
        )
        .into_response();
    }

    if target_uid == user_id {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "vous ne pouvez pas vous retirer vous-même",
        )
        .into_response();
    }

    match sqlx::query!(
        "DELETE FROM public.company_members WHERE user_id = $1 AND company_id = $2",
        target_uid,
        company_id
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, "membre introuvable").into_response()
        }
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── GET /api/companies/:id/fleet-roles ──────────────────────────

pub async fn list_fleet_roles(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let is_member = match is_company_member(&state.db, user_id, company_id).await {
        Ok(v) => v,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if !is_member {
        return err(
            StatusCode::NOT_FOUND,
            "entreprise introuvable ou accès refusé",
        )
        .into_response();
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            fr.id,
            fr.user_id,
            fr.company_id,
            fr.org_id,
            fr.role,
            fr.granted_by,
            fr.granted_at,
            u.username,
            u.email,
            o.name AS org_name
        FROM public.fleet_roles fr
        JOIN public.users u ON u.id = fr.user_id
        LEFT JOIN public.organizations o ON o.id = fr.org_id
        WHERE fr.company_id = $1
        ORDER BY fr.granted_at ASC
        "#,
        company_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(rows) => {
            let roles: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "id": r.id,
                        "user_id": r.user_id,
                        "company_id": r.company_id,
                        "org_id": r.org_id,
                        "role": r.role,
                        "granted_by": r.granted_by,
                        "granted_at": r.granted_at,
                        "username": r.username,
                        "email": r.email,
                        "org_name": r.org_name,
                    })
                })
                .collect();
            (StatusCode::OK, Json(roles)).into_response()
        }
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── POST /api/companies/:id/fleet-roles ─────────────────────────

pub async fn assign_fleet_role(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<AssignFleetRolePayload>,
) -> impl IntoResponse {
    let role = match get_fleet_role(&state.db, user_id, company_id, None).await {
        Ok(r) => r,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if role.as_deref() != Some("fleet_admin") {
        return err(
            StatusCode::FORBIDDEN,
            "réservé au fleet_admin global de l'entreprise",
        )
        .into_response();
    }

    if payload.role != "fleet_admin" && payload.role != "fleet_viewer" {
        return err(
            StatusCode::UNPROCESSABLE_ENTITY,
            "rôle invalide : utilisez fleet_admin ou fleet_viewer",
        )
        .into_response();
    }

    let is_target_member = match is_company_member(&state.db, payload.user_id, company_id).await {
        Ok(v) => v,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if !is_target_member {
        return err(
            StatusCode::NOT_FOUND,
            "l'utilisateur n'est pas membre de l'entreprise",
        )
        .into_response();
    }

    // Si org_id fourni, vérifier qu'elle appartient à l'entreprise
    if let Some(oid) = payload.org_id {
        let org_exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM public.organizations
             WHERE id = $1 AND company_id = $2",
            oid,
            company_id
        )
        .fetch_one(&state.db)
        .await;

        match org_exists {
            Ok(count) if count.unwrap_or(0) == 0 => {
                return err(
                    StatusCode::NOT_FOUND,
                    "organisation introuvable dans cette entreprise",
                )
                .into_response()
            }
            Err(_) => {
                return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données")
                    .into_response()
            }
            _ => {}
        }
    }

    // Supprime l'éventuel rôle existant pour ce scope, puis insère le nouveau
    let del = if payload.org_id.is_none() {
        sqlx::query!(
            "DELETE FROM public.fleet_roles
             WHERE user_id = $1 AND company_id = $2 AND org_id IS NULL",
            payload.user_id,
            company_id
        )
        .execute(&state.db)
        .await
    } else {
        sqlx::query!(
            "DELETE FROM public.fleet_roles
             WHERE user_id = $1 AND company_id = $2 AND org_id = $3",
            payload.user_id,
            company_id,
            payload.org_id
        )
        .execute(&state.db)
        .await
    };

    if del.is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response();
    }

    match sqlx::query!(
        "INSERT INTO public.fleet_roles (user_id, company_id, org_id, role, granted_by)
         VALUES ($1, $2, $3, $4, $5)",
        payload.user_id,
        company_id,
        payload.org_id,
        payload.role,
        user_id
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── DELETE /api/companies/:id/fleet-roles/:role_id ──────────────

pub async fn revoke_fleet_role(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((company_id, role_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let role = match get_fleet_role(&state.db, user_id, company_id, None).await {
        Ok(r) => r,
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if role.as_deref() != Some("fleet_admin") {
        return err(
            StatusCode::FORBIDDEN,
            "réservé au fleet_admin global de l'entreprise",
        )
        .into_response();
    }

    // Interdit de révoquer son propre rôle global
    let target = sqlx::query_scalar!(
        "SELECT user_id FROM public.fleet_roles
         WHERE id = $1 AND company_id = $2 AND org_id IS NULL",
        role_id,
        company_id
    )
    .fetch_optional(&state.db)
    .await;

    if let Ok(Some(target_uid)) = target {
        if target_uid == user_id {
            return err(
                StatusCode::UNPROCESSABLE_ENTITY,
                "vous ne pouvez pas révoquer votre propre rôle fleet_admin global",
            )
            .into_response();
        }
    }

    match sqlx::query!(
        "DELETE FROM public.fleet_roles WHERE id = $1 AND company_id = $2",
        role_id,
        company_id
    )
    .execute(&state.db)
    .await
    {
        Ok(r) if r.rows_affected() == 0 => {
            err(StatusCode::NOT_FOUND, "rôle introuvable").into_response()
        }
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── GET /api/companies/:id/vehicles ─────────────────────────────
// Vue flotte globale (fleet_admin ou fleet_viewer global uniquement)

pub async fn list_fleet_vehicles(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let fleet_role = match get_fleet_role(&state.db, user_id, company_id, None).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return err(
                StatusCode::FORBIDDEN,
                "accès à la vue flotte refusé — rôle fleet requis",
            )
            .into_response()
        }
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    let _ = fleet_role; // fleet_admin ou fleet_viewer : les deux peuvent lire

    let rows = sqlx::query_as!(
        FleetVehicle,
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
            v.company_id,
            v.org_id,
            o.name AS org_name
        FROM public.vehicles v
        LEFT JOIN public.organizations o ON o.id = v.org_id
        WHERE v.company_id = $1
        ORDER BY o.name NULLS LAST, v.make, v.model
        "#,
        company_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(vehicles) => (StatusCode::OK, Json(vehicles)).into_response(),
        Err(e) => {
            tracing::error!("list_fleet_vehicles error: {e:?}");
            err(StatusCode::INTERNAL_SERVER_ERROR, format!("erreur base de données: {e}")).into_response()
        }
    }
}

// ─── GET /api/companies/:id/organizations/:oid/vehicles ──────────
// Vue flotte d'une organisation (rôle global ou local accepté)

pub async fn list_org_vehicles(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path((company_id, org_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let global_role = get_fleet_role(&state.db, user_id, company_id, None).await;
    let local_role = get_fleet_role(&state.db, user_id, company_id, Some(org_id)).await;

    let has_access = match (&global_role, &local_role) {
        (Ok(Some(_)), _) | (_, Ok(Some(_))) => true,
        (Ok(None), Ok(None)) => false,
        _ => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    if !has_access {
        return err(
            StatusCode::FORBIDDEN,
            "accès à la vue flotte refusé — rôle fleet requis sur cette organisation",
        )
        .into_response();
    }

    let rows = sqlx::query_as!(
        FleetVehicle,
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
            v.company_id,
            v.org_id,
            o.name AS org_name
        FROM public.vehicles v
        LEFT JOIN public.organizations o ON o.id = v.org_id
        WHERE v.org_id = $1
        ORDER BY v.make, v.model
        "#,
        org_id
    )
    .fetch_all(&state.db)
    .await;

    match rows {
        Ok(vehicles) => (StatusCode::OK, Json(vehicles)).into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── POST /api/vehicles/:id/fleet ────────────────────────────────
// Rattache un véhicule à une entreprise/organisation
// Conditions : être owner du véhicule ET fleet_admin de l'entreprise cible

pub async fn assign_vehicle_to_fleet(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<AssignVehicleFleetPayload>,
) -> impl IntoResponse {
    let access = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access
         WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    match access {
        Ok(Some(r)) if r == "owner" => {}
        Ok(Some(_)) => {
            return err(
                StatusCode::FORBIDDEN,
                "seul le propriétaire du véhicule peut l'assigner à la flotte",
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

    let fleet_role = get_fleet_role(
        &state.db,
        user_id,
        payload.company_id,
        payload.org_id,
    )
    .await;

    match fleet_role {
        Ok(Some(r)) if r == "fleet_admin" => {}
        Ok(_) => {
            return err(
                StatusCode::FORBIDDEN,
                "vous devez être fleet_admin de l'entreprise (ou de l'organisation) pour assigner un véhicule",
            )
            .into_response()
        }
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    }

    match sqlx::query!(
        "UPDATE public.vehicles SET company_id = $1, org_id = $2 WHERE id = $3",
        payload.company_id,
        payload.org_id,
        vehicle_id
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}

// ─── DELETE /api/vehicles/:id/fleet ──────────────────────────────
// Détache un véhicule de la flotte (owner OU fleet_admin)

pub async fn remove_vehicle_from_fleet(
    AuthenticatedUser(user_id): AuthenticatedUser,
    Path(vehicle_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let vehicle = sqlx::query!(
        "SELECT company_id FROM public.vehicles WHERE id = $1",
        vehicle_id
    )
    .fetch_optional(&state.db)
    .await;

    let company_id = match vehicle {
        Ok(Some(r)) => r.company_id,
        Ok(None) => {
            return err(StatusCode::NOT_FOUND, "véhicule introuvable").into_response()
        }
        Err(_) => {
            return err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response()
        }
    };

    let access = sqlx::query_scalar!(
        "SELECT role FROM public.vehicle_access WHERE vehicle_id = $1 AND user_id = $2",
        vehicle_id,
        user_id
    )
    .fetch_optional(&state.db)
    .await;

    let is_owner = matches!(access, Ok(Some(ref r)) if r == "owner");

    let is_fleet_admin = if let Some(cid) = company_id {
        matches!(
            get_fleet_role(&state.db, user_id, cid, None).await,
            Ok(Some(ref r)) if r == "fleet_admin"
        )
    } else {
        false
    };

    if !is_owner && !is_fleet_admin {
        return err(
            StatusCode::FORBIDDEN,
            "droits insuffisants — owner du véhicule ou fleet_admin requis",
        )
        .into_response();
    }

    match sqlx::query!(
        "UPDATE public.vehicles SET company_id = NULL, org_id = NULL WHERE id = $1",
        vehicle_id
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => err(StatusCode::INTERNAL_SERVER_ERROR, "erreur base de données").into_response(),
    }
}
