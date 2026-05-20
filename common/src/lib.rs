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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vehicle {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub year: Option<i16>,
    pub vin: Option<String>,
    pub created_at: DateTime<Utc>,
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
    pub id: Uuid, // L'id qui manquait selon l'erreur
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub owner_id: Uuid,
    #[serde(rename = "my_role", alias = "role")] // accepte les deux
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
    // Calculé à la lecture
    pub km_current: i32,
    pub km_consumed: i32,    // km_current - km_start
    pub km_remaining: i32,   // km_allowed - km_consumed
    pub status: String,      // active | exceeded | closed
    pub days_remaining: i64, // jours jusqu'à end_date
    pub forecast_km: i32,    // projection km à échéance
    pub overage_risk: bool,
    pub estimated_limit_date: Option<chrono::NaiveDate>, // true si projection > km_allowed
}

#[derive(Debug, Deserialize)]
pub struct CreateLoaPayload {
    pub km_allowed: i32,
    pub km_start: i32,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
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
