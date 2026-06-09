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
- **Licences** : jetons SHA-256, middleware `402`, CLI `gen-tokens`, délivrance automatique via formulaire
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
│   ├── secrets.rs             ← chargement secrets Infisical au démarrage (fallback .env)
│   ├── notifier.rs            ← envoi notifications email expiration licence (Resend noreply@limtrack.app)
│   ├── handlers.rs            ← login, status, helpers généraux
│   ├── lib.rs                 ← expose notifier + secrets aux binaires CLI
│   ├── user_handler.rs
│   ├── vehicles_handler.rs
│   ├── contracts_handler.rs
│   ├── mileage_handler.rs
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
│   │   ├── mainpage.rs
│   │   ├── fleet.rs           ← page gestion de flotte (admin entreprise)
│   │   ├── profile.rs
│   │   ├── about.rs           ← page À propos : version, description, contact mailto:, Ko-fi, GitHub Sponsors
│   │   ├── request_license.rs ← page /request-license : formulaire email → jeton gratuit 365j
│   │   └── admin.rs           ← page /admin : dashboard admin (stats, users, licences, flottes)
│   └── components/
│       ├── ui.rs              ← helpers partagés : input_class(), get_token(), format_km()
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
│       └── mileage/
│           ├── mileage_list.rs
│           └── mileage_widget.rs
├── frontend/src-tauri/        ← Tauri iOS
│   ├── tauri.conf.json
│   ├── gen/apple/             ← Projet Xcode généré
│   └── icons/                 ← Icônes toutes tailles
├── common/src/lib.rs
├── Cargo.toml                 ← version = "1.2.0"
├── docs/
│   └── appstore-screenshots.md  ← guide screenshots App Store (credentials, checklist, tailles)
├── sql/
│   ├── migrations/            ← SQL à appliquer manuellement sur NeonDB
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
-- users.is_admin BOOLEAN DEFAULT FALSE — migration 005, accès dashboard admin
-- contracts_loa.price_per_extra_km FLOAT NULL — migration 006, coût dépassement km
-- users.is_ios BOOLEAN DEFAULT FALSE — migration 007, version Personal iOS (sans flotte)
-- users.password_reset_token TEXT NULL — migration 008, hash SHA-256 du token de reset
-- users.password_reset_expires_at TIMESTAMPTZ NULL — migration 008, expiry 1h
-- vehicles.archived_at TIMESTAMPTZ NULL — migration 009, archivage fin de LOA
-- broadcasts (id, message, created_at, expires_at, exclude_ios) — migration 010, messages broadcast admin
-- contracts_insurance.auto_renew BOOLEAN NOT NULL DEFAULT FALSE — migration 011, renouvellement automatique J-7
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
GET         /api/admin/stats
GET         /api/admin/users
GET         /api/admin/license-requests
POST        /api/admin/generate-token
GET         /api/admin/companies

# Broadcasts
GET         /api/broadcasts/active                                ← message actif (filtré is_ios si exclude_ios)
```

## Licences — système de jetons
- Période d'essai : `trial_ends_at = NOW() + 3 mois` à l'inscription
- Accès actif si `trial_ends_at > NOW() OR access_expires_at > NOW()`
- Routes exemptées du middleware : `/login`, `/api/user/register`, `/api/user/forgot-password`, `/api/user/reset-password`, `/api/profile/license`, `/api/profile/redeem`, `/api/license/request`, `/api/ios/activate`, `/api/admin/*`
- **Dashboard admin** : routes `/api/admin/*` protégées par `AdminUser` extractor (vérifie `users.is_admin = true`). Activer avec `UPDATE public.users SET is_admin = TRUE WHERE email = '...'`
- `AppState` contient `resend_api_key: String` (lu au démarrage depuis Infisical via `load_secrets()`)
- **Mode lecture seule** : licence expirée → `GET` passe (lecture autorisée), `POST/PUT/DELETE/PATCH` → `402 Payment Required`
- Jetons : format `XXXX-XXXX-XXXX-XXXX`, SHA-256 stocké (jamais en clair), cumulables
- Durées disponibles : 30, 90, 180, 365 jours
- **Page d'inscription** : encadré info "Période d'essai gratuite — 3 mois" affiché avant le bouton de soumission ; message de succès rappelle la durée d'essai
- **Délivrance automatique** : `POST /api/license/request` (public, sans auth) — email → jeton 365j généré et envoyé via Resend. Anti-doublon via table `license_requests` : une nouvelle demande est autorisée uniquement si le jeton précédent a déjà été utilisé (permettant le renouvellement annuel pour les LOA 3-4 ans). `RESEND_API_KEY` lu au démarrage via `AppState.resend_api_key`.

## iOS App Store — modèle payant
- **Version web (PWA)** : gratuite, licences sur demande, dons Ko-fi/GitHub Sponsors
- **Version App Store iOS** : payante (achat unique), accès lifetime inclus
- **Activation iOS** : `POST /api/ios/activate` — accordé au premier lancement Tauri, vérifié par `IOS_ACTIVATION_KEY` (Infisical). Idempotent. Stocké `ios_activated` en localStorage.
- **Détection Tauri** : `crate::config::is_tauri()` via `window.__TAURI__`. Fiable en production ; **peu fiable en dev Simulator** (ne pas s'y fier pour masquer du contenu).
- **Détection compte iOS** : champ `users.is_ios` (migration 007) — source de vérité pour masquer Licence/Flotte dans le profil, les sections web-only dans À propos, et l'alerte d'expiration de licence dans la notification bell. Stocké dans `localStorage["limtrack_is_ios"]` dès le chargement de mainpage pour éviter le flash au rendu.
- **Clé iOS** : `IOS_ACTIVATION_KEY` injectée à la compilation (`option_env!`) — à définir en variable d'env lors du build Tauri iOS.
- **Conformité AGPL v3** : exception App Store ajoutée dans `licence.md` (Thierry Soulie, détenteur unique).
- **Privacy Policy** : page `/privacy` hébergée sur `limtrack.app/privacy` (obligatoire App Store).
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

## Sécurité — protections anti-flood et limites métier

### Rate limiting — `tower_governor`
Crate `tower_governor = { version = "0.4", features = ["axum"] }`, `SmartIpKeyExtractor` (lit `X-Forwarded-For` / Cloudflare en priorité). Sous-routeur `sensitive_public` limité à **1 req/s, burst 5** :
`/login`, `/api/user/register`, `/api/user/forgot-password`, `/api/user/reset-password`, `/api/license/request`

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
- Bottom sheet : boutons Ajouter/Rejoindre en `flex-row` hors du container scrollable + spacer `height: env(safe-area-inset-bottom)` en bas du panneau
- Notification bell : position panneau `top: calc(var(--nav-top) + 3.5rem)` + bouton ✕ explicite
- Boutons icônes seuls mobile : `hidden md:inline` sur les textes

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

## Infisical — gestion des secrets
`backend/src/secrets.rs` — `load_secrets()` async appelé au démarrage de tous les binaires. Si `INFISICAL_TOKEN` est présent → appel `GET /api/v3/secrets/raw` et injection dans l'env. Sinon → fallback `dotenvy` (dev local).
- Instance : EU cloud `https://eu.infisical.com` — **Service Token** (pas Machine Identity, incompatible E2EE)
- **VPS** : secrets dans `/opt/limtrack/.env` (pas d'Infisical sur VPS — fallback dotenvy via variables Docker)
- Les noms des secrets dans Infisical = noms des variables d'env (`DATABASE_URL`, `JWT_SECRET`, `RESEND_API_KEY`)

## Pièges SQLx connus

### LEFT JOIN → colonne nullable
SQLx peut marquer une colonne issue d'un `LEFT JOIN` comme `NOT NULL` dans le cache offline, causant un `ColumnDecode { UnexpectedNullError }` à runtime. Forcer la nullabilité avec la syntaxe `"col_name?"` :
```sql
o.name AS "org_name?"   -- force Option<String> même si le cache dit NOT NULL
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

## Parsing erreurs HTTP — pattern fiable en WASM
`parse_error_response` est dans `components/ui.rs` (fonction partagée, `pub async fn`). Utilisée dans tous les fichiers d'écriture (`contract_list`, `contract_widget`, `mileage_list`, `vehicle_header`, `profile`).
1. Lit le corps via `resp.text()` + `serde_json::from_str` (ne pas revenir à `resp.json()` + `serde_wasm_bindgen::from_value::<serde_json::Value>` — échoue silencieusement en WASM)
2. Fallback par code HTTP si le JSON ne parse pas : 409 → chevauchement de période, 402/403/404/429 → messages métier explicites

## Warnings connus
- `RequestInit::method/headers/body` dépréciés → bénins, correction complexe, à faire lors d'une maj web-sys
- `web_sys 0.3` — `set_headers()` attend `&JsValue` pas `&Headers`

## Version automatique depuis les git tags
`frontend/build.rs` exécute `git describe --tags --abbrev=0` à la compilation et expose la constante `APP_VERSION` dans le WASM via `env!("APP_VERSION")`. Fallback sur `CARGO_PKG_VERSION` si aucun tag n'existe. Se re-déclenche si `.git/HEAD` ou `.git/refs/tags` changent.

```rust
// Utilisation dans about.rs
const APP_VERSION: &str = env!("APP_VERSION");
```

## Version actuelle
`1.2.0` — déployé en production (Cloudflare Pages + OVH VPS)


