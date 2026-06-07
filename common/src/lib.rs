use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- MODÈLE UTILISATEUR ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: Uuid, // ID provenant de Neon Auth
    pub email: String,
    pub full_name: String,
}

// --- MODÈLE VÉHICULE ---
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Vehicle {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub year: Option<i16>,
    pub vin: Option<String>,
    pub created_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
    pub role: Option<String>,
}

// --- MODÈLE CONTRAT ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: Uuid,
    pub vehicle_id: Uuid,      // Liaison vers Vehicle.id
    pub contract_type: String, // ex: Assurance, Entretien, Location
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

// --- MODÈLE KILOMÉTRAGE ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MileageEntry {
    pub id: Uuid,
    pub vehicle_id: Uuid, // Liaison vers Vehicle.id
    pub value: i32,       // Valeur en km
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "backend", derive(sqlx::Type))]
// Si tu stockes "owner", "editor" en minuscules dans ta DB :
#[cfg_attr(
    feature = "backend",
    sqlx(type_name = "varchar", rename_all = "lowercase")
)]
#[serde(rename_all = "lowercase")]
pub enum AccessRole {
    Owner,
    Editor,
    Viewer,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct VehicleWithAccess {
    pub id: Uuid,
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub owner_id: Uuid,
    #[serde(default)]
    pub archived_at: Option<DateTime<Utc>>,
    #[serde(rename = "my_role", alias = "role")]
    pub my_role: AccessRole,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)] // Sécurité : on n'envoie JAMAIS le hash au frontend
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ApiStatus {
    pub version: String,
    pub online: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
}

// ═══════════════════════════════════════════════════════════════
// CONTRATS LOA
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractLoa {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub km_allowed: i32,
    pub km_start: i32,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub price_per_extra_km: Option<f64>, // prix/km en cas de dépassement (optionnel)
    // Calculé à la lecture
    pub km_current: i32,
    pub km_consumed: i32,
    pub km_remaining: i32,
    pub status: String,
    pub days_remaining: i64,
    pub forecast_km: i32,
    pub overage_risk: bool,
    pub estimated_limit_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLoaPayload {
    pub km_allowed: i32,
    pub km_start: i32,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub price_per_extra_km: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════
// CONTRATS ASSURANCE
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractInsurance {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub km_annual_limit: i32,
    pub km_start: i32,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub insurer: Option<String>,
    // Calculé à la lecture
    pub km_current: i32,
    pub km_consumed: i32,  // depuis début de l'année d'assurance
    pub km_remaining: i32, // km_annual_limit - km_consumed
    pub status: String,    // active | exceeded | closed
    pub days_remaining: i64,
    pub forecast_km: i32, // projection à fin de période annuelle
    pub overage_risk: bool,
    pub estimated_limit_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInsurancePayload {
    pub km_annual_limit: i32,
    pub km_start: i32,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub insurer: Option<String>,
}

// ─── Modèles ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MileageLog {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub contract_loa_id: Option<Uuid>,
    pub contract_insurance_id: Option<Uuid>,
    pub value: i32,
    pub recorded_at: chrono::NaiveDate,
    pub source: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMileagePayload {
    pub value: i32,
    pub recorded_at: Option<chrono::NaiveDate>, // défaut : aujourd'hui
    pub contract_loa_id: Option<Uuid>,
    pub contract_insurance_id: Option<Uuid>,
    pub source: Option<String>, // défaut : "manual"
}

#[derive(Debug, Deserialize)]
pub struct JoinVehiclePayload {
    pub role: String, // "editor" ou "viewer"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShareCode {
    pub code: String,
    pub role: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateShareCodePayload {
    pub role: String, // "editor" | "viewer"
}

#[derive(Debug, Deserialize)]
pub struct UseShareCodePayload {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPreferences {
    pub notif_days_before: i32,
    pub notif_km_percent: i32,
    pub updated_once: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesPayload {
    pub notif_days_before: i32,
    pub notif_km_percent: i32,
}

// ═══════════════════════════════════════════════════════════════
// ENTREPRISE / FLOTTE
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub siret: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct Organization {
    pub id: Uuid,
    pub company_id: Uuid,
    pub parent_org_id: Option<Uuid>,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

/// Entreprise avec compteurs et rôle de l'utilisateur courant
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyWithStats {
    pub id: Uuid,
    pub name: String,
    pub siret: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub member_count: i64,
    pub vehicle_count: i64,
    /// Rôle global de l'utilisateur courant : "fleet_admin" | "fleet_viewer" | null
    pub my_role: Option<String>,
}

/// Membre d'entreprise avec son rôle global éventuel
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyMember {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub username: String,
    pub email: String,
    pub joined_at: DateTime<Utc>,
    /// Rôle global : "fleet_admin" | "fleet_viewer" | null
    pub fleet_role: Option<String>,
}

/// Véhicule vu depuis la flotte (inclut org)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct FleetVehicle {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub year: Option<i16>,
    pub vin: Option<String>,
    pub created_at: DateTime<Utc>,
    pub company_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
    pub org_name: Option<String>,
}

// ─── Payloads fleet ───────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateCompanyPayload {
    pub name: String,
    pub siret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrganizationPayload {
    pub name: String,
    pub parent_org_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberPayload {
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignFleetRolePayload {
    pub user_id: Uuid,
    pub org_id: Option<Uuid>,
    /// "fleet_admin" ou "fleet_viewer"
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignVehicleFleetPayload {
    pub company_id: Uuid,
    pub org_id: Option<Uuid>,
}

// ═══════════════════════════════════════════════════════════════
// RAPPORT DE FLOTTE
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FleetReportContract {
    pub contract_type: String,          // "loa" | "insurance"
    pub km_authorized: i32,             // km_allowed ou km_annual_limit
    pub km_consumed: i32,
    pub km_remaining: i32,
    pub status: String,                 // active | exceeded | closed
    pub days_remaining: i64,
    pub forecast_km: i32,
    pub overage_risk: bool,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub price_per_extra_km: Option<f64>, // LOA uniquement
    pub insurer: Option<String>,         // Insurance uniquement
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FleetReportVehicle {
    pub id: Uuid,
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub year: Option<i16>,
    pub org_name: Option<String>,
    pub contracts: Vec<FleetReportContract>,
}

// ═══════════════════════════════════════════════════════════════
// LICENCES
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LicenseStatus {
    /// "trial" | "active" | "expired"
    pub status: String,
    /// Date de fin de période d'essai (toujours présente)
    pub trial_ends_at: DateTime<Utc>,
    /// Date d'expiration du dernier jeton activé (None si aucun jeton)
    pub access_expires_at: Option<DateTime<Utc>>,
    /// Jours restants si dans la fenêtre d'alerte, None sinon (lifetime ou pas d'alerte)
    pub days_until_expiry: Option<i64>,
    /// "personal" | "fleet"
    pub license_type: String,
}

#[derive(Debug, Deserialize)]
pub struct RedeemTokenPayload {
    pub token: String,
}
