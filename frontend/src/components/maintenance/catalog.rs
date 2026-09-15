// src/components/maintenance/catalog.rs
// Catalogue générique d'entretien — valeurs indicatives éditables, pas des données
// constructeur exactes (aucune API constructeur publique/gratuite fiable n'existe
// pour l'Europe). `fuel_type: None` = commun aux deux motorisations.

pub struct GenericTemplate {
    pub label: &'static str,
    pub fuel_type: Option<&'static str>,
    pub interval_km: Option<i32>,
    pub interval_months: Option<i32>,
}

impl GenericTemplate {
    pub fn is_periodic(&self) -> bool {
        self.interval_km.is_some() || self.interval_months.is_some()
    }
}

pub const GENERIC_CATALOG: &[GenericTemplate] = &[
    // Communs
    GenericTemplate { label: "Contrôle technique", fuel_type: None, interval_km: None, interval_months: Some(24) },
    GenericTemplate { label: "Pneus", fuel_type: None, interval_km: None, interval_months: None },
    GenericTemplate { label: "Essuie-glaces", fuel_type: None, interval_km: None, interval_months: None },
    GenericTemplate { label: "Filtre habitacle", fuel_type: None, interval_km: Some(15000), interval_months: Some(12) },
    GenericTemplate { label: "Liquide de frein", fuel_type: None, interval_km: None, interval_months: Some(24) },
    // Thermique
    GenericTemplate { label: "Vidange (huile + filtre)", fuel_type: Some("thermique"), interval_km: Some(15000), interval_months: Some(12) },
    GenericTemplate { label: "Filtre à air", fuel_type: Some("thermique"), interval_km: Some(20000), interval_months: Some(12) },
    GenericTemplate { label: "Plaquettes de frein", fuel_type: Some("thermique"), interval_km: Some(30000), interval_months: None },
    GenericTemplate { label: "Courroie/chaîne de distribution", fuel_type: Some("thermique"), interval_km: Some(120000), interval_months: Some(60) },
    GenericTemplate { label: "Liquide de refroidissement", fuel_type: Some("thermique"), interval_km: None, interval_months: Some(48) },
    GenericTemplate { label: "Bougies", fuel_type: Some("thermique"), interval_km: Some(40000), interval_months: None },
    GenericTemplate { label: "Batterie 12V", fuel_type: Some("thermique"), interval_km: None, interval_months: None },
    // Électrique
    GenericTemplate { label: "Diagnostic batterie de traction", fuel_type: Some("electrique"), interval_km: None, interval_months: Some(12) },
    GenericTemplate { label: "Liquide de refroidissement batterie/moteur", fuel_type: Some("electrique"), interval_km: None, interval_months: Some(48) },
    GenericTemplate { label: "Plaquettes de frein", fuel_type: Some("electrique"), interval_km: Some(60000), interval_months: None },
    GenericTemplate { label: "Climatisation / pompe à chaleur", fuel_type: Some("electrique"), interval_km: None, interval_months: Some(24) },
];
