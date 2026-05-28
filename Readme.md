# odo.io

> **Gestion de flotte kilométrique** — Suivez vos contrats LOA et assurance, surveillez vos kilométrages et recevez des alertes avant de dépasser vos limites.

![Version](https://img.shields.io/badge/version-0.3.0-indigo)
![Rust](https://img.shields.io/badge/Rust-2021-orange)
![Leptos](https://img.shields.io/badge/Leptos-0.6-purple)
![Axum](https://img.shields.io/badge/Axum-0.7-blue)
![Tauri](https://img.shields.io/badge/Tauri-2.x-yellow)
![License](https://img.shields.io/badge/license-ELv2-blue)

---

## Présentation

**odo.io** est une application full-stack écrite entièrement en Rust. Elle permet à des particuliers et à des entreprises de :

- Gérer leurs véhicules et partager leur accès avec d'autres utilisateurs
- Suivre leurs contrats **LOA** et **Assurance** avec calculs de projection kilométrique
- Enregistrer leurs relevés kilométriques et visualiser leur trajectoire vs l'idéale
- Recevoir des **alertes** personnalisées avant de dépasser les limites contractuelles
- Gérer une **flotte d'entreprise** : organisations, membres, rôles et véhicules assignés
- Utiliser l'application sur **iOS** via Tauri Mobile

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
| Mobile          | [Tauri](https://tauri.app/) v2 (iOS)                                |
| Types partagés  | Crate `common` (workspace Cargo)                                    |

---

## Architecture

```
odo.io/
├── backend/          # API REST Axum
│   └── src/
│       ├── main.rs
│       ├── auth.rs
│       ├── state.rs
│       ├── handlers.rs         # login, status, helpers généraux
│       ├── user_handler.rs
│       ├── vehicles_handler.rs
│       ├── contracts_handler.rs
│       ├── mileage_handler.rs
│       ├── share_handler.rs
│       └── company_handler.rs  # gestion flotte : entreprises, orgs, membres, rôles
├── frontend/         # App Leptos/WASM + Tauri Mobile
│   ├── src/
│   │   ├── config.rs           # URL API centralisée (API_BASE)
│   │   ├── pages/
│   │   │   ├── mainpage.rs
│   │   │   ├── login.rs
│   │   │   ├── register.rs
│   │   │   ├── signup.rs
│   │   │   ├── fleet.rs        # page gestion de flotte (admin entreprise)
│   │   │   ├── profile.rs
│   │   │   └── home.rs
│   │   └── components/
│   │       ├── ui.rs           # helpers partagés (input_class, get_token, format_km)
│   │       ├── vehicle.rs      # VehicleCard
│   │       ├── vehicle_dashboard.rs
│   │       ├── vehicle_detail.rs
│   │       ├── vehicle_header.rs
│   │       ├── vehicle_list.rs
│   │       ├── notification_bell.rs
│   │       ├── contracts/
│   │       │   ├── contract_list.rs
│   │       │   └── contract_widget.rs
│   │       └── mileage/
│   │           ├── mileage_list.rs
│   │           └── mileage_widget.rs
│   └── src-tauri/    # Configuration Tauri Mobile
│       ├── src/
│       ├── gen/apple/          # Projet Xcode généré
│       ├── icons/              # Icônes app toutes tailles
│       └── tauri.conf.json
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

### Gestion de flotte (entreprise)
- ✅ Création et gestion d'entreprises (nom, SIRET)
- ✅ Organisations hiérarchiques au sein d'une entreprise
- ✅ Gestion des membres (ajout, suppression)
- ✅ Rôles fleet : `admin`, `manager`, `viewer` — globaux ou par organisation
- ✅ Assignation de véhicules à la flotte / à une organisation
- ✅ Vue flotte complète : véhicules par entreprise et par organisation
- ✅ Suppression de compte utilisateur

### Licences
- ✅ Période d'essai gratuite de **3 mois** à l'inscription
- ✅ Activation par **jetons** (`XXXX-XXXX-XXXX-XXXX`) de 30, 90, 180 ou 365 jours
- ✅ Jetons cumulables (extension à partir de la date d'expiration courante)
- ✅ Accès bloqué (`402 Payment Required`) si essai et licence expirés
- ✅ Affichage du statut licence dans le Profil (`trial` / `active` / `expired`)
- ✅ CLI `gen-tokens` pour générer des jetons en lot

### Sécurité
- ✅ Vérification de la solidité des mots de passe via [`zxcvbn`](https://github.com/shssoichiro/zxcvbn-rs) (score ≥ 3/4) à l'inscription et au changement de mot de passe
- ✅ Feedback explicite retourné si le mot de passe est trop faible
- ✅ Détection des mots de passe dérivés du username ou de l'email

### Profil
- ✅ Modification du mot de passe
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
- ✅ App iOS via Tauri v2
- ✅ Icône app personnalisée toutes tailles
- ✅ Testé sur Simulator iOS (iPhone 13 Pro)
- ✅ Sideloading compatible (Apple ID gratuit)

---

## Prérequis

### Web
- [Rust](https://rustup.rs/) (nightly — requis par Leptos)
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- [Node.js](https://nodejs.org/) (pour Tailwind CSS via npx)
- PostgreSQL ou compte [NeonDB](https://neon.tech/)

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

Appliquer les migrations SQL disponibles dans `migrations/` :

```bash
psql $DATABASE_URL -f migrations/001_license_tokens.sql
```

Tables créées : `users` (+ `trial_ends_at`, `access_expires_at`), `vehicles`, `vehicle_access`, `contracts_loa`, `contracts_insurance`, `mileage_log`, `vehicle_share_codes`, `user_preferences`, `companies`, `organizations`, `company_members`, `fleet_roles`, `license_tokens`.

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

### 6. Générer des jetons de licence

```bash
cd backend

# 5 jetons de 30 jours
cargo run --bin gen-tokens -- --count 5 --days 30

# 1 jeton d'un an
cargo run --bin gen-tokens -- --count 1 --days 365
```

Les jetons sont insérés en base et affichés **une seule fois** en clair. Transmettez-les à vos utilisateurs par email ou tout autre canal sécurisé. Ils peuvent être saisis dans la section **Profil → Licence** ou activés automatiquement via `POST /api/profile/redeem`.

---

## Configuration

### URL API (`config.rs`)

L'URL de l'API est centralisée dans `frontend/src/config.rs` :

```rust
pub const API_BASE: &str = "https://api.tsodev.fr";
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

### Véhicules & profil

| Méthode           | Route                                   | Description                    |
| ----------------- | --------------------------------------- | ------------------------------ |
| `POST`            | `/login`                                | Authentification               |
| `POST`            | `/api/user/register`                    | Inscription                    |
| `GET/DELETE`      | `/api/profile`                          | Profil / suppression de compte |
| `POST`            | `/api/profile/password`                 | Changement de mot de passe     |
| `GET/PUT`         | `/api/profile/preferences`              | Préférences notifications      |
| `GET`             | `/api/profile/shares`                   | Gestion des partages           |
| `GET/POST`        | `/api/vehicles`                         | Liste / création véhicules     |
| `GET/DELETE/PATCH`| `/api/vehicles/:id`                     | Détail / suppression / édition |
| `POST`            | `/api/vehicles/:id/share`               | Génère un code de partage      |
| `POST`            | `/api/vehicles/join`                    | Rejoindre via code             |
| `DELETE`          | `/api/vehicles/:id/access/:user_id`     | Révoquer un accès              |
| `DELETE`          | `/api/vehicles/:id/leave`               | Quitter un véhicule partagé    |
| `GET/POST`        | `/api/vehicles/:id/contracts/loa`       | Contrats LOA                   |
| `GET/POST`        | `/api/vehicles/:id/contracts/insurance` | Contrats Assurance             |
| `GET/POST`        | `/api/vehicles/:id/mileage`             | Relevés kilométriques          |

### Gestion de flotte

| Méthode           | Route                                              | Description                        |
| ----------------- | -------------------------------------------------- | ---------------------------------- |
| `GET/POST`        | `/api/companies`                                   | Liste / création d'entreprises     |
| `GET/DELETE`      | `/api/companies/:id`                               | Détail / suppression entreprise    |
| `GET/POST`        | `/api/companies/:id/organizations`                 | Organisations d'une entreprise     |
| `DELETE`          | `/api/companies/:id/organizations/:oid`            | Supprimer une organisation         |
| `GET/POST`        | `/api/companies/:id/members`                       | Membres d'une entreprise           |
| `DELETE`          | `/api/companies/:id/members/:uid`                  | Retirer un membre                  |
| `GET/POST`        | `/api/companies/:id/fleet-roles`                   | Rôles fleet (global ou par org)    |
| `DELETE`          | `/api/companies/:id/fleet-roles/:role_id`          | Révoquer un rôle fleet             |
| `GET`             | `/api/companies/:id/vehicles`                      | Véhicules de la flotte             |
| `GET`             | `/api/companies/:id/organizations/:oid/vehicles`   | Véhicules par organisation         |
| `POST/DELETE`     | `/api/vehicles/:id/fleet`                          | Assigner / retirer d'une flotte    |

---

## Déploiement production

| Service  | URL                   | Plateforme |
| -------- | --------------------- | ---------- |
| Frontend | https://odo.tsodev.fr | Netlify    |
| Backend  | https://api.tsodev.fr | Railway    |
| BDD      | NeonDB (PostgreSQL)   | Neon       |

---

## Roadmap

- ✅ App web responsive mobile-first
- ✅ App iOS via Tauri Mobile
- ✅ Gestion de flotte d'entreprise (entreprises, organisations, membres, rôles)
- ✅ Suppression de compte utilisateur
- ✅ Système de licences par jetons (trial 3 mois + activation par jeton)
- [ ] App Android via Tauri Mobile
- [ ] Sideloading iOS (Apple ID gratuit) → App Store (Apple Developer)
- [ ] Export PDF / CSV des historiques
- [ ] Notifications push natives
- [ ] Notification d'expiration de licence (in-app + email) avant et après expiration
- [ ] Tableau de bord multi-véhicules

---

## Licence

Elastic License 2.0 (ELv2) © 2026 [TSODev](mailto:thierry.soulie@tsodev.fr)

---

## Remerciements

Ce projet a été développé avec l'assistance de [Claude](https://claude.ai), l'IA d'Anthropic.