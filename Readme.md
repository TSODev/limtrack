# odo.io

> **Gestion de flotte kilométrique** — Suivez vos contrats LOA et assurance, surveillez vos kilométrages et recevez des alertes avant de dépasser vos limites.

![Version](https://img.shields.io/badge/version-0.1.0-indigo)
![Rust](https://img.shields.io/badge/Rust-2021-orange)
![Leptos](https://img.shields.io/badge/Leptos-0.6-purple)
![Axum](https://img.shields.io/badge/Axum-0.7-blue)
![License](https://img.shields.io/badge/license-MIT-green)

---

## Présentation

**odo.io** est une application web full-stack écrite entièrement en Rust. Elle permet à des particuliers ou des petites flottes de :

- Gérer leurs véhicules et partager leur accès avec d'autres utilisateurs
- Suivre leurs contrats **LOA** et **Assurance** avec calculs de projection kilométrique
- Enregistrer leurs relevés kilométriques et visualiser leur trajectoire vs l'idéale
- Recevoir des **alertes** personnalisées avant de dépasser les limites contractuelles

---

## Stack technique

| Couche          | Technologie                                                         |
| --------------- | ------------------------------------------------------------------- |
| Frontend        | [Leptos](https://leptos.dev/) 0.6 (WASM)                            |
| Backend         | [Axum](https://github.com/tokio-rs/axum) 0.7                        |
| Base de données | PostgreSQL (NeonDB) via [SQLx](https://github.com/launchbadge/sqlx) |
| Styles          | [Tailwind CSS](https://tailwindcss.com/)                            |
| Auth            | JWT (jsonwebtoken) + bcrypt                                         |
| Build frontend  | [Trunk](https://trunkrs.dev/)                                       |
| Types partagés  | Crate `common` (workspace Cargo)                                    |

---

## Architecture

```
odo.io/
├── backend/          # API REST Axum
│   ├── src/
│   │   ├── main.rs
│   │   ├── auth.rs
│   │   ├── state.rs
│   │   ├── user_handler.rs
│   │   ├── vehicles_handler.rs
│   │   ├── contracts_handler.rs
│   │   ├── mileage_handler.rs
│   │   └── share_handler.rs
├── frontend/         # App Leptos/WASM
│   ├── src/
│   │   ├── pages/
│   │   │   ├── mainpage.rs
│   │   │   ├── login.rs
│   │   │   ├── register.rs
│   │   │   ├── profile.rs
│   │   │   └── home.rs
│   │   └── components/
│   │       ├── vehicle_dashboard.rs
│   │       ├── vehicle_header.rs
│   │       ├── vehicle_list.rs
│   │       ├── notification_bell.rs
│   │       ├── contracts/
│   │       │   ├── contract_list.rs
│   │       │   └── contracts_widget.rs
│   │       └── mileage/
│   │           ├── mileage_list.rs
│   │           └── mileage_widget.rs
├── common/           # Types partagés backend/frontend
│   └── src/lib.rs
├── Cargo.toml        # Workspace
└── Trunk.toml        # Config build frontend
```

---

## Fonctionnalités

### Véhicules
- ✅ Ajout, modification et suppression de véhicule
- ✅ Confirmation de suppression par plaque d'immatriculation
- ✅ Validation du format d'immatriculation (AA-111-AA)

### Partage et rôles
- ✅ Trois rôles : `Owner`, `Editor`, `Viewer`
- ✅ Partage via code à usage unique (format `XXX-XXX-XXX`, valable 24h)
- ✅ Révocation d'accès et départ d'un véhicule partagé

### Contrats
- ✅ Contrats **LOA** : km autorisés, date début/fin
- ✅ Contrats **Assurance** : limite annuelle, assureur
- ✅ Calculs en temps réel : km consommés, restants, projection à échéance
- ✅ Date estimée d'atteinte de la limite kilométrique
- ✅ Statuts : `active`, `exceeded`, `closed`

### Kilométrage
- ✅ Enregistrement de relevés kilométriques avec date
- ✅ Historique avec écart entre relevés
- ✅ Sparkline avec courbe réelle vs trajectoire idéale du contrat
- ✅ Indicateur visuel : en avance / en retard sur la trajectoire

### Notifications
- ✅ Icône cloche dans la navbar avec badge
- ✅ Alertes sur seuil kilométrique et proximité d'échéance
- ✅ Seuils personnalisables par utilisateur (jours et %)

### Profil
- ✅ Modification du mot de passe
- ✅ Préférences de notification (sliders)
- ✅ Gestion des partages (véhicules possédés et partagés)

### Interface
- ✅ Responsive mobile-first
- ✅ Bottom sheet pour la sélection de véhicule sur mobile
- ✅ Page d'accueil avec image de fond

---

## Prérequis

- [Rust](https://rustup.rs/) (nightly — requis par Leptos)
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- [Node.js](https://nodejs.org/) (pour Tailwind CSS via npx)
- PostgreSQL ou compte [NeonDB](https://neon.tech/)

---

## Installation

### 1. Cloner le dépôt

```bash
git clone https://github.com/ton-repo/odo.io.git
cd odo.io
```

### 2. Variables d'environnement

Créer un fichier `.env` à la racine du backend :

```env
DATABASE_URL=postgres://user:password@host/dbname
JWT_SECRET=votre_secret_jwt_tres_long_et_aleatoire
```

### 3. Base de données

Appliquer les migrations SQL (tables `users`, `vehicles`, `vehicle_access`, `contracts_loa`, `contracts_insurance`, `mileage_log`, `vehicle_share_codes`, `user_preferences`).

### 4. Lancer le backend

```bash
cd backend
cargo run
# API disponible sur http://127.0.0.1:3000
```

### 5. Lancer le frontend

```bash
cd frontend
trunk serve
# App disponible sur http://127.0.0.1:8080
```

---

## Configuration Trunk

Le fichier `Trunk.toml` proxifie les appels `/api` vers le backend :

```toml
[[proxy]]
rewrite = "/api"
backend = "http://127.0.0.1:3000/api"
```

---

## API — Routes principales

| Méthode      | Route                                   | Description                |
| ------------ | --------------------------------------- | -------------------------- |
| `POST`       | `/login`                                | Authentification           |
| `POST`       | `/api/user/register`                    | Inscription                |
| `GET`        | `/api/profile`                          | Profil utilisateur         |
| `GET/PUT`    | `/api/profile/preferences`              | Préférences notifications  |
| `GET`        | `/api/profile/shares`                   | Gestion des partages       |
| `GET/POST`   | `/api/vehicles`                         | Liste / création véhicules |
| `GET/DELETE` | `/api/vehicles/:id`                     | Détail / suppression       |
| `POST`       | `/api/vehicles/:id/share`               | Génère un code de partage  |
| `POST`       | `/api/vehicles/join`                    | Rejoindre via code         |
| `GET/POST`   | `/api/vehicles/:id/contracts/loa`       | Contrats LOA               |
| `GET/POST`   | `/api/vehicles/:id/contracts/insurance` | Contrats Assurance         |
| `GET/POST`   | `/api/vehicles/:id/mileage`             | Relevés kilométriques      |

---

## Roadmap

- [ ] App mobile native (Tauri Mobile iOS/Android)
- [ ] Export PDF / CSV des historiques
- [ ] Notifications push
- [ ] Tableau de bord multi-véhicules

---

## Licence

MIT © 2026 [TSODev](mailto:thierry.soulie@tsodev.fr)