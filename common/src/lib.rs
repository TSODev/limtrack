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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: Uuid,
    pub owner_id: String, // Liaison vers UserProfile.user_id
    pub make: String,     // Marque (ex: Tesla)
    pub model: String,    // Modèle (ex: Model 3)
    pub plate_number: String,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, sqlx::Type)]
// Si tu stockes "owner", "editor" en minuscules dans ta DB :
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum AccessRole {
    Owner,
    Editor,
    Viewer,
}

//#[derive(Debug, Clone, Serialize, Deserialize)]
//pub struct VehicleWithAccess {
//    pub vehicle: Vehicle,
//    pub my_role: AccessRole,
//}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct VehicleWithAccess {
    pub id: Uuid, // L'id qui manquait selon l'erreur
    pub make: String,
    pub model: String,
    pub plate_number: String,
    pub owner_id: Uuid,
    pub my_role: AccessRole,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)] // Sécurité : on n'envoie JAMAIS le hash au frontend
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}
