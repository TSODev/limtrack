# LimTrack

<p align="center">
  <img src="frontend/public/icons/icon-192.png" width="96" alt="LimTrack" />
</p>

> **Gestion de flotte kilométrique** — Suivez vos contrats LOA et assurance, surveillez vos kilométrages et recevez des alertes avant de dépasser vos limites.

![Version](https://img.shields.io/badge/version-1.5.11-indigo)
![Rust](https://img.shields.io/badge/Rust-2021-orange)
![Leptos](https://img.shields.io/badge/Leptos-0.6-purple)
![Axum](https://img.shields.io/badge/Axum-0.7-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.x-yellow)
![License](https://img.shields.io/badge/license-AGPL--v3-blue)
![PWA](https://img.shields.io/badge/PWA-Mobile%20ready-brightgreen)
![Web](https://img.shields.io/badge/Web-SaaS%20ready-brightgreen)

<p align="center">
  <a href="https://apps.apple.com/app/id6777175237">
    <img src="https://developer.apple.com/app-store/marketing/guidelines/images/badge-download-on-the-app-store.svg" alt="Télécharger sur l'App Store" height="48" />
  </a>
</p>

---

## Présentation

**LimTrack** est une application full-stack écrite entièrement en Rust, **SaaS ready** (déployée sur le web) et **Mobile ready** (PWA installable + app iOS via Tauri). Elle permet à des particuliers et à des entreprises de :

- Gérer leurs véhicules et partager leur accès avec d'autres utilisateurs
- Suivre leurs contrats **LOA** et **Assurance** avec calculs de projection kilométrique et **estimation du coût de dépassement** (prix/km configurable)
- Enregistrer leurs relevés kilométriques et visualiser leur trajectoire vs l'idéale
- Planifier des **voyages futurs** (ponctuels ou récurrents) et projeter leur impact sur la capacité kilométrique restante
- Tenir un **carnet d'entretien** : types récurrents, historique multi-points (ex. révision = vidange + filtres en une seule fiche), photos de facture jointes, échéances estimées, impression PDF (fiche unique ou carnet complet)
- Recevoir des **alertes** personnalisées avant de dépasser les limites contractuelles
- Gérer une **flotte d'entreprise** : organisations, membres, rôles et véhicules assignés
- **Exporter** les données en PDF (contrat, flotte, entretien) et CSV (relevés kilométriques)
- Utiliser l'application sur **iOS** via Tauri Mobile (App Store, version payante) ou en **PWA** sur tout appareil — **application gratuite pour tout le monde** depuis la v1.4.0 (système de licences par jetons conservé dans le code mais désactivé)

---

## Stack technique

| Couche          | Technologie                                                         |
| --------------- | ------------------------------------------------------------------- |
| Frontend        | [Leptos](https://leptos.dev/) 0.6 (WASM)                            |
| Backend         | [Axum](https://github.com/tokio-rs/axum) 0.7                        |
| Base de données | PostgreSQL (auto-hébergé, VPS OVH) via [SQLx](https://github.com/launchbadge/sqlx) |
| Styles          | [Tailwind CSS](https://tailwindcss.com/)                            |
| Auth            | JWT (jsonwebtoken) + bcrypt                                         |
| Secrets         | Fichier `.env` (VPS OVH) / `.env` local (dev)                       |
| Build frontend  | [Trunk](https://trunkrs.dev/)                                       |
| Mobile          | [Tauri](https://tauri.app/) v2 (iOS)                                |
| Types partagés  | Crate `common` (workspace Cargo)                                    |

---

## Architecture

```
limtrack/
├── backend/src/
│   ├── main.rs
│   ├── auth.rs
│   ├── state.rs                    # AppState (db, resend_api_key)
│   ├── secrets.rs                  # chargement secrets via dotenvy (.env)
│   ├── notifier.rs                 # notifications email expiration (Resend)
│   ├── user_handler.rs
│   ├── vehicles_handler.rs
│   ├── contracts_handler.rs
│   ├── mileage_handler.rs
│   ├── trips_handler.rs            # voyages planifiés + projection d'usage
│   ├── maintenance_handler.rs      # carnet d'entretien (types + entrées multi-points)
│   ├── attachments_handler.rs      # pièces jointes (factures) sur les fiches d'entretien
│   ├── share_handler.rs
│   ├── company_handler.rs          # flotte : entreprises, orgs, membres, rôles
│   ├── license_handler.rs          # GET /api/profile/license + POST /api/profile/redeem
│   ├── license_middleware.rs       # middleware 402 — désactivé depuis v1.4.0 (app gratuite)
│   ├── request_license_handler.rs  # POST /api/license/request (public, délivrance auto)
│   ├── admin_handler.rs            # /api/admin/* — dashboard admin
│   ├── broadcast_handler.rs        # messages broadcast admin → utilisateurs
│   └── bin/
│       ├── gen_tokens.rs           # CLI génération jetons
│       ├── assign_license.rs       # CLI assignation jetons (manuel/batch CSV)
│       ├── notify_expiry.rs        # CLI notifications email manuelles
│       └── send_broadcast.rs       # CLI envoi broadcast
├── frontend/src/
│   ├── config.rs                   # API_BASE, CONTACT_EMAIL
│   ├── build.rs                    # APP_VERSION depuis git describe --tags (fallback Cargo.toml)
│   ├── pages/
│   │   ├── home.rs                 # page d'accueil publique (version affichée en footer)
│   │   ├── login.rs
│   │   ├── register.rs
│   │   ├── mainpage.rs
│   │   ├── fleet.rs                # gestion de flotte + export PDF/CSV
│   │   ├── profile.rs
│   │   ├── about.rs                # À propos, fonctionnalités, Ko-fi, GitHub Sponsors
│   │   ├── request_license.rs      # /request-license : formulaire licence gratuite (masqué, app gratuite)
│   │   ├── forgot_password.rs      # /forgot-password : demande de réinitialisation
│   │   ├── reset_password.rs       # /reset-password?token= : nouveau mot de passe
│   │   └── admin.rs                # /admin : dashboard administrateur
│   └── components/
│       ├── ui.rs                   # helpers : input_class(), get_token(), format_km()
│       ├── vehicle.rs
│       ├── vehicle_dashboard.rs
│       ├── vehicle_detail.rs
│       ├── vehicle_header.rs
│       ├── vehicle_list.rs
│       ├── notification_bell.rs
│       ├── contracts/
│       │   ├── contract_list.rs    # export PDF contrat + CSV relevés
│       │   └── contract_widget.rs
│       ├── mileage/
│       │   ├── mileage_list.rs
│       │   └── mileage_widget.rs
│       ├── trips/
│       │   ├── trip_list.rs        # CRUD voyages (récurrence quotidien/hebdo/mensuel)
│       │   └── trip_widget.rs
│       └── maintenance/
│           ├── catalog.rs          # catalogue générique thermique/électrique
│           ├── maintenance_list.rs # CRUD types + historique + impression PDF
│           └── maintenance_widget.rs
├── frontend/src-tauri/             # Tauri iOS
├── common/src/lib.rs               # Types partagés backend/frontend
├── Cargo.toml                      # Workspace (version 1.5.11)
├── sql/migrations/                 # Migrations SQL (001→019)
├── .github/workflows/
│   ├── deploy-frontend.yml         # CI/CD Cloudflare Pages
│   └── deploy-backend.yml          # CI/CD OVH VPS (build Docker + SSH deploy)
└── Trunk.toml
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

### Voyages planifiés
- ✅ Voyages ponctuels ou récurrents (quotidien / hebdomadaire / mensuel)
- ✅ Projection de la capacité kilométrique restante (par jour / semaine / mois) en tenant compte des voyages à venir
- ✅ Détection d'une indisponibilité prévisible avant l'échéance du contrat
- ✅ Overlay dédié sur la courbe de kilométrage (trajectoire idéale + projection avec voyages)

### Carnet d'entretien
- ✅ Types d'entretien récurrents (intervalle km et/ou mois), deux types pré-remplis à la création d'un véhicule (Vidange, Contrôle technique)
- ✅ Catalogue générique d'entretien (thermique/électrique) proposé à la saisie, filtré selon la motorisation du véhicule
- ✅ **Entretien multi-points** : une même fiche peut couvrir plusieurs types (ex. révision = vidange + filtre à air + filtre à huile), libellé auto-généré ou personnalisable
- ✅ Échéances estimées (km et date) à partir du rythme kilométrique réel du véhicule, statut "en retard" automatique
- ✅ Pièces jointes (photos de facture, PDF) — compression automatique des photos côté client avant l'envoi
- ✅ **Impression PDF** — fiche individuelle ou carnet complet (types + statuts + historique), photos jointes intégrées au document

### Notifications
- ✅ Icône cloche dans la navbar avec badge
- ✅ Alertes sur seuil kilométrique et proximité d'échéance
- ✅ Seuils personnalisables par utilisateur (jours et %)

### Gestion de flotte (entreprise)
- ✅ Création et gestion d'entreprises (nom, SIRET)
- ✅ Organisations hiérarchiques au sein d'une entreprise
- ✅ Gestion des membres (ajout, suppression)
- ✅ Rôles fleet : `admin`, `manager`, `viewer` — globaux ou par organisation
- ✅ Assignation de véhicules à la flotte / à une organisation
- ✅ Vue flotte complète : véhicules par entreprise et par organisation
- ✅ Suppression de compte utilisateur

### Licences — désactivées depuis la v1.4.0 (app gratuite pour tout le monde)
> Le code ci-dessous est **conservé intact** dans le projet (réactivable via deux constantes) mais **inactif en production** : `LICENSE_ENFORCEMENT_ENABLED = false` côté backend et `LICENSE_ENABLED = false` côté frontend masquent tout le système (402, UI licence, notifications d'expiration).

- Période d'essai gratuite de **3 mois** à l'inscription
- Activation par **jetons** (`XXXX-XXXX-XXXX-XXXX`) de 30, 90, 180 ou 365 jours
- Jetons cumulables (extension à partir de la date d'expiration courante)
- **Jetons lifetime** (`--lifetime`) pour accès illimité (~100 ans)
- **Deux types de licence** : `personal` (véhicules personnels) et `fleet` (accès gestion de flotte)
- Accès bloqué (`402 Payment Required`) si essai et licence expirés
- Mode lecture seule à l'expiration (`GET` autorisés, écritures bloquées)
- Affichage du statut licence dans le Profil (`trial` / `active` / `expired`)
- CLI `gen-tokens` : génère des jetons (`--days`, `--lifetime`, `--fleet`)
- CLI `assign-license` : assigne un jeton à un utilisateur (manuel ou batch CSV)
- **Alertes d'expiration in-app** dans la cloche (J-7/J-15/J-30 selon durée du jeton)
- **Notifications email** via Resend, envoyées automatiquement à 8h UTC quotidiennement

### Sécurité
- ✅ Vérification de la solidité des mots de passe via [`zxcvbn`](https://github.com/shssoichiro/zxcvbn-rs) (score ≥ 3/4) à l'inscription et au changement de mot de passe
- ✅ Feedback explicite retourné si le mot de passe est trop faible
- ✅ Détection des mots de passe dérivés du username ou de l'email
- ✅ **Réinitialisation du mot de passe** par email (token SHA-256, expiry 1h, via Resend)
- ✅ Rate limiting (`tower_governor`) sur les routes sensibles (login, inscription, mot de passe oublié) — 1 req/s, burst 5
- ✅ Limites métier : véhicules actifs par propriétaire, contrats par véhicule, relevés/jour, taux km/jour cohérent entre relevés

### Profil
- ✅ Modification du mot de passe
- ✅ **Mot de passe oublié** : lien sur la page de connexion → email de réinitialisation
- ✅ Préférences de notification (sliders)
- ✅ Gestion des partages (véhicules possédés et partagés)
- ✅ Suppression de compte (zone dangereuse)

### Interface
- ✅ Responsive mobile-first
- ✅ Bottom sheet pour la sélection de véhicule sur mobile
- ✅ Boutons icônes seuls sur mobile (partage, suppression)
- ✅ Safe areas iOS (notch, Dynamic Island, home indicator)
- ✅ Page d'accueil avec image de fond

### Mobile (Tauri iOS)
- ✅ **Disponible sur l'[App Store](https://apps.apple.com/app/id6777175237)** (achat unique)
- ✅ App iOS via Tauri v2
- ✅ Icône app personnalisée toutes tailles
- ✅ Testé sur Simulator iOS (iPhone 13 Pro)

---

## Prérequis

### Web
- [Rust](https://rustup.rs/) (nightly — requis par Leptos)
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- [Node.js](https://nodejs.org/) (pour Tailwind CSS via npx)
- PostgreSQL (local ou auto-hébergé)

### iOS (Tauri Mobile)
- macOS avec [Xcode](https://developer.apple.com/xcode/) 15+
- [Tauri CLI v2](https://tauri.app/) (`cargo install tauri-cli --version "^2"`)
- CocoaPods (`sudo gem install cocoapods`)
- Targets Rust iOS :
```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
```

---

## Installation

### 1. Cloner le dépôt

```bash
git clone https://github.com/TSODev/limtrack.git
cd limtrack
```

### 2. Variables d'environnement

Créer un fichier `.env` à la racine du backend (développement local uniquement) :

```env
DATABASE_URL=postgres://user:password@host/dbname
JWT_SECRET=votre_secret_jwt_tres_long_et_aleatoire
RESEND_API_KEY=re_...   # Notifications email (Resend) — désactivé si absent
```

> **Production (VPS OVH)** : les secrets sont définis dans `/opt/limtrack/.env` sur le serveur et chargés via les variables d'environnement Docker.

### 3. Base de données

Appliquer le schéma initial (`sql/schema/neon_tables.sql`) puis toutes les migrations SQL dans `sql/migrations/` **dans l'ordre numérique** (001 à 019 à ce jour) :

```bash
for f in sql/migrations/0*.sql; do psql $DATABASE_URL -f "$f"; done
```

Tables principales : `users`, `vehicles`, `vehicle_access`, `contracts_loa`, `contracts_insurance`, `mileage_log`, `vehicle_share_codes`, `user_preferences`, `companies`, `organizations`, `company_members`, `fleet_roles`, `license_tokens`, `license_requests`, `planned_trips`, `maintenance_types`, `maintenance_entries`, `maintenance_entry_types` (relation multi-points), `maintenance_attachments`, `broadcasts`. Détail complet dans [`CLAUDE.md`](CLAUDE.md#base-de-données).

### 4. Lancer le backend

```bash
cd backend
cargo run
# API disponible sur http://127.0.0.1:3000
```

### 5. Lancer le frontend web

```bash
cd frontend
trunk serve
# App disponible sur http://127.0.0.1:8080
```

### 6. Gérer les jetons de licence (désactivé par défaut, cf. section Licences)

```bash
cd backend

# Générer des jetons
cargo run --bin gen-tokens -- --count 5 --days 30           # 5 jetons 30j personal
cargo run --bin gen-tokens -- --count 1 --days 365 --fleet  # 1 jeton 1 an fleet
cargo run --bin gen-tokens -- --count 1 --lifetime --fleet  # 1 jeton lifetime fleet

# Assigner un jeton directement à un utilisateur
cargo run --bin assign-license -- --email user@example.com --token XXXX-XXXX-XXXX-XXXX

# Assignation en lot (fichier CSV : email,token)
cargo run --bin assign-license -- --file batch.csv

# Envoyer manuellement les notifications d'expiration
cargo run --bin notify-expiry
```

---

## Configuration

### URL API (`config.rs`)

L'URL de l'API est centralisée dans `frontend/src/config.rs` :

```rust
pub const API_BASE: &str = "https://api.limtrack.app";
```

Modifier cette valeur pour pointer vers votre propre backend.

### Trunk (`Trunk.toml`)

Le fichier `Trunk.toml` proxifie les appels `/api` vers le backend en développement local :

```toml
[[proxy]]
rewrite = "/api"
backend = "http://127.0.0.1:3000/api"
```

---

## Lancer sur iOS (Simulator)

### 1. Builder le frontend

```bash
cd frontend
trunk build --release
```

### 2. Servir les fichiers statiques

```bash
python3 -c "
import http.server, socketserver, os

class H(http.server.SimpleHTTPRequestHandler):
    def guess_type(self, p):
        return 'application/wasm' if p.endswith('.wasm') else super().guess_type(p)
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        super().end_headers()

os.chdir('dist')
with socketserver.TCPServer(('', 1430), H) as s:
    s.serve_forever()
"
```

### 3. Lancer Tauri iOS

```bash
cargo tauri ios dev --no-dev-server-wait
```

Puis sélectionner le Simulator dans Xcode et cliquer **▶ Run**.

---

## API — Routes principales

### Monitoring

| Méthode | Route     | Description                                  |
| ------- | --------- | -------------------------------------------- |
| `GET`   | `/health` | Health check (public, hors middleware) → `ok` |

### Auth & Profil

| Méthode      | Route                               | Description                                      |
| ------------ | ----------------------------------- | ------------------------------------------------ |
| `POST`       | `/login`                            | Authentification (email ou username)             |
| `POST`       | `/api/user/register`                | Inscription                                      |
| `POST`       | `/api/user/forgot-password`         | Demande de réinitialisation du mot de passe      |
| `POST`       | `/api/user/reset-password`          | Réinitialisation du mot de passe (token SHA-256) |
| `GET/DELETE` | `/api/profile`                      | Profil / suppression de compte                   |
| `POST`       | `/api/profile/password`             | Changement de mot de passe                       |
| `GET/PUT`    | `/api/profile/preferences`          | Préférences notifications                        |
| `GET`        | `/api/profile/shares`               | Gestion des partages                             |
| `GET`        | `/api/profile/license`              | Statut de la licence                             |
| `POST`       | `/api/profile/redeem`               | Activer un jeton de licence                      |
| `POST`       | `/api/license/request`              | Demander un jeton gratuit 365j (public)          |
| `POST`       | `/api/ios/activate`                 | Activation iOS App Store (public)                |

### Véhicules

| Méthode        | Route                                              | Description                        |
| -------------- | -------------------------------------------------- | ---------------------------------- |
| `GET/POST`     | `/api/vehicles`                                    | Liste / création (actifs seulement)|
| `GET`          | `/api/vehicles/archived`                           | Véhicules archivés                 |
| `GET/DELETE`   | `/api/vehicles/:id`                                | Détail / suppression               |
| `PATCH`        | `/api/vehicles/:id/archive`                        | Archiver (owner)                   |
| `PATCH`        | `/api/vehicles/:id/unarchive`                      | Désarchiver (owner)                |
| `POST`         | `/api/vehicles/:id/share`                          | Génère un code de partage          |
| `POST`         | `/api/vehicles/join`                               | Rejoindre via code                 |
| `DELETE`       | `/api/vehicles/:id/access/:user_id`                | Révoquer un accès                  |
| `DELETE`       | `/api/vehicles/:id/leave`                          | Quitter un véhicule partagé        |
| `GET/POST`     | `/api/vehicles/:id/contracts/loa`                  | Liste / création contrats LOA      |
| `PATCH/DELETE` | `/api/vehicles/:id/contracts/loa/:contract_id`     | Modifier / supprimer contrat LOA   |
| `GET/POST`     | `/api/vehicles/:id/contracts/insurance`                      | Liste / création contrats assurance         |
| `PATCH/DELETE` | `/api/vehicles/:id/contracts/insurance/:contract_id`         | Modifier `auto_renew` / supprimer           |
| `POST`         | `/api/vehicles/:id/contracts/insurance/:contract_id/renew`   | Renouveler immédiatement (crée successeur)  |
| `GET/POST`     | `/api/vehicles/:id/mileage`                        | Liste / ajout relevés km           |
| `DELETE`       | `/api/vehicles/:id/mileage/:entry_id`              | Supprimer un relevé km             |
| `POST/DELETE`  | `/api/vehicles/:id/fleet`                          | Assigner / retirer d'une flotte    |

### Voyages planifiés

| Méthode        | Route                                          | Description                                    |
| -------------- | ----------------------------------------------- | ----------------------------------------------- |
| `GET/POST`     | `/api/vehicles/:id/trips`                       | Liste / création d'un voyage planifié           |
| `PATCH/DELETE` | `/api/vehicles/:id/trips/:trip_id`              | Modifier / supprimer un voyage                  |
| `GET`          | `/api/vehicles/:id/usage-forecast`              | Projection km/jour disponible (voyages inclus)  |

### Carnet d'entretien

| Méthode        | Route                                                                    | Description                                  |
| -------------- | -------------------------------------------------------------------------- | --------------------------------------------- |
| `GET/POST`     | `/api/vehicles/:id/maintenance-types`                                       | Liste / création d'un type d'entretien         |
| `PATCH/DELETE` | `/api/vehicles/:id/maintenance-types/:type_id`                              | Modifier / supprimer un type                   |
| `GET/POST`     | `/api/vehicles/:id/maintenance-entries`                                     | Liste / création d'une fiche (multi-points via `maintenance_type_ids`) |
| `DELETE`       | `/api/vehicles/:id/maintenance-entries/:entry_id`                           | Supprimer une fiche                            |
| `GET`          | `/api/vehicles/:id/maintenance-status`                                      | Échéance estimée par type actif                |
| `GET/POST`     | `/api/vehicles/:id/maintenance-entries/:entry_id/attachments`               | Liste / upload de pièces jointes (multipart)   |
| `GET/DELETE`   | `/api/vehicles/:id/attachments/:attachment_id`                              | Téléchargement / suppression d'une pièce jointe |

### Gestion de flotte

| Méthode      | Route                                            | Description                      |
| ------------ | ------------------------------------------------ | -------------------------------- |
| `GET/POST`   | `/api/companies`                                 | Liste / création d'entreprises   |
| `GET/DELETE` | `/api/companies/:id`                             | Détail / suppression entreprise  |
| `GET/POST`   | `/api/companies/:id/organizations`               | Organisations d'une entreprise   |
| `DELETE`     | `/api/companies/:id/organizations/:oid`          | Supprimer une organisation       |
| `GET/POST`   | `/api/companies/:id/members`                     | Membres d'une entreprise         |
| `DELETE`     | `/api/companies/:id/members/:uid`                | Retirer un membre                |
| `GET/POST`   | `/api/companies/:id/fleet-roles`                 | Rôles fleet (global ou par org)  |
| `DELETE`     | `/api/companies/:id/fleet-roles/:role_id`        | Révoquer un rôle fleet           |
| `GET`        | `/api/companies/:id/vehicles`                    | Véhicules de la flotte           |
| `GET`        | `/api/companies/:id/organizations/:oid/vehicles` | Véhicules par organisation       |
| `GET`        | `/api/companies/:id/fleet-report`                | Rapport flotte complet (PDF/CSV) |

### Administration (`is_admin = true` requis)

| Méthode  | Route                           | Description                              |
| -------- | ------------------------------- | ---------------------------------------- |
| `GET`    | `/api/admin/stats`              | Statistiques globales                    |
| `GET`    | `/api/admin/users`              | Liste des utilisateurs                   |
| `PATCH`  | `/api/admin/users/:id`          | Édition admin (rôle, licence, accès...)  |
| `GET`    | `/api/admin/growth`             | Croissance hebdomadaire (12 semaines)    |
| `GET`    | `/api/admin/license-requests`   | Demandes de licences gratuites           |
| `POST`   | `/api/admin/generate-token`     | Générer un jeton depuis le dashboard     |
| `POST`   | `/api/admin/assign-license`     | Assigner un jeton existant à un compte   |
| `POST`   | `/api/admin/notify-expiry`      | Déclencher manuellement les emails d'expiration |
| `POST`   | `/api/admin/broadcasts`         | Créer un message broadcast               |
| `GET`    | `/api/admin/companies`          | Liste des entreprises (admin)            |

### Broadcasts

| Méthode | Route                     | Description                                          |
| ------- | ------------------------- | ----------------------------------------------------- |
| `GET`   | `/api/broadcasts/active`  | Message broadcast actif (filtré selon compte iOS)     |

---

## Déploiement production

| Service  | URL                        | Plateforme                        |
| -------- | -------------------------- | --------------------------------- |
| Frontend | https://limtrack.app       | Cloudflare Pages (GitHub Actions) |
| Backend  | https://api.limtrack.app   | OVH VPS — Docker + Caddy         |
| BDD      | PostgreSQL auto-hébergé    | OVH VPS — Docker (volume persistant) |

Le déploiement backend est automatisé via GitHub Actions : tout push sur `main` touchant le backend déclenche un build Docker et un déploiement SSH sur le VPS.

---

## Application gratuite

LimTrack est open source et **gratuit pour tout le monde** (web/PWA) depuis la v1.4.0 — aucune inscription à un système de licence n'est nécessaire, le système de jetons reste dans le code mais est désactivé. Seule la version iOS App Store reste payante (achat unique, accès à vie).

---

## Soutenir le projet

LimTrack est développé et hébergé bénévolement (~5 €/mois d'infrastructure). Si vous souhaitez contribuer :

- ☕ **Ko-fi** : [ko-fi.com/limtrack](https://ko-fi.com/limtrack)
- ♥ **GitHub Sponsors** : [github.com/sponsors/TSODev](https://github.com/sponsors/TSODev)

---

## Licence

GNU Affero General Public License v3.0 (AGPL-3.0) © 2026 [TSODev](mailto:thierry.soulie@tsodev.fr)

Voir [licence.md](licence.md) pour les détails.

---

## Remerciements

Ce projet a été développé avec l'assistance de [Claude](https://claude.ai), l'IA d'Anthropic.