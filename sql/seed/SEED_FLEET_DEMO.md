# Jeu de données démo — Fonctionnalités Fleet

Données de test pour présenter les fonctionnalités de gestion de flotte de LimTrack.

## Import

```bash
./dev/import_seed.sh
```

Le script est idempotent : il supprime les données `TC-*` existantes avant de réinsérer.

---

## Entreprise

| Champ  | Valeur           |
|--------|------------------|
| Nom    | TranspoCorp SA   |
| SIRET  | 12345678901234   |

### Structure organisationnelle

```
TranspoCorp SA
└── Direction des Opérations         (niveau 1)
    ├── Service Commercial           (niveau 2)
    └── Service Technique            (niveau 2)
```

---

## Comptes utilisateurs

| Identifiant      | Mot de passe        | Email                    | Rôle fleet                        | Licence                    |
|------------------|---------------------|--------------------------|-----------------------------------|----------------------------|
| `alice.martin`   | `FleetAdmin2024!`   | alice@transpocorp.fr     | **fleet_admin** (global)          | Active jusqu'au 2027-01-01 |
| `bob.dupont`     | `FleetBob2024!`     | bob@transpocorp.fr       | **fleet_viewer** (Svc Commercial) | Active jusqu'au 2026-12-01 |
| `charlie.moreau` | `FleetCharlie2024!` | charlie@transpocorp.fr   | **fleet_viewer** (Svc Technique)  | Période d'essai (→ 2026-09-01) |
| `diana.lefevre`  | `FleetDiana2024!`   | diana@transpocorp.fr     | Aucun (membre simple)             | Période d'essai (→ 2026-08-15) |

---

## Véhicules

| Plaque     | Véhicule              | Propriétaire     | Organisation             | Année | VIN                |
|------------|-----------------------|------------------|--------------------------|-------|--------------------|
| TC-001-AA  | Renault Mégane Estate | alice.martin     | Direction des Opérations | 2020  | VF1BZBZE0F0000001  |
| TC-002-BB  | Peugeot 308 SW        | bob.dupont       | Service Commercial       | 2020  | VF3LCBHZEKE000002  |
| TC-003-CC  | Citroën C5 Aircross   | bob.dupont       | Service Commercial       | 2020  | VF7RHRHMZLE000003  |
| TC-004-DD  | Volkswagen Passat SW  | charlie.moreau   | Service Technique        | 2021  | WVWZZZ3CZPE000004  |
| TC-005-EE  | Toyota Yaris Cross    | charlie.moreau   | Service Technique        | 2021  | JTMB3FV10N0000005  |
| TC-006-FF  | Ford Focus SW         | diana.lefevre    | Service Commercial       | 2020  | WF0WXXGCDWKE00006  |
| TC-007-GG  | Skoda Octavia Combi   | alice.martin     | Direction des Opérations | 2019  | TMBAH7NE0K0000007  |
| TC-008-HH  | BMW Série 1           | diana.lefevre    | Direction des Opérations | 2021  | WBA5E31080G000008  |

### Accès partagés (hors propriétaires)

| Véhicule   | Utilisateur      | Rôle   | Contexte                        |
|------------|------------------|--------|---------------------------------|
| TC-001-AA  | bob.dupont       | editor | Collaboration inter-équipe      |
| TC-002-BB  | diana.lefevre    | viewer | Backup commercial               |
| TC-007-GG  | charlie.moreau   | viewer | Véhicule pool Direction         |

---

## Contrats LOA

| Plaque    | Période                       | km alloués | km départ | km actuel | km consommés | Statut                          |
|-----------|-------------------------------|------------|-----------|-----------|--------------|----------------------------------|
| TC-001-AA | 2023-06-01 → **2026-06-01**   | 45 000     | 5 000     | 48 200    | 43 200 (96%) | ⚠️ Expire aujourd'hui, near limit |
| TC-002-BB | 2023-09-01 → 2027-09-01       | 80 000     | 8 000     | 60 800    | 52 800 (66%) | ✅ Sain                          |
| TC-003-CC | 2023-03-01 → **2026-03-01**   | 45 000     | 12 000    | 61 000    | 49 000       | ❌ Expiré + km dépassé           |
| TC-004-DD | 2024-01-15 → 2028-01-15       | 80 000     | 0         | 31 800    | 31 800 (40%) | ✅ Sain                          |
| TC-005-EE | 2024-06-01 → 2027-06-01       | 45 000     | 3 000     | 18 200    | 15 200 (34%) | ✅ Sain, faible usage            |
| TC-006-FF | 2023-12-01 → 2026-12-01       | 45 000     | 7 000     | 33 500    | 26 500 (59%) | 🟡 6 mois restants               |
| TC-007-GG | 2022-12-01 → 2026-12-01       | 60 000     | 15 000    | 76 800    | 61 800       | ❌ km dépassé (LOA encore valide) |
| TC-008-HH | 2024-06-01 → 2027-06-01       | 60 000     | 2 000     | 15 000    | 13 000 (22%) | ✅ Sain, très faible usage       |

---

## Contrats Assurance

| Plaque    | Assureur  | Période                       | Limite annuelle | km départ | km consommés | Statut                         |
|-----------|-----------|-------------------------------|-----------------|-----------|--------------|--------------------------------|
| TC-001-AA | AXA       | 2026-01-01 → 2027-01-01       | 15 000          | 41 000    | 7 200 (48%)  | ⚠️ Projection > limite (17 400) |
| TC-002-BB | Groupama  | 2026-01-01 → 2027-01-01       | 20 000          | 49 000    | 11 800 (59%) | ✅ Sain                         |
| TC-003-CC | MAIF      | 2025-03-01 → **2026-03-01**   | 15 000          | 38 000    | 23 000       | ❌ Expirée + km dépassée        |
| TC-004-DD | MMA       | 2025-07-15 → **2026-07-15**   | 20 000          | 18 000    | 13 800 (69%) | ⚠️ Expire dans ~6 semaines      |
| TC-005-EE | AXA       | 2025-10-01 → 2026-10-01       | 15 000          | 11 000    | 7 200 (48%)  | ✅ Sain                         |
| TC-006-FF | Allianz   | 2025-11-01 → 2026-11-01       | 15 000          | 25 500    | 8 000 (53%)  | ✅ Sain                         |
| TC-007-GG | AGF       | 2025-12-01 → 2026-12-01       | 25 000          | 60 000    | 16 800 (67%) | ✅ Sain                         |
| TC-008-HH | AXA       | 2026-03-01 → 2027-03-01       | 15 000          | 12 000    | 3 000 (20%)  | ✅ Sain, très faible usage      |

---

## Relevés kilométriques

79 entrées au total, source `manual`, couvrant la période 2022-12-01 → 2026-06-01.

| Plaque    | Nombre d'entrées | Premier relevé | Dernier relevé | Évolution          |
|-----------|-----------------|----------------|----------------|--------------------|
| TC-001-AA | 12              | 2023-07-01     | 2026-04-01     | 6 200 → 48 200 km  |
| TC-002-BB | 10              | 2023-10-01     | 2026-06-01     | 10 200 → 60 800 km |
| TC-003-CC | 10              | 2023-04-01     | 2026-02-15     | 13 500 → 61 000 km |
| TC-004-DD | 10              | 2024-03-01     | 2026-06-01     | 2 200 → 31 800 km  |
| TC-005-EE | 8               | 2024-08-01     | 2026-05-15     | 4 300 → 18 200 km  |
| TC-006-FF | 10              | 2024-02-01     | 2026-06-01     | 8 900 → 33 500 km  |
| TC-007-GG | 12              | 2023-03-01     | 2026-05-20     | 19 800 → 76 800 km |
| TC-008-HH | 7               | 2024-09-01     | 2026-05-20     | 4 500 → 15 000 km  |

---

## Scénarios de démonstration

### Panel Fleet (vue alice.martin — fleet_admin)
- Vue globale de tous les véhicules de TranspoCorp SA
- Filtrage par organisation (Direction / Service Commercial / Service Technique)
- TC-007-GG visible en alerte km dépassé malgré LOA valide

### Panel Fleet (vue bob.dupont — fleet_viewer Service Commercial)
- Visibilité limitée aux véhicules du Service Commercial (TC-002-BB, TC-003-CC, TC-006-FF)
- TC-003-CC en double alerte : LOA expirée + km dépassé

### Profil véhicule TC-001-AA (Mégane)
- LOA expire le jour de la démo, km à 96 %
- Assurance AXA avec projection de dépassement (overage_risk = true)
- Bob visible comme editor dans les accès partagés

### Profil véhicule TC-007-GG (Skoda)
- km_consumed (61 800) > km_allowed (60 000) → statut `exceeded`
- LOA date encore valide (→ déc. 2026) : cas typique de dépassement anticipé
- Charlie visible comme viewer dans les accès partagés
