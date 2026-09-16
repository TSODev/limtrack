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
    pub contract_status: Option<String>, // "ok" | "warning" | "danger" | None
    /// "thermique" | "electrique" | "hybride" | None (non renseigné)
    pub fuel_type: Option<String>,
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
    #[serde(default)]
    pub fuel_type: Option<String>,
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
    pub auto_renew: bool,
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
    #[serde(default)]
    pub auto_renew: Option<bool>,
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

// ═══════════════════════════════════════════════════════════════
// VOYAGES PLANIFIÉS
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct PlannedTrip {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub label: String,
    pub estimated_km: i32,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    /// "none" | "daily" | "weekly" | "monthly"
    pub recurrence: String,
    pub recurrence_interval: i32,
    /// hebdo uniquement, 0=lundi..6=dimanche
    pub days_of_week: Option<Vec<i16>>,
    /// mensuel uniquement, 1-31
    pub day_of_month: Option<i16>,
    pub recurrence_end_date: Option<chrono::NaiveDate>,
    pub active: bool,
    /// Marqué manuellement par l'utilisateur — purement informatif, n'affecte pas
    /// usage-forecast (qui exclut déjà toute date passée, réalisé ou non).
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTripPayload {
    pub label: String,
    pub estimated_km: i32,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    #[serde(default)]
    pub recurrence: Option<String>, // défaut : "none"
    #[serde(default)]
    pub recurrence_interval: Option<i32>, // défaut : 1
    #[serde(default)]
    pub days_of_week: Option<Vec<i16>>,
    #[serde(default)]
    pub day_of_month: Option<i16>,
    #[serde(default)]
    pub recurrence_end_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTripPayload {
    pub label: Option<String>,
    pub estimated_km: Option<i32>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub recurrence: Option<String>,
    pub recurrence_interval: Option<i32>,
    pub days_of_week: Option<Vec<i16>>,
    pub day_of_month: Option<i16>,
    pub recurrence_end_date: Option<chrono::NaiveDate>,
    pub active: Option<bool>,
    pub completed: Option<bool>,
}

/// Point de la courbe de projection cumulée (échantillonnage hebdomadaire)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ForecastPoint {
    pub date: chrono::NaiveDate,
    pub cumulative_km: i32,
}

/// Résultat de la projection d'usage future (mileage réel + voyages planifiés)
/// vs le plafond du contrat LOA/assurance actif le plus restrictif.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UsageForecast {
    /// None si aucun contrat actif (rien à projeter)
    pub km_per_day_available: Option<f64>,
    /// Première date où l'usage projeté (réel + voyages) dépasserait le plafond
    pub unavailable_from: Option<chrono::NaiveDate>,
    /// Nombre de jours entre unavailable_from et la fin du contrat
    pub unavailable_days: Option<i64>,
    pub points: Vec<ForecastPoint>,
}

// ═══════════════════════════════════════════════════════════════
// CARNET D'ENTRETIEN
// ═══════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct MaintenanceType {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub label: String,
    pub interval_km: Option<i32>,
    pub interval_months: Option<i32>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMaintenanceTypePayload {
    pub label: String,
    pub interval_km: Option<i32>,
    pub interval_months: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMaintenanceTypePayload {
    pub label: Option<String>,
    pub interval_km: Option<i32>,
    pub interval_months: Option<i32>,
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct MaintenanceEntry {
    pub id: Uuid,
    pub vehicle_id: Uuid,
    pub label: String,
    pub performed_at: chrono::NaiveDate,
    pub km_at_service: i32,
    pub cost: Option<f64>,
    pub provider: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    /// Types d'entretien rattachés (une révision peut en couvrir plusieurs). Vide pour une
    /// entrée libre ("Autre").
    #[serde(default)]
    pub type_ids: Vec<Uuid>,
    /// Nombre de pièces jointes (calculé à la lecture, absent lors de la création)
    #[serde(default)]
    pub attachment_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateMaintenanceEntryPayload {
    /// Un ou plusieurs types d'entretien couverts par cette entrée (ex: révision = vidange + filtres)
    #[serde(default)]
    pub maintenance_type_ids: Vec<Uuid>,
    /// Requis si maintenance_type_ids est vide (entrée "Autre" libre)
    pub label: Option<String>,
    pub performed_at: chrono::NaiveDate,
    pub km_at_service: i32,
    pub cost: Option<f64>,
    pub provider: Option<String>,
    pub notes: Option<String>,
}

/// Statut agrégé d'un type d'entretien : dernière intervention + prochaine échéance estimée
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct MaintenanceStatus {
    pub type_id: Uuid,
    pub label: String,
    pub interval_km: Option<i32>,
    pub interval_months: Option<i32>,
    pub last_performed_at: Option<chrono::NaiveDate>,
    pub last_km: Option<i32>,
    pub next_due_km: Option<i32>,
    pub next_due_date: Option<chrono::NaiveDate>,
    pub overdue: bool,
}

/// Pièce jointe (facture) d'une fiche d'entretien. `file_path` n'est jamais exposé
/// au frontend — l'accès au contenu passe uniquement par l'endpoint de téléchargement.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "backend", derive(sqlx::FromRow))]
pub struct MaintenanceAttachment {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub vehicle_id: Uuid,
    pub original_filename: String,
    pub content_type: String,
    pub size_bytes: i32,
    pub created_at: DateTime<Utc>,
}
