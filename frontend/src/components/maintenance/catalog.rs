// src/components/maintenance/catalog.rs
// Catalogue générique d'entretien — valeurs indicatives éditables, pas des données
// constructeur exactes (aucune API constructeur publique/gratuite fiable n'existe
// pour l'Europe). `fuel_type: None` = commun aux deux motorisations.

pub struct GenericTemplate {
    pub category: &'static str,
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

/// Ordre d'affichage des groupes dans le sélecteur (optgroup)
pub const CATEGORY_ORDER: &[&str] = &[
    "Général",
    "Pneumatiques & Freinage",
    "Carrosserie",
    "Mécanique moteur",
    "Mécanique de transmission",
    "Vitrage",
    "Électrique",
];

pub const GENERIC_CATALOG: &[GenericTemplate] = &[
    // Général
    GenericTemplate { category: "Général", label: "Contrôle technique", fuel_type: None, interval_km: None, interval_months: Some(24) },
    GenericTemplate { category: "Général", label: "Filtre habitacle", fuel_type: None, interval_km: Some(15000), interval_months: Some(12) },

    // Pneumatiques & Freinage
    GenericTemplate { category: "Pneumatiques & Freinage", label: "Pneus", fuel_type: None, interval_km: None, interval_months: None },
    GenericTemplate { category: "Pneumatiques & Freinage", label: "Permutation des pneus", fuel_type: None, interval_km: Some(10000), interval_months: None },
    GenericTemplate { category: "Pneumatiques & Freinage", label: "Liquide de frein", fuel_type: None, interval_km: None, interval_months: Some(24) },
    GenericTemplate { category: "Pneumatiques & Freinage", label: "Disques de frein", fuel_type: None, interval_km: Some(60000), interval_months: None },
    GenericTemplate { category: "Pneumatiques & Freinage", label: "Plaquettes de frein", fuel_type: Some("thermique"), interval_km: Some(30000), interval_months: None },
    GenericTemplate { category: "Pneumatiques & Freinage", label: "Plaquettes de frein", fuel_type: Some("electrique"), interval_km: Some(60000), interval_months: None },

    // Carrosserie
    GenericTemplate { category: "Carrosserie", label: "Retouche peinture", fuel_type: None, interval_km: None, interval_months: None },
    GenericTemplate { category: "Carrosserie", label: "Traitement anti-corrosion", fuel_type: None, interval_km: None, interval_months: Some(24) },
    GenericTemplate { category: "Carrosserie", label: "Réparation", fuel_type: None, interval_km: None, interval_months: None },

    // Mécanique moteur (thermique)
    GenericTemplate { category: "Mécanique moteur", label: "Vidange (huile + filtre)", fuel_type: Some("thermique"), interval_km: Some(15000), interval_months: Some(12) },
    GenericTemplate { category: "Mécanique moteur", label: "Filtre à air", fuel_type: Some("thermique"), interval_km: Some(20000), interval_months: Some(12) },
    GenericTemplate { category: "Mécanique moteur", label: "Filtre à carburant", fuel_type: Some("thermique"), interval_km: Some(40000), interval_months: None },
    GenericTemplate { category: "Mécanique moteur", label: "Courroie de distribution", fuel_type: Some("thermique"), interval_km: Some(120000), interval_months: Some(60) },
    GenericTemplate { category: "Mécanique moteur", label: "Courroie d'accessoires", fuel_type: Some("thermique"), interval_km: Some(80000), interval_months: None },
    GenericTemplate { category: "Mécanique moteur", label: "Bougies", fuel_type: Some("thermique"), interval_km: Some(40000), interval_months: None },
    GenericTemplate { category: "Mécanique moteur", label: "Turbo", fuel_type: Some("thermique"), interval_km: None, interval_months: None },
    GenericTemplate { category: "Mécanique moteur", label: "Batterie 12V", fuel_type: Some("thermique"), interval_km: None, interval_months: None },
    GenericTemplate { category: "Mécanique moteur", label: "Liquide de refroidissement", fuel_type: Some("thermique"), interval_km: None, interval_months: Some(48) },

    // Mécanique de transmission / pont
    GenericTemplate { category: "Mécanique de transmission", label: "Vidange boîte de vitesses", fuel_type: None, interval_km: Some(60000), interval_months: None },
    GenericTemplate { category: "Mécanique de transmission", label: "Embrayage", fuel_type: Some("thermique"), interval_km: None, interval_months: None },
    GenericTemplate { category: "Mécanique de transmission", label: "Cardans / soufflets", fuel_type: None, interval_km: None, interval_months: None },

    // Vitrage
    GenericTemplate { category: "Vitrage", label: "Pare-brise", fuel_type: None, interval_km: None, interval_months: None },
    GenericTemplate { category: "Vitrage", label: "Essuie-glaces", fuel_type: None, interval_km: None, interval_months: None },

    // Électrique
    GenericTemplate { category: "Électrique", label: "Diagnostic batterie de traction", fuel_type: Some("electrique"), interval_km: None, interval_months: Some(12) },
    GenericTemplate { category: "Électrique", label: "Liquide de refroidissement batterie/moteur", fuel_type: Some("electrique"), interval_km: None, interval_months: Some(48) },
    GenericTemplate { category: "Électrique", label: "Climatisation / pompe à chaleur", fuel_type: Some("electrique"), interval_km: None, interval_months: Some(24) },
];
