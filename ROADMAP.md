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

---

## En cours — [Unreleased]

- **Notice de complexité du mot de passe** : encadré informatif dans les formulaires d'inscription et de changement de mot de passe.
- **Suppression `minlength` côté client** : validation déléguée exclusivement à `zxcvbn` côté backend.
- **`CONTACT_EMAIL` centralisé** dans `config.rs` (suppression du hard-code dans `about.rs`).
- **URL de login via `API_BASE`** : suppression de l'URL hardcodée dans `login.rs`.

---

## Rebranding — odo.io → LimTrack

> **Contexte** : le nom "odo.io" est une trademark réservée. L'application est renommée **LimTrack**.  
> Ce chantier est un prérequis au lancement SaaS public (v1.0.0).

### DNS et infrastructure

| Actuel | Cible | Service |
|--------|-------|---------|
| `odo.tsodev.fr` | `limtrack.app` (TBD) | Netlify — frontend |
| `api.tsodev.fr` | `api.limtrack.app` (TBD) | Railway — backend |
| `noreply@tsodev.fr` | `noreply@limtrack.app` (TBD) | Resend — emails |

- [ ] Choisir et enregistrer le domaine `limtrack.app` (ou `.fr`, `.io`)
- [ ] Configurer le domaine custom sur Netlify
- [ ] Configurer le domaine custom sur Railway
- [ ] Mettre à jour le domaine expéditeur dans Resend
- [ ] Redirection 301 `odo.tsodev.fr` → `limtrack.app` (période de transition)

### Code source — occurrences à remplacer

**Configuration et manifestes**
- [ ] `frontend/index.html` — `<title>` et `apple-mobile-web-app-title`
- [ ] `frontend/public/manifest.json` — `name` et `short_name` (PWA)
- [ ] `frontend/src-tauri/tauri.conf.json` — `title` (fenêtre Tauri iOS)
- [ ] `Cargo.toml` — `repository` URL (`TSODev/odo.io` → `TSODev/LimTrack`)
- [ ] `frontend/src/config.rs` — `API_BASE` URL
- [ ] `frontend/Trunk.toml` — `backend` URL

**Pages frontend**
- [ ] `frontend/src/pages/home.rs` — brand navbar + copyright
- [ ] `frontend/src/pages/mainpage.rs` — brand navbar
- [ ] `frontend/src/pages/register.rs` — brand formulaire
- [ ] `frontend/src/pages/profile.rs` — brand header + message licence expirée
- [ ] `frontend/src/pages/about.rs` — nom + description (3 occurrences)

**Composants frontend**
- [ ] `frontend/src/components/notification_bell.rs` — label `"Licence odo.io"`

**Backend**
- [ ] `backend/src/notifier.rs` — objet emails, expéditeur `from`, template HTML, liens profile

**Documentation**
- [ ] `Readme.md` — titre et mentions (5 occurrences)
- [ ] `licence.md` — définition du logiciel (ELv2)
- [ ] `CLAUDE.md` — titre et mentions
- [ ] `CHANGELOG.md` — mentions historiques (laisser en place pour la traçabilité)
- [ ] `sql/seed/SEED_FLEET_DEMO.md`

**Artefacts**
- [ ] Renommer `api/odoio-collection.postman_collection.json` → `api/limtrack-collection.postman_collection.json`
- [ ] Renommer le dépôt GitHub `TSODev/odo.io` → `TSODev/LimTrack`
- [ ] Renommer le dossier local `odo.io/` → `LimTrack/` (après push)

---

## v1.0.0 — SaaS complet

Objectif : autonomie complète des utilisateurs, sans intervention admin.

### Paiement self-service
- [ ] Intégration Stripe — achat de licence en ligne (durée + slots véhicules)
- [ ] Génération automatique du jeton via webhook Stripe
- [ ] Statut micro-entrepreneur à régulariser avant activation du mode live

### Inscription libre
- [ ] Onboarding sans intervention admin : inscription → paiement → activation autonome

### Dashboard administrateur
- [ ] Vue globale utilisateurs, licences actives/expirées, quotas, activité

### Licences avancées

**Quota de véhicules par utilisateur**
- [ ] Colonne `max_vehicles` dans `users` (défaut 3 ou 5)
- [ ] Extensible par jeton (`vehicle_slots`)
- [ ] Vérification au `POST /api/vehicles`
- [ ] Quota affiché dans le profil

**Licence entreprise**
- [ ] Table `company_licenses` (`company_id`, `max_vehicles`, `expires_at`)
- [ ] Jeton couvrant toute la flotte avec quota véhicules
- [ ] Application automatique aux nouveaux véhicules assignés

---

## Application mobile

- [ ] **Tauri Android** — build et tests
- [ ] **Sideloading iPhone réel** → App Store

---

## Fonctionnalités

- [ ] **Export PDF/CSV** — relevés kilométriques et contrats
- [ ] **Notifications push natives** — mobile Tauri
