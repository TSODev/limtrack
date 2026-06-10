# Roadmap — LimTrack (ex odo.io)

## Versions livrées

| Version | Date | Points clés |
|---------|------|-------------|
| [0.1.0-alpha] | 2026-05-10 | Initialisation — Axum, SQLx, JWT, CRUD véhicules, scaffold Leptos |
| [0.1.0] | 2026-05-21 | Backend complet : contrats LOA/assurance, relevés km, partage, rôles, Netlify + Railway |
| [0.2.0] | 2026-05-28 | Gestion de flotte entreprise, rôles fleet, suppression compte, PWA, Tauri iOS |
| [0.2.1] | 2026-05-28 | zxcvbn — vérification solidité mots de passe ; migration licence MIT → ELv2 |
| [0.3.0] | 2026-05-28 | Système de licences par jetons (essai 3 mois, jetons cumulables), middleware 402 |
| [0.3.1] | 2026-05-28 | Middleware lecture seule en mode expiré ; notice période d'essai à l'inscription |
| [0.4.0] | 2026-06-01 | Alertes expiration in-app + email Resend, jetons lifetime/fleet, CLI assign-license |
| [0.5.0] | 2026-06-01 | Secrets via Infisical (EU cloud), fallback .env en local |
| [0.6.0] | 2026-06-04 | Migration AGPL v3, open source public, licences gratuites sur formulaire, Ko-fi / GitHub Sponsors |
| [0.7.0] | 2026-06-04 | Dashboard administrateur — stats, utilisateurs, licences, flottes, génération de jetons |
| [1.0.0] | 2026-06-05 | Version stable — export PDF/CSV, SaaS ready, Mobile ready (PWA), open source AGPL v3 |
| [1.1.0] | 2026-06-05 | Prix/km dépassement LOA, rapport flotte enrichi (contrats), App Store iOS ready |
| [1.1.0-appstore] | 2026-06-05 | Soumission App Store (build 2, Apple ID 6777175237) — refusé par le développeur |
| [1.1.1] | 2026-06-06 | Réinitialisation du mot de passe par email (token SHA-256, expiry 1h, Resend) |
| [1.1.2] | 2026-06-07 | Archivage véhicules (LOA terminée), renouvellement licence gratuite, suppression contrats/relevés km |
| [1.1.3] | 2026-06-07 | Confirmation avant archivage, correctifs UX (chevron, bouton amber invisible) |
| [1.2.0] | 2026-06-09 | Monitoring `/health`, messages d'erreur centralisés, renouvellement assurance auto, broadcast admin, UX iOS, navigation widgets, badge statut contrats |
| [1.2.0-appstore] | 2026-06-09 | Soumission App Store build 3 (Apple ID 6777175237) — en attente de vérification Apple |

---

## En cours — [Unreleased]

- [x] Broadcast messages — messages ponctuels admin → utilisateurs (banner, auto-dismiss, exclude_ios)
- [x] `--help` clap sur tous les CLIs
- [x] Monitoring `/health` (DB ping, Uptime Kuma)
- [x] Renouvellement assurance auto (tâche de fond 8h UTC, route `/renew`, toggle `auto_renew`)
- [x] Navigation depuis les widgets → onglets (titre cliquable, `on_navigate` callback)
- [x] Badge statut contrats sur les cartes véhicule (danger/warning/ok, calcul SQL corrigé)
- [x] Client HTTP partagé `api_client.rs` (~480 lignes supprimées sur 10 fichiers)
- [x] Vue SQL `v_contract_status` (migration 012) — calcul danger/warning/ok centralisé, `LEFT JOIN` dans `vehicles_handler.rs`
- [x] Dashboard admin v2 — 5 onglets (Aperçu/Utilisateurs/Licences/Flottes/Génération), cartes stats cliquables, croissance hebdomadaire 12 semaines, filtres client-side, édition inline utilisateurs
- [x] Migration 013 — `users.license_type` (backfill + mis à jour à l'activation du jeton)
- [x] `GET /api/admin/growth` — croissance hebdomadaire users/véhicules (`date_trunc('week', ...)`, 12 semaines)
- [x] `PATCH /api/admin/users/:id` — édition admin (username, email, is_admin, is_ios, license_type, access_expires_at)
- [x] `POST /api/admin/assign-license` — assigner un jeton existant à un compte par email
- [x] `POST /api/admin/notify-expiry` — déclencher manuellement les emails d'expiration
- [x] `POST /api/admin/broadcasts` — créer un broadcast depuis l'UI admin

**Prochaine étape :** réponse Apple en attente (build 3, soumis le 2026-06-09).

---

## Rebranding — odo.io → LimTrack

> **Contexte** : le nom "odo.io" est une trademark réservée. L'application est renommée **LimTrack**.  
> Ce chantier est un prérequis au lancement public open source (v1.0.0).

### DNS et infrastructure ✅

| Ancien | Nouveau | Service |
|--------|---------|---------|
| `odo.tsodev.fr` (Netlify) | `limtrack.app` (Cloudflare Pages) | Frontend |
| `api.tsodev.fr` | `api.limtrack.app` | OVH VPS (`164.132.40.109`) — backend |
| `noreply@tsodev.fr` | `noreply@limtrack.app` | Resend — emails |

- [x] Domaine `limtrack.app` enregistré sur Cloudflare (2026-06-03)
- [x] Frontend déployé sur Cloudflare Pages via GitHub Actions
- [x] Domaine custom `api.limtrack.app` configuré sur Railway
- [x] Domaine `limtrack.app` vérifié sur Resend
- [x] Netlify supprimé

### Code source ✅

**Configuration et manifestes**
- [x] `frontend/index.html` — `<title>` et `apple-mobile-web-app-title`
- [x] `frontend/public/manifest.json` — `name` et `short_name` (PWA)
- [x] `frontend/src-tauri/tauri.conf.json` — `title` (fenêtre Tauri iOS)
- [x] `Cargo.toml` — `repository` URL → `TSODev/limtrack`
- [x] `frontend/src/config.rs` — `API_BASE` → `https://api.limtrack.app`

**Pages frontend**
- [x] `frontend/src/pages/home.rs`
- [x] `frontend/src/pages/mainpage.rs`
- [x] `frontend/src/pages/register.rs`
- [x] `frontend/src/pages/profile.rs`
- [x] `frontend/src/pages/about.rs`

**Composants frontend**
- [x] `frontend/src/components/notification_bell.rs`

**Backend**
- [x] `backend/src/notifier.rs` — sujets, expéditeur `noreply@limtrack.app`, liens `limtrack.app`

**Documentation**
- [x] `Readme.md`
- [x] `licence.md`
- [x] `CLAUDE.md`
- [x] `CHANGELOG.md`
- [x] `sql/seed/SEED_FLEET_DEMO.md`

**Artefacts**
- [x] Renommer `api/odoio-collection.postman_collection.json` → `api/limtrack-collection.postman_collection.json`
- [x] Dépôt GitHub renommé `TSODev/odo.io` → `TSODev/limtrack`
- [x] **Logo app** — nouvelles icônes LimTrack générées (PWA 192/512px, Tauri iOS/Android/Desktop)

---

## v1.0.0 — Open Source + Communauté ✅ LIVRÉ

> Modèle retenu (2026-06-04) : **open source, licences gratuites sur demande, dons volontaires** (Ko-fi / GitHub Sponsors).  
> Objectif : partage et communauté, pas de monétisation.

### ~~Paiement self-service~~
- ~~[ ] Intégration Stripe — achat de licence en ligne (durée + slots véhicules)~~
- ~~[ ] Génération automatique du jeton via webhook Stripe~~
- ~~[ ] Statut micro-entrepreneur à régulariser avant activation du mode live~~

### Délivrance de licences gratuites ✅
- [x] Formulaire `/request-license` → génération automatique du jeton → envoi email (Resend)
- [x] Ko-fi (`ko-fi.com/limtrack`) et GitHub Sponsors (`github.com/sponsors/TSODev`)

### ~~Inscription libre~~
- ~~[ ] Onboarding sans intervention admin : inscription → paiement → activation autonome~~

### Dashboard administrateur ✅
- [x] Page `/admin` avec stats globales, liste utilisateurs, demandes de licence, génération de jeton
- [x] Section Flottes : entreprises, membres, organisations, véhicules
- [x] Bouton Admin dans la navbar (admins uniquement)
- [x] v2 — 5 onglets, cartes cliquables, croissance hebdomadaire, filtres, édition inline utilisateurs, migration 013 `license_type`

---

## Fonctionnalités

- [x] **Export PDF/CSV** — contrats (rapport + relevés avec trajectoire idéale), flotte (membres + véhicules)
- [ ] **Notifications push natives** — PWA / mobile

---

## Qualité du code — refactoring

### Client HTTP partagé (frontend) ✅
Chaque composant Leptos définit ses propres `fetch_json` / `post_json` / `patch_json` / `delete_json`. La centralisation de `parse_error_response` dans `ui.rs` est un premier pas — l'étape suivante est un vrai module `api_client.rs` avec des helpers génériques réutilisables partout.
- [x] Créer `frontend/src/api_client.rs` — 8 fonctions (`api_get`, `api_post`, `api_post_response`, `api_put`, `api_patch`, `api_patch_empty`, `api_delete`, `api_delete_body`)
- [x] Migrer les 10 composants (contracts, mileage, vehicle_header, vehicle_list, join_vehicle_button, notification_bell, profile, fleet) — ~480 lignes supprimées

### Calculs métier dupliqués SQL / Rust ✅
Le statut des contrats (`exceeded` / `active` / `closed`) et le calcul `overage_risk` existaient à la fois en Rust (`contracts_handler.rs`) et reconstitués en SQL (`vehicles_handler.rs`). Une divergence avait causé un bug (badge toujours vert).
- [x] Vue SQL `v_contract_status` (migration 012) — source de vérité unique, référencée via `LEFT JOIN` dans `vehicles_handler.rs`

---

## Documentation technique

- [ ] **Documentation API Swagger/OpenAPI** — intégration `utoipa` + `utoipa-swagger-ui`, endpoint `/api-docs` avec Swagger UI interactif. Utile pour les contributeurs open source.
