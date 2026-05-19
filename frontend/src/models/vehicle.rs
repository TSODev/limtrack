use uuid::Uuid;

// src/models/vehicle.rs
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Vehicle {
    pub id: Uuid,
    pub make: String,
    pub model: String,
    pub plate_number: String,
    //    pub kilometrage: u32,
}
