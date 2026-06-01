Voici un résumé complet pour Claude Code :

---

# odo.io — Résumé projet pour Claude Code

## Présentation
Application web full-stack **entièrement en Rust** de gestion de flotte kilométrique. Suivi contrats LOA/assurance, relevés kilométriques, alertes personnalisées.

## Stack technique
- **Frontend** : Leptos 0.6 (WASM), Tailwind CSS v4, Trunk
- **Backend** : Axum 0.7, SQLx 0.8, PostgreSQL (NeonDB)
- **Auth** : JWT (jsonwebtoken) + bcrypt
- **Sécurité mots de passe** : `zxcvbn` (score ≥ 3/4) à l'inscription et au changement de mot de passe
- **Licences** : jetons SHA-256, middleware `402`, CLI `gen-tokens`
- **Mobile** : Tauri v2 (iOS configuré, Android à faire)
- **Types partagés** : crate `common` (workspace Cargo)
- **Déploiement** : Netlify (frontend) + Railway (backend)

## Architecture workspace
```
odo.io/
├── backend/src/
│   ├── main.rs
│   ├── auth.rs
│   ├── state.rs
│   ├── handlers.rs            ← login, status, helpers généraux
│   ├── lib.rs
│   ├── user_handler.rs
│   ├── vehicles_handler.rs
│   ├── contracts_handler.rs
│   ├── mileage_handler.rs
│   ├── share_handler.rs
│   ├── company_handler.rs     ← gestion flotte : entreprises, orgs, membres, rôles
│   ├── license_handler.rs     ← GET /api/profile/license + POST /api/profile/redeem
│   ├── license_middleware.rs  ← middleware 402 si licence expirée
│   └── bin/gen_tokens.rs      ← CLI génération jetons (cargo run --bin gen-tokens)
├── frontend/src/
│   ├── config.rs              ← API_BASE = "https://api.tsodev.fr"
│   ├── build.rs               ← lit git describe --tags → APP_VERSION (fallback CARGO_PKG_VERSION)
│   ├── pages/
│   │   ├── home.rs
│   │   ├── login.rs
│   │   ├── register.rs
│   │   ├── signup.rs
│   │   ├── mainpage.rs
│   │   ├── fleet.rs           ← page gestion de flotte (admin entreprise)
│   │   ├── profile.rs
│   │   └── about.rs           ← page À propos : version, description, contact mailto:
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
├── Cargo.toml                 ← version = "0.3.0"
├── migrations/                ← SQL à appliquer manuellement sur NeonDB
└── Trunk.toml
```

## URLs production
- Frontend : `https://odo.tsodev.fr` (Netlify)
- Backend : `https://api.tsodev.fr` (Railway)
- BDD : NeonDB PostgreSQL

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
```

## Routes API
```
POST   /login
POST   /api/user/register
GET    /api/profile
DELETE /api/profile              ← suppression compte (nouveau)
POST   /api/profile/password
GET    /api/profile/shares
GET/PUT /api/profile/preferences
GET/POST /api/vehicles
GET/DELETE/PATCH /api/vehicles/:id
POST   /api/vehicles/:id/share
POST   /api/vehicles/join
DELETE /api/vehicles/:id/access/:user_id
DELETE /api/vehicles/:id/leave
GET/POST /api/vehicles/:id/contracts/loa
GET/POST /api/vehicles/:id/contracts/insurance
GET/POST /api/vehicles/:id/mileage
POST/DELETE /api/vehicles/:id/fleet     ← assigner/retirer un véhicule d'une flotte

GET/POST   /api/companies
GET/DELETE /api/companies/:id
GET/POST   /api/companies/:id/organizations
DELETE     /api/companies/:id/organizations/:oid
GET/POST   /api/companies/:id/members
DELETE     /api/companies/:id/members/:uid
GET/POST   /api/companies/:id/fleet-roles
DELETE     /api/companies/:id/fleet-roles/:role_id
GET        /api/companies/:id/vehicles
GET        /api/companies/:id/organizations/:oid/vehicles
```

## Licences — système de jetons
- Période d'essai : `trial_ends_at = NOW() + 3 mois` à l'inscription
- Accès actif si `trial_ends_at > NOW() OR access_expires_at > NOW()`
- Routes exemptées du middleware : `/login`, `/api/user/register`, `/api/profile/license`, `/api/profile/redeem`
- **Mode lecture seule** : licence expirée → `GET` passe (lecture autorisée), `POST/PUT/DELETE/PATCH` → `402 Payment Required`
- Jetons : format `XXXX-XXXX-XXXX-XXXX`, SHA-256 stocké (jamais en clair), cumulables
- Durées disponibles : 30, 90, 180, 365 jours
- **Page d'inscription** : encadré info "Période d'essai gratuite — 3 mois" affiché avant le bouton de soumission ; message de succès rappelle la durée d'essai

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
```

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
- Safe areas iOS : `style="padding-top: env(safe-area-inset-top)"`
- Boutons icônes seuls mobile : `hidden md:inline` sur les textes
- Notification bell : `fixed sm:absolute`

## Tauri iOS — lancer le Simulator
```bash
# Terminal 1 — serveur statique
cd frontend && trunk build --release
python3 -c "
import http.server, socketserver, os
class H(http.server.SimpleHTTPRequestHandler):
    def guess_type(self, p):
        return 'application/wasm' if p.endswith('.wasm') else super().guess_type(p)
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy','same-origin')
        self.send_header('Cross-Origin-Embedder-Policy','require-corp')
        super().end_headers()
os.chdir('dist')
with socketserver.TCPServer(('',1430),H) as s: s.serve_forever()
"

# Terminal 2 — Tauri
cargo tauri ios dev "77F8FC35-195B-4C78-9690-28CF71ECDE54" --no-dev-server-wait
# Puis ▶ Run dans Xcode sur iPhone 13 Pro
```

## Railway — point important
Railway compile avec `SQLX_OFFLINE=true`. Après toute modification de requête SQL dans le backend, il faut regénérer le cache :
```bash
cd backend
cargo sqlx prepare
git add .sqlx/
git commit -m "fix: sqlx cache"
git push
```

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
`0.4.0`

## Roadmap
### Application mobile
- [ ] Tauri Android
- [ ] Sideloading iPhone réel → App Store

### Fonctionnalités
- [ ] Export PDF/CSV
- [ ] Notifications push natives
- [ ] Notification d'expiration de licence (J-7, in-app + email)

### Licences avancées
- [ ] **Quota de véhicules par licence** : `max_vehicles` dans `users` (défaut 3 ou 5), extensible par jeton (`vehicle_slots`). Vérification au `POST /api/vehicles`. Quota affiché dans le profil.
- [ ] **Licence entreprise** : table `company_licenses` (company_id, max_vehicles, expires_at), jeton couvrant toute la flotte avec quota véhicules, application automatique aux nouveaux véhicules assignés.

### SaaS complet
- [ ] **Paiement self-service** : intégration Stripe, achat de licence en ligne (durée + slots véhicules), génération automatique du jeton via webhook
- [ ] **Inscription libre** : onboarding sans intervention admin — inscription → paiement → activation autonome
- [ ] **Dashboard administrateur** : vue globale utilisateurs, licences actives/expirées, quotas, activité

### Sécurité — gestion des secrets
- [x] **Infisical (v0.5.0)** : `backend/src/secrets.rs` — `load_secrets()` async appelé au démarrage de tous les binaires. Si `INFISICAL_TOKEN` est présent → appel `GET /api/v4/secrets` et injection des secrets comme variables d'env. Sinon → fallback `dotenvy` (dev local).
  - Railway : seulement `INFISICAL_TOKEN`, `INFISICAL_PROJECT_ID`, `INFISICAL_ENVIRONMENT`
  - Self-hosted : ajouter `INFISICAL_URL`
  - Les noms des secrets dans Infisical = noms des variables d'env (`DATABASE_URL`, `JWT_SECRET`, `RESEND_API_KEY`)

