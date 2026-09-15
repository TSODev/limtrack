Voici un résumé complet pour Claude Code :

---

# LimTrack — Résumé projet pour Claude Code

## Présentation
Application web full-stack **entièrement en Rust** de gestion de flotte kilométrique. **SaaS ready** (web) et **Mobile ready** (PWA + Tauri iOS). Suivi contrats LOA/assurance, relevés kilométriques, alertes personnalisées, export PDF/CSV.

## Stack technique
- **Frontend** : Leptos 0.6 (WASM), Tailwind CSS v4, Trunk
- **Backend** : Axum 0.7, SQLx 0.8, PostgreSQL (auto-hébergé sur VPS OVH)
- **Auth** : JWT (jsonwebtoken) + bcrypt
- **Sécurité mots de passe** : `zxcvbn` (score ≥ 3/4) à l'inscription et au changement de mot de passe
- **Licences** : jetons SHA-256, middleware `402`, CLI `gen-tokens`, délivrance automatique via formulaire — **désactivé depuis la v1.4.0** (app gratuite pour tout le monde, code conservé)
- **Modèle** : open source AGPL v3, licences gratuites sur demande, dons Ko-fi / GitHub Sponsors
- **Mobile** : Tauri v2 (iOS configuré, Android à faire), PWA installable
- **Export** : PDF (contrats, flotte) et CSV (relevés avec trajectoire idéale, flotte) — génération 100% frontend (WASM, Blob API)
- **Types partagés** : crate `common` (workspace Cargo)
- **Déploiement** : Cloudflare Pages (frontend, GitHub Actions) + OVH VPS (backend + PostgreSQL)

## Architecture workspace
```
limtrack/
├── backend/src/
│   ├── main.rs
│   ├── auth.rs
│   ├── state.rs
│   ├── secrets.rs             ← chargement secrets via dotenvy (.env)
│   ├── notifier.rs            ← envoi notifications email expiration licence (Resend noreply@limtrack.app)
│   ├── handlers.rs            ← login, status, helpers généraux
│   ├── lib.rs                 ← expose notifier + secrets aux binaires CLI
│   ├── user_handler.rs
│   ├── vehicles_handler.rs
│   ├── contracts_handler.rs
│   ├── mileage_handler.rs
│   ├── trips_handler.rs       ← CRUD voyages planifiés + GET .../usage-forecast (projection km/jour)
│   ├── maintenance_handler.rs ← CRUD carnet d'entretien + GET .../maintenance-status (échéances estimées)
│   ├── attachments_handler.rs ← upload/téléchargement/suppression pièces jointes (stockage disque uploads/)
│   ├── share_handler.rs
│   ├── company_handler.rs     ← gestion flotte : entreprises, orgs, membres, rôles
│   ├── license_handler.rs     ← GET /api/profile/license + POST /api/profile/redeem
│   ├── license_middleware.rs  ← middleware 402 si licence expirée
│   ├── request_license_handler.rs ← POST /api/license/request (public, délivrance automatique)
│   ├── admin_handler.rs           ← /api/admin/* — dashboard admin (AdminUser extractor, is_admin requis)
│   ├── broadcast_handler.rs       ← GET /api/broadcasts/active — message ponctuel filtré selon is_ios
│   └── bin/
│       ├── gen_tokens.rs      ← CLI génération jetons (cargo run --bin gen-tokens)
│       ├── assign_license.rs  ← CLI assignation jetons manuel/batch CSV
│       ├── notify_expiry.rs   ← CLI déclenchement manuel notifications email
│       ├── send_broadcast.rs  ← CLI envoi broadcast (--message, --days, --exclude-ios)
│       └── CLI.md             ← documentation complète de tous les utilitaires
├── frontend/src/
│   ├── config.rs              ← API_BASE = "https://api.limtrack.app"
│   ├── build.rs               ← lit git describe --tags → APP_VERSION (fallback CARGO_PKG_VERSION)
│   ├── pages/
│   │   ├── home.rs
│   │   ├── login.rs
│   │   ├── register.rs
│   │   ├── mainpage.rs        ← signal `vehicles_loaded` + composant `OnboardingEmpty` (écran d'accueil si 0 véhicule) + `fetch_profile_flags` retourne `(is_admin, is_ios, is_trial_only)`
│   │   ├── fleet.rs           ← page gestion de flotte (admin entreprise)
│   │   ├── profile.rs
│   │   ├── about.rs           ← page À propos : version, description, contact mailto:, Ko-fi, GitHub Sponsors
│   │   ├── request_license.rs ← page /request-license : formulaire email → jeton gratuit 365j
│   │   └── admin.rs           ← page /admin : dashboard admin (stats, users, licences, flottes)
│   └── components/
│       ├── ui.rs              ← helpers partagés : input_class(), get_token(), format_km(), format_date_fr()
│       ├── vehicle.rs         ← VehicleCard component
│       ├── vehicle_dashboard.rs
│       ├── vehicle_detail.rs  ← détail véhicule avec VehicleWithAccess
│       ├── vehicle_header.rs
│       ├── vehicle_list.rs
│       ├── notification_bell.rs
│       ├── add_vehicle_button.rs
│       ├── join_vehicle_button.rs
│       ├── contracts/
│       │   ├── contract_list.rs
│       │   └── contract_widget.rs
│       ├── mileage/
│       │   ├── mileage_list.rs
│       │   └── mileage_widget.rs      ← trajectoire idéale + overlay projection (usage-forecast)
│       ├── trips/
│       │   ├── trip_list.rs           ← CRUD voyages + TripModal (récurrence)
│       │   └── trip_widget.rs         ← widget dashboard "Capacité kilométrique" (jour/semaine/mois)
│       └── maintenance/
│           ├── catalog.rs             ← catalogue générique statique (thermique/électrique)
│           ├── maintenance_list.rs    ← CRUD types + journal d'interventions
│           └── maintenance_widget.rs  ← widget dashboard échéance la plus urgente
├── frontend/src-tauri/        ← Tauri iOS
│   ├── tauri.conf.json
│   ├── gen/apple/             ← Projet Xcode généré
│   └── icons/                 ← Icônes toutes tailles
├── common/src/lib.rs
├── Cargo.toml                 ← version = "1.4.0"
├── docs/
│   └── appstore-screenshots.md  ← guide screenshots App Store (credentials, checklist, tailles)
├── sql/
│   ├── migrations/            ← SQL à appliquer manuellement sur PostgreSQL VPS
│   ├── schema/                ← Définition initiale des tables (neon_tables.sql)
│   └── seed/
│       ├── seed_fleet_demo.sql        ← données flotte (alice.martin / FleetAdmin2024!)
│       ├── seed_appstore_review.sql   ← compte App Store (apple.reviewer / AppReview2024!)
│       ├── import_seed.sh
│       └── import_appstore_review.sh
├── .github/workflows/
│   └── deploy-frontend.yml    ← CI/CD : build Leptos/WASM + deploy Cloudflare Pages
├── api/
│   └── limtrack-collection.postman_collection.json  ← Collection Postman
└── Trunk.toml
```

## URLs production
- Frontend : `https://limtrack.app` (Cloudflare Pages)
- Backend : `https://api.limtrack.app` (OVH VPS `164.132.40.109`)
- BDD : PostgreSQL auto-hébergé sur VPS (Docker, volume persistant)

## Base de données
```sql
users                  -- Auth JWT + bcrypt + trial_ends_at + access_expires_at
vehicles               -- owner_id, make, model, plate_number, company_id
vehicle_access         -- rôles : owner, editor, viewer (ON DELETE CASCADE)
contracts_loa          -- ON DELETE CASCADE
contracts_insurance    -- ON DELETE CASCADE
mileage_log            -- ON DELETE CASCADE
vehicle_share_codes    -- codes XXX-XXX-XXX (ON DELETE CASCADE)
user_preferences       -- notif_days_before, notif_km_percent
companies              -- name, siret, created_by
organizations          -- company_id, parent_org_id, name (hiérarchie)
company_members        -- user_id, company_id
fleet_roles            -- user_id, company_id, org_id, role, granted_by
license_tokens         -- token_hash (SHA-256), duration_days, used_at, used_by
license_requests       -- email (UNIQUE), token_hash, requested_at — anti-doublon formulaire public
planned_trips          -- vehicle_id, label, estimated_km, start_date, end_date, recurrence
                       -- (none/daily/weekly/monthly), recurrence_interval, days_of_week SMALLINT[],
                       -- day_of_month, recurrence_end_date, active (ON DELETE CASCADE)
maintenance_types      -- vehicle_id, label, interval_km, interval_months, active (ON DELETE CASCADE)
maintenance_entries    -- vehicle_id, label (snapshot/libre), performed_at, km_at_service, cost, provider, notes
-- vehicles.fuel_type TEXT NULL ('thermique'|'electrique'|'hybride') — migration 016, filtre le catalogue générique d'entretien
maintenance_attachments -- entry_id (ON DELETE CASCADE), vehicle_id, file_path, original_filename, content_type, size_bytes -- migration 017
maintenance_entry_types -- entry_id + maintenance_type_id (ON DELETE CASCADE sur les deux, PK composite) — migration 018,
                       -- relation many-to-many : une entrée (une facture, un jeu de photos, un coût) peut couvrir plusieurs
                       -- types (ex. révision = vidange + filtres). Remplace l'ancienne colonne mono-type
                       -- maintenance_entries.maintenance_type_id (supprimée par cette migration).
-- users.is_admin BOOLEAN DEFAULT FALSE — migration 005, accès dashboard admin
-- contracts_loa.price_per_extra_km FLOAT NULL — migration 006, coût dépassement km
-- users.is_ios BOOLEAN DEFAULT FALSE — migration 007, version Personal iOS (sans flotte)
-- users.password_reset_token TEXT NULL — migration 008, hash SHA-256 du token de reset
-- users.password_reset_expires_at TIMESTAMPTZ NULL — migration 008, expiry 1h
-- vehicles.archived_at TIMESTAMPTZ NULL — migration 009, archivage fin de LOA
-- broadcasts (id, message, created_at, expires_at, exclude_ios) — migration 010, messages broadcast admin
-- contracts_insurance.auto_renew BOOLEAN NOT NULL DEFAULT FALSE — migration 011, renouvellement automatique J-7
-- VIEW v_contract_status (vehicle_id, status) — migration 012, calcul danger/warning/ok centralisé (utilisé via LEFT JOIN dans vehicles_handler.rs)
-- users.license_type TEXT NOT NULL DEFAULT 'personal' — migration 013, type de licence centralisé sur users (backfill depuis dernier jeton, éditable via PATCH /api/admin/users/:id)
-- planned_trips (voir ci-dessus) — migration 014, voyages planifiés (ponctuels/récurrents) pour la projection d'usage futur
-- maintenance_types / maintenance_entries (voir ci-dessus) — migration 015, carnet d'entretien
-- maintenance_entry_types (voir ci-dessus) — migration 018, entretien multi-points (une entrée ↔ plusieurs types)
```

## Routes API
```
# Monitoring (public, hors middleware)
GET         /health                                                ← SELECT 1 sur DB → 200 "ok" / 503 "db_error" — Kuma : http://backend:3000/health

# Auth & Profil (public sauf mention)
POST        /login                                             ← email ou username
POST        /api/user/register
POST        /api/user/forgot-password                         ← public, token SHA-256, expiry 1h
POST        /api/user/reset-password                          ← public, valide token + maj bcrypt
GET/DELETE  /api/profile
POST        /api/profile/password
GET         /api/profile/shares
GET/PUT     /api/profile/preferences
GET         /api/profile/license
POST        /api/profile/redeem
POST        /api/license/request                              ← public, jeton 365j gratuit (renouvellement si précédent utilisé)
POST        /api/ios/activate                                 ← public, activation iOS App Store

# Véhicules
GET/POST    /api/vehicles                                         ← filtre archived_at IS NULL
GET         /api/vehicles/archived
GET/DELETE  /api/vehicles/:id
PATCH       /api/vehicles/:id/archive                            ← owner uniquement
PATCH       /api/vehicles/:id/unarchive                         ← owner uniquement
POST        /api/vehicles/:id/share
POST        /api/vehicles/join
DELETE      /api/vehicles/:id/access/:user_id
DELETE      /api/vehicles/:id/leave
GET/POST    /api/vehicles/:id/contracts/loa
PATCH/DELETE /api/vehicles/:id/contracts/loa/:contract_id
GET/POST    /api/vehicles/:id/contracts/insurance
PATCH/DELETE /api/vehicles/:id/contracts/insurance/:contract_id   ← PATCH : auto_renew uniquement
POST        /api/vehicles/:id/contracts/insurance/:contract_id/renew ← crée le contrat successeur
GET/POST    /api/vehicles/:id/mileage
DELETE      /api/vehicles/:id/mileage/:entry_id
POST/DELETE /api/vehicles/:id/fleet                           ← assigner/retirer d'une flotte

# Voyages planifiés
GET/POST    /api/vehicles/:id/trips
PATCH/DELETE /api/vehicles/:id/trips/:trip_id
GET         /api/vehicles/:id/usage-forecast                  ← km/jour disponible + date d'indisponibilité prévisible (voyages inclus)

# Carnet d'entretien
GET/POST    /api/vehicles/:id/maintenance-types
PATCH/DELETE /api/vehicles/:id/maintenance-types/:type_id
GET/POST    /api/vehicles/:id/maintenance-entries
DELETE      /api/vehicles/:id/maintenance-entries/:entry_id
GET         /api/vehicles/:id/maintenance-status               ← échéance estimée (km/date) + statut "en retard" par type actif
GET/POST    /api/vehicles/:id/maintenance-entries/:entry_id/attachments  ← POST = multipart, limite de corps dédiée 40 Mo
GET/DELETE  /api/vehicles/:id/attachments/:attachment_id       ← GET = octets du fichier + Content-Type

# Flotte
GET/POST    /api/companies
GET/DELETE  /api/companies/:id
GET/POST    /api/companies/:id/organizations
DELETE      /api/companies/:id/organizations/:oid
GET/POST    /api/companies/:id/members
DELETE      /api/companies/:id/members/:uid
GET/POST    /api/companies/:id/fleet-roles
DELETE      /api/companies/:id/fleet-roles/:role_id
GET         /api/companies/:id/vehicles
GET         /api/companies/:id/organizations/:oid/vehicles
GET         /api/companies/:id/fleet-report                   ← rapport PDF/CSV flotte

# Admin (is_admin = true requis)
GET         /api/admin/stats                                      ← total users/trial/active/expired/vehicles/license-requests
GET         /api/admin/users
PATCH       /api/admin/users/:id                                  ← édition admin : username, email, is_admin, is_ios, license_type, access_expires_at
GET         /api/admin/growth                                     ← croissance hebdomadaire users + véhicules sur 12 semaines
GET         /api/admin/license-requests
POST        /api/admin/generate-token
POST        /api/admin/assign-license                             ← assigne un jeton existant à un compte (email + token)
POST        /api/admin/notify-expiry                              ← déclenche manuellement les emails d'expiration (Resend)
POST        /api/admin/broadcasts                                 ← crée un broadcast (message, days, exclude_ios)
GET         /api/admin/companies

# Broadcasts
GET         /api/broadcasts/active                                ← message actif (filtré is_ios si exclude_ios)
```

## Licences — système de jetons (désactivé depuis v1.4.0)
> **App gratuite pour tout le monde** : `LICENSE_ENFORCEMENT_ENABLED = false` (`license_middleware.rs`) et `LICENSE_ENABLED = false` (`frontend/src/config.rs`) désactivent respectivement le 402 côté backend et toute l'UI licence côté frontend (profil, modal essai, à propos, cloche notif, FAQ, `/request-license`). Le code ci-dessous est **conservé intact** et réactivable en repassant les deux constantes à `true` (à garder synchronisées). La tâche de fond d'emails d'expiration (`notifier.rs`) est également désactivée tant que `LICENSE_ENFORCEMENT_ENABLED = false`.
- Période d'essai : `trial_ends_at = NOW() + 3 mois` à l'inscription
- Accès actif si `trial_ends_at > NOW() OR access_expires_at > NOW()`
- Routes exemptées du middleware : `/login`, `/api/user/register`, `/api/user/forgot-password`, `/api/user/reset-password`, `/api/profile/license`, `/api/profile/redeem`, `/api/license/request`, `/api/ios/activate`, `/api/admin/*`
- **Dashboard admin** : routes `/api/admin/*` protégées par `AdminUser` extractor (vérifie `users.is_admin = true`). Activer avec `UPDATE public.users SET is_admin = TRUE WHERE email = '...'`
- `AppState` contient `resend_api_key: String` (lu au démarrage via `load_secrets()` → dotenvy `.env`)
- **Mode lecture seule** : licence expirée → `GET` passe (lecture autorisée), `POST/PUT/DELETE/PATCH` → `402 Payment Required`
- Jetons : format `XXXX-XXXX-XXXX-XXXX`, SHA-256 stocké (jamais en clair), cumulables
- Durées disponibles : 30, 90, 180, 365 jours
- **Page d'inscription** : encadré info "Période d'essai gratuite — 3 mois" affiché avant le bouton de soumission ; message de succès rappelle la durée d'essai
- **Modal "période d'essai"** (`mainpage.rs`) : affiché une seule fois par navigateur (`limtrack_trial_notice_shown` en localStorage), **uniquement si `is_trial_only == true`** (vérifié via `GET /api/profile/license` → `status == "trial"`). Un compte avec une licence active ne voit jamais ce modal, même sur un nouveau navigateur. `fetch_profile_flags` retourne `(is_admin, is_ios, is_trial_only)` en un double fetch parallèle.
- **Délivrance automatique** : `POST /api/license/request` (public, sans auth) — email → jeton 365j généré et envoyé via Resend. Anti-doublon via table `license_requests` : une nouvelle demande est autorisée uniquement si le jeton précédent a déjà été utilisé (permettant le renouvellement annuel pour les LOA 3-4 ans). `RESEND_API_KEY` lu au démarrage via `AppState.resend_api_key`.

## iOS App Store — modèle payant
- **Version web (PWA)** : gratuite, licences sur demande, dons Ko-fi/GitHub Sponsors
- **Version App Store iOS** : payante (achat unique), accès lifetime inclus
- **Activation iOS** : `POST /api/ios/activate` — accordé au premier lancement Tauri, vérifié par `IOS_ACTIVATION_KEY` (variable d'env VPS). Idempotent. Stocké `ios_activated` en localStorage. En cas de succès : écrit aussi `limtrack_is_ios = "1"` en localStorage ET met à jour les signaux Leptos `set_is_ios_user(true)` / `set_show_trial_modal(false)` immédiatement — évite le flash du modal d'essai au premier lancement.
- **Détection Tauri** : `crate::config::is_tauri()` via `window.__TAURI__`. Fiable en production ; **peu fiable en dev Simulator** (ne pas s'y fier pour masquer du contenu).
- **Détection compte iOS** : champ `users.is_ios` (migration 007) — source de vérité pour masquer Licence/Flotte dans le profil, les sections web-only dans À propos, et l'alerte d'expiration de licence dans la notification bell. Stocké dans `localStorage["limtrack_is_ios"]` dès le chargement de mainpage pour éviter le flash au rendu.
- **Détection contexte Tauri** : `crate::config::is_tauri()` — utilisé à l'inscription (`register.rs`) pour masquer la notice et le message "3 mois d'essai" non pertinents sur iOS.
- **Clé iOS** : `IOS_ACTIVATION_KEY` injectée à la compilation (`option_env!`) — à définir en variable d'env lors du build Tauri iOS.
- **Conformité AGPL v3** : exception App Store ajoutée dans `licence.md` (Thierry Soulie, détenteur unique).
- **Privacy Policy** : page `/privacy` hébergée sur `limtrack.app/privacy` (obligatoire App Store).
- **Pages iOS-only (`/privacy`, `/support`)** : navbar sans lien "Accueil" — bouton "Fermer ✕" uniquement. En contexte Tauri : `window.__TAURI_INTERNALS__.invoke('exit', {exitCode:0})`. En Safari (lien App Store) : `window.close()` (best-effort, Safari l'autorise seulement si la fenêtre a été ouverte par script). Ces pages ne doivent jamais permettre de naviguer vers l'app web.
- **Règle Apple 3.1.1** : liens de dons masqués pour les comptes `is_ios = true` (Ko-fi/GitHub Sponsors interdits sur iOS).
- **Compte review App Store** : `apple.reviewer / AppReview2024!` (seed `seed_appstore_review.sql`). Voir `docs/appstore-screenshots.md`.

```bash
# Générer des jetons (depuis backend/)
cargo run --bin gen-tokens -- --count 5 --days 30
cargo run --bin gen-tokens -- --count 1 --days 365 --fleet
cargo run --bin gen-tokens -- --count 1 --lifetime --fleet

# Assigner un jeton à un utilisateur
cargo run --bin assign-license -- --email user@example.com --token XXXX-XXXX-XXXX-XXXX
cargo run --bin assign-license -- --file batch.csv   # CSV: email,token

# Notifications email manuelles
cargo run --bin notify-expiry

# Broadcast message à tous les utilisateurs
cargo run --bin send-broadcast -- --message "Texte du message"
cargo run --bin send-broadcast -- --message "Texte" --days 7          # expire dans 7 jours
cargo run --bin send-broadcast -- --message "Dons Ko-fi" --exclude-ios  # masqué sur iOS (règle 3.1.1)

# Aide sur n'importe quel CLI
cargo run --bin gen-tokens -- --help
cargo run --bin assign-license -- --help
cargo run --bin notify-expiry -- --help
cargo run --bin send-broadcast -- --help
```

## Accès véhicules — `vehicle_access`

**Source de vérité unique** : tous les handlers (véhicules, kilométrage, contrats, voyages, entretien, pièces jointes) vérifient l'accès exclusivement via `SELECT role FROM vehicle_access WHERE vehicle_id = $1 AND user_id = $2` — jamais via `vehicles.owner_id` directement. `list_vehicles` fait un `JOIN` (pas un `LEFT JOIN`) dessus : sans ligne `vehicle_access`, un véhicule est invisible à son propre propriétaire, y compris dans sa propre liste.

**Bug corrigé (migration 019, 2026-09-15)** : `create_vehicle` n'a jamais inséré cette ligne depuis son tout premier commit — seul `share_handler.rs::join_vehicle` insère dans `vehicle_access` (rejoindre via code de partage). Conséquence en production : tout véhicule créé via `POST /api/vehicles` (hors seeds SQL, qui insèrent `vehicle_access` à la main) était orphelin — invisible dans la liste, inutilisable pour kilométrage/contrats/entretien/voyages — et la limite `MAX_VEHICLES_PER_USER = 10` (comptée via `JOIN vehicle_access WHERE role = 'owner'`) n'était jamais atteinte puisque ce compteur était toujours à 0.
- **Fix** : `create_vehicle` insère désormais `vehicles` + `vehicle_access (role='owner')` dans la même transaction (`state.db.begin()`), avant le seed best-effort des types d'entretien par défaut (qui reste hors transaction, non bloquant).
- **Backfill** : migration `019` — `INSERT ... SELECT ... WHERE NOT EXISTS (...)`, idempotente, comble la ligne manquante pour tout véhicule existant déjà en base (à appliquer manuellement sur le VPS comme les autres migrations).
- **Piège à ne pas réintroduire** : toute nouvelle route qui insère dans `vehicles` (import, duplication, etc.) doit insérer `vehicle_access (role='owner')` dans la même transaction — ne pas se fier à un trigger DB, il n'en existe aucun (vérifié : `SELECT * FROM pg_trigger WHERE NOT tgisinternal` ne renvoie rien).

## Sécurité — protections anti-flood et limites métier

### Rate limiting — `tower_governor`
Crate `tower_governor = { version = "0.4", features = ["axum"] }`, `SmartIpKeyExtractor` (lit `X-Forwarded-For` / Cloudflare en priorité, retombe sur `ConnectInfo<SocketAddr>` sinon). Sous-routeur `sensitive_public` limité à **1 req/s, burst 5** :
`/login`, `/api/user/register`, `/api/user/forgot-password`, `/api/user/reset-password`, `/api/license/request`

**Important** : `main.rs` doit servir l'app via `root.into_make_service_with_connect_info::<SocketAddr>()` (pas `axum::serve(listener, root)` seul), sinon `SmartIpKeyExtractor` ne peut jamais retomber sur l'IP de connexion réelle quand `X-Forwarded-For` est absent → 500 "Unable To Extract Key!" (touche tout accès direct au VPS sans passer par Cloudflare, ou tout test local).

### Taille du corps
`DefaultBodyLimit::max(64 * 1024)` sur toutes les routes — bloque les requêtes > 64 Ko.

### Validation longueur des champs
| Champ | Limite | Raison |
|-------|--------|--------|
| make, model | 100 car. | champs libres véhicule |
| plate_number | 20 car. | format plaque |
| vin | 17 car. | norme ISO 3779 |
| username | 50 car. | identifiant utilisateur |
| email | 254 car. | RFC 5321 |
| password | 1 000 car. | protection DoS bcrypt (hachage coûteux) |
| insurer | 200 car. | nom assureur libre |

### Limites métier
- Max **10 véhicules actifs** par propriétaire (archivage pour libérer un slot)
- Max **5 contrats LOA** par véhicule
- Max **5 contrats Assurance** par véhicule
- Max **5 relevés kilométriques par jour** par véhicule
- Max **1 500 km/jour de taux** entre deux relevés consécutifs (`km_diff / jours_entre ≤ 1500`)
- **Unicité des périodes LOA** : `start < new_end AND end > new_start` → 409 Conflict
- **Unicité des périodes Assurance** : même logique

## Sécurité — vérification des mots de passe
Crate `zxcvbn` utilisée dans `user_handler.rs`. Score minimum **3/4** requis.
```rust
use zxcvbn::zxcvbn;

fn check_password_strength(password: &str, user_inputs: &[&str]) -> Result<(), String> {
    let estimate = zxcvbn(password, user_inputs);
    if u8::from(estimate.score()) < 3 {
        let msg = estimate.feedback().as_ref()
            .and_then(|f| f.warning())
            .map(|w| w.to_string())
            .unwrap_or_else(|| "Mot de passe trop faible.".to_string());
        return Err(msg);
    }
    Ok(())
}
```
Appelé dans `register` avec `&[username, email]` et dans `change_password` avec les données récupérées en BDD.

## Contrats Assurance — renouvellement automatique

### Champ `auto_renew`
Migration `011` : `ALTER TABLE public.contracts_insurance ADD COLUMN auto_renew BOOLEAN NOT NULL DEFAULT FALSE`.
Dans `common/src/lib.rs` : `ContractInsurance { pub auto_renew: bool }` et `CreateInsurancePayload { #[serde(default)] pub auto_renew: Option<bool> }`.

### Tâche de fond (`main.rs`)
Lancée dans `tokio::spawn` au démarrage, se déclenche chaque jour à 8h UTC. Appelle `contracts_handler::run_insurance_renewals(&db)` qui cherche les contrats avec `auto_renew = true AND end_date <= today + 7 jours AND pas de successeur` et crée le contrat suivant via `do_renew()`.

### `do_renew` (interne à `contracts_handler.rs`)
```rust
// new_start = old.end_date
// new_end   = old_end + signed_duration_since(old_start)  ← même durée
// km_start  = dernier relevé kilométrique (ou 0)
// auto_renew = true sur le nouveau contrat
```

### Routes
- `PATCH /api/vehicles/:id/contracts/insurance/:cid` — payload `{ "auto_renew": bool }`, `COALESCE` en SQL
- `POST  /api/vehicles/:id/contracts/insurance/:cid/renew` — crée immédiatement le successeur ; renvoie `409` si un contrat avec `start_date = old.end_date` existe déjà

### Frontend
- **Onglet Contrats (`contract_list.rs`)** : toggle + bouton "Renouveler maintenant →" dans `ContractInsuranceCard` (owner/editor uniquement)
  - `on_updated: Callback<()>` déclenche un rechargement de la liste après toggle ou renouvellement
  - Mise à jour optimiste du signal `auto_renew` local + PATCH
  - Bouton renouvellement POST /renew + message d'erreur JSON (ex. 409)
  - `patch_json` helper + `parse_error_response` (lit `{"error": "..."}` avant "Erreur HTTP : N")
  - `InsuranceModal` : checkbox auto_renew à la création
- **Dashboard (`contract_widget.rs`)** : `ContractInsuranceSummary` en lecture seule — badge ↻ statique si `auto_renew = true`, aucune action

## Voyages planifiés — planification & projection d'usage

### Modèle (`common::PlannedTrip`, migration 014)
`recurrence` : `"none"` (ponctuel) / `"daily"` / `"weekly"` (+ `days_of_week: Vec<i16>`, 0=lundi..6=dimanche) / `"monthly"` (+ `day_of_month: i16`, clampé au dernier jour du mois). `recurrence_end_date` optionnel (`None` = expansion jusqu'à la fin du contrat actif). Écriture réservée owner|editor (`require_editor`, comme `mileage_handler.rs`), max `MAX_TRIPS_PER_VEHICLE = 20`.

**Limite connue** : `recurrence_end_date` ne peut pas être explicitement effacée via `PATCH` (un `Option<T>` ne distingue pas "champ absent" de "`null`" côté serde) — supprimer/recréer le voyage pour repasser en récurrence sans date de fin.

### Expansion des occurrences (`trips_handler.rs::expand_occurrences`)
Fonction pure, calculée **à la demande** (pas de job cron) — répartit `estimated_km` uniformément sur les jours de chaque occurrence pour éviter les pics verticaux dans la projection. Horizon plafonné au plus tôt de : fin du contrat actif, `recurrence_end_date`, ou 3 ans (garde-fou anti-boucle infinie, `MAX_OCCURRENCES_GUARD`).

### `GET /api/vehicles/:id/usage-forecast`
Combine `daily_rate = km_consumed / days_elapsed` (même formule que `estimate_limit_date` dans `contracts_handler.rs`) avec les voyages planifiés actifs pour projeter jour par jour l'usage cumulé jusqu'à `end_date`. Retourne `km_per_day_available` (peut être négatif = dépassement déjà prévisible), `unavailable_from`/`unavailable_days` (première date où le cumul atteindrait le plafond), et `points` (échantillonnage hebdomadaire pour le graphique). Si LOA **et** assurance sont actifs simultanément, calcule les deux et retourne le plus restrictif (date d'indisponibilité la plus proche).

**Distinction importante avec `contracts_handler.rs`** : `ContractLoa/Insurance.estimated_limit_date` (widget "Contrat actif") est calculé **sans** les voyages planifiés (rythme historique seul) — c'est volontaire, pour ne pas modifier la logique de risque existante (badges, `v_contract_status`). Les deux dates peuvent donc légitimement différer ; les libellés frontend précisent "(rythme actuel)" vs "(voyages inclus)" pour éviter la confusion.

**Piège corrigé** : `km_allowed`/`km_annual_limit` sont des valeurs **relatives** (delta depuis `km_start`), pas des compteurs absolus. Le seuil de `unavailable_from` doit comparer `km_consumed + cumulative >= km_allowed` (tout en relatif) — ne jamais réintroduire `km_start` dans cette comparaison (bug corrigé qui déclenchait une fausse indisponibilité dès le jour 1 pour tout véhicule avec `km_start` non nul ; invisible en test local avec `km_start = 0`, donc **toujours tester avec un `km_start` réaliste non nul**).

### Frontend
- Onglet "Voyages" dans `vehicle_dashboard.rs` (`DashboardTab::Trips`), `can_manage_trips` = owner|editor (comme `can_edit`, pas `can_manage_contracts` qui est owner-only)
- `trip_list.rs` : CRUD + `TripModal` (récurrence avec champs conditionnels — jours de semaine si hebdo, jour du mois si mensuel). `Modal`/`Field`/`ModalActions` dupliqués localement (convention du projet — déjà dupliqués dans `contract_widget.rs`/`contract_list.rs`, pas de composants partagés)
- `trip_widget.rs` : widget dashboard "Capacité kilométrique" — 3 tuiles (par jour / semaine / mois) + badge "Indisponible à partir du [date] (voyages inclus)" si applicable
- `mileage_widget.rs` : trajectoire idéale prolongée jusqu'à `end_date` (au lieu de s'arrêter à aujourd'hui) + overlay pointillé violet de la projection avec voyages (`GET .../usage-forecast`)

## Carnet d'entretien

### Modèle (migration 015, multi-points depuis migration 018)
Deux tables : `maintenance_types` (définition récurrente — label + `interval_km` et/ou `interval_months`, au moins un des deux requis) et `maintenance_entries` (journal — une intervention réelle, avec `label` **copié/snapshot** ou libre). Une entrée reste un événement unique (une facture, un jeu de photos, un coût) mais peut être rattachée à **plusieurs** types via la table de jointure `maintenance_entry_types` (ex. une révision = vidange + filtre à air + filtre à huile en une seule entrée) — `ON DELETE CASCADE` des deux côtés : supprimer un type ne retire que le rattachement (la ligne du pivot), jamais l'entrée ni son historique.

**Résolution du label côté backend** (`maintenance_handler.rs::create_maintenance_entry`) : le payload `CreateMaintenanceEntryPayload.maintenance_type_ids: Vec<Uuid>` peut être vide (entrée "Autre" libre, `label` alors requis) ou contenir 1..`MAX_TYPES_PER_ENTRY` (10) ids. Si `label` est fourni, il prime toujours (override utilisateur, ex. "Révision 30 000 km") ; sinon il est généré en joignant les labels des types sélectionnés avec `", "`. L'insertion (entry + lignes du pivot) est faite dans une transaction (`state.db.begin()`).

**`GET .../maintenance-entries`** retourne `type_ids: Vec<Uuid>` par entrée via `ARRAY_AGG(...) FILTER (...)` + `LEFT JOIN maintenance_entry_types` + `GROUP BY e.id` (`COALESCE(..., '{}')` pour éviter `NULL` quand l'entrée n'a aucun type rattaché).

**`maintenance-status`** : la recherche de la dernière entrée par type passe par un `JOIN maintenance_entry_types` (au lieu d'un filtre direct sur une colonne) — une entrée multi-types met donc à jour l'échéance de **chacun** des types qu'elle couvre.

**Seed par défaut** (`vehicles_handler.rs::create_vehicle`) : à la création d'un véhicule, deux types sont insérés automatiquement (best-effort, ne bloque pas la création si l'insert échoue) — "Vidange" (15 000 km / 12 mois) et "Contrôle technique" (24 mois). Éditables/supprimables ensuite normalement.

**Pas d'API constructeur** : aucune API publique/gratuite n'existe pour les recommandations d'entretien OEM (les offres commerciales type Vehicle Databases/CarScan/MOTOR sont centrées marché US, couverture Europe faible) — à la place, un **catalogue générique statique** embarqué dans le frontend (`frontend/src/components/maintenance/catalog.rs::GENERIC_CATALOG`) propose des valeurs indicatives éditables, filtrées par motorisation.

### Motorisation du véhicule (migration 016)
`vehicles.fuel_type` (`TEXT NULL`, `thermique`/`electrique`/`hybride`) — éditable **uniquement à la création** du véhicule (`add_vehicle_button.rs`) : il n'existe pas de formulaire d'édition véhicule dans le frontend aujourd'hui (`update_vehicle` dans `vehicles_handler.rs` existe côté backend mais n'est appelé par aucune page — warning de compilation "never used" à ne pas confondre avec du code mort à supprimer). Les véhicules existants restent `fuel_type = NULL` : c'est le cas de repli explicitement voulu, pas un bug — le catalogue générique affiche alors tous les items avec une étiquette (⛽/🔋) au lieu de filtrer.

### Catalogue générique (`components/maintenance/catalog.rs`)
`GENERIC_CATALOG: &[GenericTemplate]` — liste statique (label, `fuel_type: Option<&str>` où `None` = commun aux deux motorisations, `interval_km`, `interval_months`). `GenericTemplate::is_periodic()` = au moins un intervalle défini. Proposé en **cases à cocher** (multi-sélection) dans `EntryModal` (`maintenance_list.rs`), groupées par catégorie, en plus des types déjà créés pour le véhicule :
- Clés préfixées pour lever l'ambiguïté : `type:<uuid>` (type existant), `generic:<index>` (item du catalogue). Sélection maintenue dans un `Vec<String>` (pas un `HashSet`) pour préserver l'ordre de coche et générer un libellé auto reproductible.
- Dédoublonnage : un item générique déjà instancié (label identique, insensible à la casse, à un type existant du véhicule) disparaît de la liste.
- Filtrage : si `vehicle_fuel_type` est renseigné, seuls les items `None` ou de la même motorisation sont proposés (sans étiquette) ; sinon tous les items sont montrés avec suffixe " · ⛽ thermique"/" · 🔋 électrique".
- Un champ "Libellé" texte est toujours visible (pas seulement pour "Autre") — `prop:required` dynamique : requis uniquement si aucune case n'est cochée. Rempli par l'utilisateur → override envoyé tel quel ; vide → le frontend calcule lui-même le libellé auto (join des labels résolus, types existants + génériques, avec `", "`) plutôt que de compter sur la génération côté backend, pour couvrir le cas des items génériques **ponctuels** cochés (qui ne créent pas de type, donc absents de tout `maintenance_type_ids` renvoyé par le backend).
- À la soumission, pour chaque item coché : `type:<uuid>` → ajouté tel quel à `maintenance_type_ids` ; `generic:<index>` **périodique** → `POST .../maintenance-types` (instancie le template comme type réutilisable) **puis** son id ajouté à `maintenance_type_ids` ; `generic:<index>` **ponctuel** (aucun intervalle, ex. Pneus) → contribue seulement son label au libellé auto, jamais transformé en type ni ajouté à `maintenance_type_ids`.

### `GET /api/vehicles/:id/maintenance-status`
Pour chaque type **actif**, cherche sa dernière entrée (`ORDER BY performed_at DESC, created_at DESC LIMIT 1`). Sans entrée → `last_performed_at: null`, pas d'échéance calculable. Avec entrée :
- `next_due_km = last.km_at_service + interval_km` (si `interval_km` défini)
- `next_due_date` = la plus proche entre l'échéance par temps (`last.performed_at + interval_months`) et l'échéance par km, cette dernière projetée via `estimate_date_for_km` — **rythme moyen calculé sur tout l'historique `mileage_log` du véhicule** (premier → dernier relevé), volontairement **indépendant** de `estimate_limit_date` (`contracts_handler.rs`) qui lui est borné à un contrat.
- `overdue` = km actuel du véhicule ≥ `next_due_km`, OU date du jour ≥ échéance par temps.

**Piège à ne pas réintroduire** (cf. bug `usage-forecast` du 2026-09-14) : toutes les comparaisons ici sont déjà en valeurs absolues cohérentes (`km_at_service` est un compteur absolu, pas un delta comme `km_start`/`km_allowed` des contrats) — ne pas mélanger les deux conventions si du code est partagé/réutilisé entre les deux features à l'avenir.

### Frontend
- Onglet "Entretien" dans `vehicle_dashboard.rs` (`DashboardTab::Maintenance`), `can_manage_maintenance` = owner|editor.
- `maintenance_list.rs` : section "Types" (badge À jour/Bientôt/En retard/Jamais fait calculé côté client depuis `maintenance-status`) + section "Historique" (tableau chronologique). `TypeModal`/`EntryModal`/`Modal`/`Field`/`ModalActions` dupliqués localement (convention du projet).
- `maintenance_widget.rs` : résumé dashboard de l'échéance la plus urgente (en retard prioritaire, sinon date la plus proche).

### Pièces jointes (migration 017)
Table `maintenance_attachments` (`entry_id` FK `ON DELETE CASCADE`, `vehicle_id` dénormalisé). **Le fichier sur disque n'est jamais nettoyé par la CASCADE SQL** — `attachments_handler.rs::delete_attachment` et `maintenance_handler.rs::delete_maintenance_entry` font l'unlink explicitement (best-effort, log si échec).

- Stockage : `uploads/{vehicle_id}/{entry_id}/{uuid}.{ext}` (nom généré, jamais le nom original — évite path traversal). **`docker-compose.yml`** monte `./uploads:/app/uploads` sur le service `backend` — répertoire à créer une fois sur le VPS (`mkdir -p /opt/limtrack/uploads`) avant tout déploiement touchant l'infra, sinon les fichiers sont perdus au prochain redéploiement (conteneur recréé à chaque push).
- `POST /api/vehicles/:id/maintenance-entries/:entry_id/attachments` — `axum::extract::Multipart` (feature `"multipart"` sur `axum` dans `Cargo.toml`). Limites : `MAX_ATTACHMENTS_PER_ENTRY = 5`, `MAX_FILE_SIZE = 8 Mo`, types autorisés `image/jpeg|png|webp`, `application/pdf`.
- **Limite de corps dédiée** : cette route vit sur un `Router` imbriqué séparé avec son propre `DefaultBodyLimit::max(40 Mo)`, mergé dans `app` — un layer posé sur un router imbriqué prime sur celui du router englobant pour cette sous-arborescence, permettant de garder `DefaultBodyLimit::max(64 * 1024)` global pour le reste de l'API. Voir `main.rs` (`uploads_router`).
- `GET /api/vehicles/:id/attachments/:attachment_id` renvoie les octets du fichier avec le bon `Content-Type` — jamais de `file_path` exposé au frontend (`common::MaintenanceAttachment`).
- Frontend : `EntryModal` — `<input type="file" accept="..." capture="environment" multiple>` (déclenche l'appareil photo sur mobile sans plugin natif), **caché** (`class="hidden"`) et déclenché par un vrai bouton stylé (cohérent avec le reste de l'app) via `NodeRef` + `set_timeout` (voir piège Leptos ci-dessus — le style natif du bouton de sélection de fichier via les classes Tailwind `file:*` était trop discret/inconsistant selon les navigateurs, remplacé par un bouton explicite). Upload en 2ᵉ requête après création de l'entrée (JSON) via `FormData` + fetch brut (pas de helper `api_client.rs`, multipart).
- **Visualisation** : `fetch_attachment_object_url()` — fetch authentifié (`Authorization: Bearer`, un `<a href>` classique n'enverrait pas le token) + `Blob` + `URL.createObjectURL`, affiché dans `ViewerModal` **intégré à l'app** (image via `<img>`, PDF via `<iframe>`, fallback "Télécharger" pour les autres types) — jamais via `window.open(url, "_blank")`. **Piège corrigé (2026-09-15)** : sur mobile, notamment en PWA installée (mode standalone), `window.open(..., "_blank")` navigue souvent dans la **même fenêtre** au lieu d'ouvrir un nouvel onglet — fermer la vue résultante fermait alors l'application entière (aucune page app à laquelle revenir). `URL.revokeObjectURL()` appelé à la fermeture de `ViewerModal`.

## Points importants Leptos
```rust
// Callbacks — toujours Callback<T>
on_saved: Callback<UserPreferences>
on_saved.call(value)

// Strings movées deux fois → cloner
let name = v.name.clone();
let name_del = name.clone();

// Memo<bool> pour pending()
let is_pending = create_memo(move |_| action.pending().get());

// PartialEq requis pour create_memo sur structs custom
#[derive(Clone, PartialEq)]
pub struct Vehicle { ... }

// Déclencher .click() sur un <input type="file"> caché depuis un bouton stylé :
// TOUJOURS différer via set_timeout, jamais appeler .click() en synchrone dans le
// on:click qui a déclenché l'événement — sinon panique wasm-bindgen "closure invoked
// recursively or after being dropped" (réentrance dans le closure d'event delegation
// de Leptos, encore sur la pile d'appel pour CE click). Repéré via test Chromium
// headless (Playwright) — invisible en dev normal car l'erreur n'interrompt que ce
// clic précis, pas tout le reste de l'app.
let file_input_ref = create_node_ref::<html::Input>();
on:click=move |_| {
    set_timeout(move || {
        if let Some(input) = file_input_ref.get() { input.click(); }
    }, std::time::Duration::ZERO);
}
```

## API_BASE — pattern fetch
Toutes les URLs API utilisent `crate::config::API_BASE` :
```rust
// String simple
let url = format!("{}/api/vehicles", crate::config::API_BASE);

// Dans appels de fonctions
fetch_json::<T>(&format!("{}/api/profile", crate::config::API_BASE), &token)
```

## Responsive mobile-first
- **mainpage.rs** : `hidden md:flex` (desktop) + `flex md:hidden` (mobile)
- Bottom sheet mobile pour sélection véhicule
- Safe areas iOS : CSS variable `--nav-top` définie dans `index.html`. En contexte Tauri (`tauri-ios` class), `max(env(safe-area-inset-top), 44px)` pour couvrir Dynamic Island. Usage : `style="padding-top: var(--nav-top)"` sur tous les `<nav>`.
- `overscroll-behavior-y: none` sur `body` (index.html) — bloque le rubber-band iOS
- **Onboarding** : si `vehicles_loaded && vehicles.is_empty()`, composant `OnboardingEmpty` remplace `VehicleDashboard` (desktop et mobile). La pill "Sélectionner un véhicule" est masquée pendant cet état (enveloppée dans `<Show when=!(loaded && empty)>`).
- Bottom sheet : boutons Ajouter/Rejoindre en `flex-row` hors du container scrollable + spacer `height: env(safe-area-inset-bottom)` en bas du panneau
- Notification bell : position panneau `top: calc(var(--nav-top) + 3.5rem)` + bouton ✕ explicite
- Boutons icônes seuls mobile : `hidden md:inline` sur les textes
- **Scroll tactile dans un modal `position:fixed`** : signalé bloqué (aucun scroll du tout) sur une PWA Android installée (Samsung, mode standalone) — non reproduit malgré test avec geste tactile simulé (Chromium headless, émulation Galaxy S24, `Input.dispatchTouchEvent` via CDP), qui fonctionne. Fix défensif appliqué en attendant confirmation terrain : `overscroll-contain touch-pan-y` sur le conteneur scrollable (`Modal` dans `maintenance_list.rs`, `ViewerModal`) — pattern standard pour ce symptôme en `position:fixed`, sans effet de bord si ce n'était pas la cause réelle. **Non confirmé comme correctif définitif** — à revalider avec l'utilisateur sur son appareil.

## Tauri iOS — lancer le Simulator
```bash
# Terminal 1 — serveur statique
cd frontend && trunk build --release
python3 -c "
import http.server, socketserver, os
class H(http.server.SimpleHTTPRequestHandler):
    def guess_type(self, p):
        return 'application/wasm' if str(p).endswith('.wasm') else super().guess_type(p)
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy','same-origin')
        self.send_header('Cross-Origin-Embedder-Policy','require-corp')
        super().end_headers()
    def log_message(self, *a): pass
os.chdir('dist')
with socketserver.TCPServer(('',1430),H) as s: s.serve_forever()
" &

# Terminal 2 — Tauri (toujours via cargo tauri ios dev, jamais Product→Build dans Xcode)
# Tests de développement (iPhone 13 Pro, 6.1")
cargo tauri ios dev "77F8FC35-195B-4C78-9690-28CF71ECDE54" --no-dev-server-wait

# Screenshots iPhone App Store (iPhone 13 Pro Max, 1284×2778 — taille OBLIGATOIRE)
cargo tauri ios dev "F50045E7-028E-485C-912C-C35154674374" --no-dev-server-wait

# Screenshots iPad App Store (iPad Pro 13" iOS 18.1 — iOS 26 crashe le WASM)
cargo tauri ios dev "85787740-ADB2-476B-9AA8-AD31B6EF8D21" --no-dev-server-wait
# Puis ▶ Run dans Xcode — screenshot : Cmd+S dans le Simulator
```

## Tauri iOS — build App Store
```bash
# IMPORTANT : ne jamais archiver depuis Xcode (Product→Archive) — le pre-build script
# Xcode doit se connecter au WebSocket démarré par cargo tauri ios build.

cd frontend/src-tauri
cargo tauri ios build
# IPA généré : gen/apple/build/arm64/LimTrack.ipa

# Upload via Transporter (app Apple, Mac App Store)
# Glisser-déposer le .ipa → Deliver
```

## Tauri iOS — pièges connus
- **Ne jamais builder depuis Xcode directement** : le pre-build script cherche le WebSocket de `cargo tauri ios build/dev`, sinon "Connection refused" (code 61)
- **iOS 26 (beta)** : crashe le WKWebView/WASM — utiliser iOS 18.1 pour les simulateurs
- **Icônes sans alpha** : App Store Connect rejette les PNG avec canal alpha. Supprimer via conversion JPEG intermédiaire : `sips -s format jpeg icon.png --out /tmp/t.jpg && sips -s format png /tmp/t.jpg --out icon_flat.png`
- **Régénérer les icônes** : `cargo tauri icon /chemin/icone-1024x1024.png`
- **Chiffrement** : `ITSAppUsesNonExemptEncryption = false` dans `project.yml` → Info.plist — exempte de documentation ANSSI (France) et EAR (USA)

## Déploiement backend — point important
Le build Docker utilise `SQLX_OFFLINE=true`. Après toute modification de requête SQL dans le backend, il faut regénérer le cache SQLx avant de pousser :
```bash
cd backend
cargo sqlx prepare
git add .sqlx/
git commit -m "fix: sqlx cache"
git push
```
Le push déclenche automatiquement GitHub Actions → build image → SSH deploy sur VPS.

## Gestion des secrets — `.env`
`backend/src/secrets.rs` — `load_secrets()` async appelé au démarrage de tous les binaires. Charge le fichier `.env` via `dotenvy`.
- **VPS OVH** : secrets dans `/opt/limtrack/.env`, injectés dans le container Docker via `env_file`
- **Dev local** : `.env` à la racine du projet
- Variables requises : `DATABASE_URL`, `JWT_SECRET`, `RESEND_API_KEY`, `IOS_ACTIVATION_KEY`

## Pièges SQLx connus

### LEFT JOIN → colonne nullable
SQLx peut marquer une colonne issue d'un `LEFT JOIN` comme `NOT NULL` dans le cache offline, causant un `ColumnDecode { UnexpectedNullError }` à runtime. Forcer la nullabilité avec la syntaxe `"col_name?"` :
```sql
o.name AS "org_name?"   -- force Option<String> même si le cache dit NOT NULL
```

### Colonne `status` dans contracts_loa / contracts_insurance — valeur stale
La colonne `status` en base est initialisée à `'active'` par défaut et **n'est jamais mise à jour**. Le vrai statut (`active` / `exceeded` / `closed`) est calculé à la volée en Rust dans `contracts_handler.rs` à partir des km et des dates. Ne jamais filtrer sur `status = 'exceeded'` dans une sous-requête SQL — la valeur sera toujours `'active'`.

Pour obtenir le statut agrégé d'un véhicule (danger/warning/ok), **utiliser la vue `v_contract_status`** (migration 012) — ne pas réécrire les sous-requêtes CASE inline :
```sql
LEFT JOIN public.v_contract_status vcs ON vcs.vehicle_id = v.id
-- vcs.status AS "contract_status?"  → NULL | 'danger' | 'warning' | 'ok'
```
Si la vue n'est pas applicable (besoin du détail par contrat), reproduire la logique directement en SQL :
```sql
-- danger : km consommés >= plafond
COALESCE((SELECT value FROM mileage_log WHERE vehicle_id = v.id ORDER BY recorded_at DESC LIMIT 1), l.km_start)
    - l.km_start >= l.km_allowed
-- warning : expiration <= 30j OU projection km dépasse le plafond
l.end_date <= CURRENT_DATE + 30
OR (km_consumed::FLOAT / GREATEST(CURRENT_DATE - l.start_date, 1) * (l.end_date - l.start_date) > l.km_allowed)
-- ok : end_date >= CURRENT_DATE et non dépassé
```

### Réactivité Leptos — refresh inter-composants
Pour rafraîchir un composant enfant depuis un autre composant sans relation parent-enfant directe, utiliser un signal compteur dans le parent commun :
```rust
// Dans le composant parent
let (refresh, set_refresh) = create_signal(0u32);
// Passer refresh en prop au composant à rafraîchir
// Passer un Callback au composant qui déclenche le refresh
Callback::new(move |_| set_refresh.update(|n| *n += 1))

// Dans le composant à rafraîchir
create_effect(move |_| {
    let _ = refresh.get(); // tracker le signal
    // ... fetch
});
```

## Navigation widgets → onglets
`MileageWidget`, `ContractsWidget`, `TripsWidget` et `MaintenanceWidget` reçoivent une prop `on_navigate: Callback<()>` passée depuis `vehicle_dashboard.rs`. Clic sur le titre → `set_tab.set(DashboardTab::Kilometrage / Contracts / Trips / Maintenance)`.

## Client HTTP partagé — `api_client.rs`
`frontend/src/api_client.rs` centralise tous les appels réseau du frontend. Ne jamais réécrire de helper HTTP local dans un composant — utiliser ces fonctions :

| Fonction | Méthode | Corps | Retour |
|----------|---------|-------|--------|
| `api_get::<T>` | GET | — | `Result<T, String>` |
| `api_post` | POST | JSON | `Result<(), String>` |
| `api_post_response::<T>` | POST | JSON | `Result<T, String>` |
| `api_put` | PUT | JSON | `Result<(), String>` |
| `api_patch` | PATCH | JSON | `Result<(), String>` |
| `api_patch_empty` | PATCH | — | `Result<(), String>` |
| `api_delete` | DELETE | — | `Result<(), String>` |
| `api_delete_body` | DELETE | JSON | `Result<(), String>` |

```rust
use crate::api_client::{api_get, api_post};

let vehicles = api_get::<Vec<Vehicle>>(&url, &token).await?;
api_post(&url, &token, &serde_json::json!({"role": "editor"})).await?;
```

## Avatar marque — `components/vehicle.rs`
`make_avatar_style(make: &str) -> &'static str` — retourne un inline style `background-color/color` déterministe (somme octets % 9). Palette exclusivement froide (bleu, indigo, violet, purple, fuchsia, cyan, teal, sky, slate) pour ne pas interférer avec les badges statut contrats (rouge/amber/vert). **Utiliser des inline styles** — les classes Tailwind dynamiques générées avec `format!()` sont purgées à la compilation et disparaissent en production.

## Helpers UI partagés — `components/ui.rs`
- `format_km(km: i32) -> String` — formate un entier en "45 000 km" (espace fine `\u{202F}`)
- `format_date_fr(d: NaiveDate) -> String` — formate une date en "9 juin 2030" (nécessite `use chrono::Datelike` dans la fonction). Utilisé partout où une date est affichée : contrats LOA/assurance (widget + liste), relevés kilométriques (widget + liste).
- `get_token() -> Option<String>` — lit le JWT depuis `localStorage["jwt_token"]`
- `input_class() -> &'static str` — classes Tailwind communes pour les champs de formulaire
- `parse_error_response(resp) -> String` — lit le JSON `{"error": "..."}` ou fallback par code HTTP

## Parsing erreurs HTTP — pattern fiable en WASM
`parse_error_response` est dans `components/ui.rs` (fonction partagée, `pub async fn`). Intégrée dans `api_client.rs` pour toutes les opérations d'écriture.
1. Lit le corps via `resp.text()` + `serde_json::from_str` (ne pas revenir à `resp.json()` + `serde_wasm_bindgen::from_value::<serde_json::Value>` — échoue silencieusement en WASM)
2. Fallback par code HTTP si le JSON ne parse pas : 409 → chevauchement de période, 402/403/404/429 → messages métier explicites

## Cache WASM — Safari
Après un déploiement Cloudflare Pages, Safari peut servir l'ancien WASM. Hard refresh : **Option + Cmd + R** (pas Cmd+Shift+R qui active le mode lecture). Ou : menu **Développer → Recharger en ignorant les caches**.

## Warnings connus
- `RequestInit::method/headers/body` dépréciés → bénins, correction complexe, à faire lors d'une maj web-sys
- `web_sys 0.3` — `set_headers()` attend `&JsValue` pas `&Headers`

## Version automatique depuis les git tags
`frontend/build.rs` exécute `git describe --tags --abbrev=0` à la compilation et expose la constante `APP_VERSION` dans le WASM via `env!("APP_VERSION")`. Fallback sur `CARGO_PKG_VERSION` si aucun tag n'existe. Se re-déclenche si `.git/HEAD` ou `.git/refs/tags` changent.

```rust
// Utilisation dans about.rs / home.rs
const APP_VERSION: &str = env!("APP_VERSION");
```

**En production, c'est presque toujours le fallback `CARGO_PKG_VERSION` qui s'applique** : `deploy-frontend.yml` utilise `actions/checkout@v4` sans `fetch-depth`, donc un clone superficiel (profondeur 1, aucun tag récupéré) — `git describe` échoue systématiquement en CI. Les tags (`v1.3.2` etc.) ne sont créés que pour les soumissions App Store iOS, pas pour les déploiements web, donc ce fallback est en réalité la source pertinente pour la version affichée sur le web (`Cargo.toml` → `workspace.package.version`, à jour à chaque commit versionné). **Piège** : `CARGO_PKG_VERSION` ne contient jamais de préfixe `v` (contrairement à un tag `git describe`) — les endroits qui affichent `APP_VERSION` doivent préfixer `"v"` eux-mêmes s'ils veulent ce format (voir `home.rs`), `about.rs` l'affiche brut sans préfixe. Vérifié en confrontant le WASM réellement servi en prod (`strings frontend-*.wasm`) à cette hypothèse.

## Version actuelle
`1.5.9` — déployé en production web (Cloudflare Pages + OVH VPS) le 2026-09-15
iOS App Store : soumission **en attente** — build bloqué faute de Mac disponible (MacBook Pro en panne). Options envisagées : location cloud (MacinCloud) ou OpenCore Legacy Patcher sur MacBook Air A1466 (Xcode 26 / macOS Sequoia 15.6+ obligatoire depuis le 28/04/2026). Dernière version publiée : 1.3.2 build 1 (2026-06-13).


