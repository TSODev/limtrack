# Changelog

Toutes les modifications notables de ce projet sont documentées ici.

Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/).

---

## [Unreleased]

### Ajouté
- **Système de licences par jetons** : période d'essai gratuite de 3 mois à l'inscription, puis activation par jetons (`XXXX-XXXX-XXXX-XXXX`) d'une durée de 30, 90, 180 ou 365 jours. Les jetons sont cumulables (extension à partir de la date d'expiration courante).
- **Route `GET /api/profile/license`** : retourne le statut (`trial` / `active` / `expired`), la date de fin d'essai et la date d'expiration de licence.
- **Route `POST /api/profile/redeem`** : valide et active un jeton. Le token est vérifié par son hash SHA-256 ; un jeton déjà utilisé est rejeté avec `409 Conflict`.
- **Middleware de vérification d'accès** : toutes les routes `/api/*` retournent `402 Payment Required` si le compte est expiré (essai et licence épuisés). Exempté : `/login`, `/register`, `/api/profile/license`, `/api/profile/redeem`.
- **CLI `gen-tokens`** : génère des jetons en base et les affiche en clair une seule fois (`cargo run --bin gen-tokens -- --count N --days 30|90|180|365`).
- **Section Licence dans le Profil** : affichage du statut avec badge coloré, date d'expiration et formulaire de saisie de jeton.

---

## [0.2.1] — 2026-05-28

### Ajouté
- **Vérification de la solidité des mots de passe** via `zxcvbn` (score minimum 3/4) à l'inscription (`POST /api/user/register`) et au changement de mot de passe (`POST /api/profile/password`). Le feedback est retourné en clair si le mot de passe est refusé. Le username et l'email sont passés comme contexte pour détecter les mots de passe dérivés de l'identité.

### Modifié
- **Licence** : migration de MIT vers **Elastic License 2.0 (ELv2)**. Le code reste visible mais il est désormais interdit de fournir le logiciel en tant que service hébergé (SaaS) sans accord du titulaire.

### Corrigé
- **Suppression de compte** : erreur FK lors de la suppression d'un utilisateur membre ou administrateur d'une entreprise. Les tables `fleet_roles`, `company_members` et `companies` (via `created_by`) n'étaient pas nettoyées avant le `DELETE FROM users`. L'entreprise créée par l'utilisateur est désormais supprimée en premier (cascade sur orgs/membres/rôles), puis les rôles et memberships résiduels dans d'autres entreprises.
- **Suppression de compte — transfert de propriété d'entreprise** : si l'utilisateur est créateur d'une entreprise et qu'un autre administrateur global existe, `created_by` lui est transféré plutôt que de supprimer l'entreprise. Sans autre admin, l'entreprise est supprimée. Évite la perte de données pour les co-administrateurs.

---

## [0.2.0] — 2026-05-28

### Ajouté
- **Gestion de flotte d'entreprise** : création d'entreprises (nom, SIRET), organisations hiérarchiques, gestion des membres
- **Rôles fleet** : `admin`, `manager`, `viewer` — globaux ou par organisation, avec révocation
- **Vue flotte** : liste des véhicules par entreprise et par organisation (`GET /api/companies/:id/vehicles`)
- **Assignation de véhicules** à la flotte ou à une organisation (`POST/DELETE /api/vehicles/:id/fleet`)
- **Suppression de compte** : route `DELETE /api/profile` + zone dangereuse dans l'interface profil
- **Page fleet.rs** : interface complète de gestion de flotte côté frontend (Leptos)
- **PWA** : manifest + icône odo.io pour installation web
- **Tauri iOS** : support des safe areas (notch, Dynamic Island, home indicator), refactor `API_BASE` centralisé dans `config.rs`
- **Icône app** iOS toutes tailles

### Modifié
- **Tailwind CSS v4** : migration vers `@tailwindcss/cli` (remplacement de l'intégration npx)
- **Mobile UI** : bottom sheet, boutons icônes seuls sur mobile, notification bell en `fixed` sur mobile
- **VehicleCard, overlays** : remplacement des `<div>` par `<button>` pour la compatibilité iOS Safari
- **Widget kilométrage** : sparkline trajectoire idéale corrigée avec un seul relevé ; support contrat assurance

### Corrigé
- Impossible d'ajouter un véhicule quand la liste est vide
- Sparkline trajectoire idéale avec un seul relevé kilométrique
- Compatibilité iOS Safari : `<div>` → `<button>` sur VehicleCard et overlays
- Cache SQLx régénéré pour compilation Railway (`SQLX_OFFLINE=true`)
- Suppression des warnings : imports inutilisés, `format_km` dupliqué, `last_date`
- Double bouton sur la Home Page

---

## [0.1.0] — 2026-05-21

### Ajouté
- **Backend Axum 0.7** : structure initiale avec SQLx 0.8, tracing, PostgreSQL (NeonDB)
- **Authentification** : JWT (`jsonwebtoken`) + bcrypt — `POST /login`, `POST /api/user/register`
- **Gestion de véhicules** : CRUD complet — `GET/POST /api/vehicles`, `GET/DELETE/PATCH /api/vehicles/:id`
- **Rôles d'accès** : `owner`, `editor`, `viewer` avec restriction UI selon le rôle
- **Partage de véhicule** : codes à usage unique format `XXX-XXX-XXX` valables 24h (`POST /api/vehicles/:id/share`, `POST /api/vehicles/join`)
- **Révocation d'accès** : `DELETE /api/vehicles/:id/access/:user_id`, `DELETE /api/vehicles/:id/leave`
- **Contrats LOA** : km autorisés, date début/fin, calculs projection kilométrique, statuts `active` / `exceeded` / `closed`
- **Contrats Assurance** : limite annuelle, assureur, date estimée d'atteinte
- **Relevés kilométriques** : historique avec écart entre relevés, sparkline courbe réelle vs trajectoire idéale
- **Notifications** : cloche dans la navbar, alertes seuil km et proximité d'échéance, seuils personnalisables (sliders)
- **Page Profil** : modification du mot de passe (`POST /api/profile/password`), préférences notifications (`GET/PUT /api/profile/preferences`), gestion des partages
- **Frontend Leptos 0.6** (WASM) + Trunk
- **Interface responsive** : mobile-first, bottom sheet pour sélection de véhicule, boutons icônes seuls sur mobile
- **Page d'accueil** : image de fond, responsive
- **Workspace Cargo** : crate `common` avec types partagés backend/frontend
- **Déploiement production** : Railway (backend, Dockerfile, `SQLX_OFFLINE`), Netlify/Cloudflare Pages (frontend)

### Corrigé
- Suppression des fichiers sensibles et temporaires du suivi git
- Listener backend sur `0.0.0.0` pour Railway
- Cache SQLx offline pour compilation en CI

---

## [0.1.0-alpha] — 2026-05-10

### Ajouté
- Initialisation du dépôt
- Backend Axum minimal avec SQLx et tracing
- Route `POST /api/user/register`
- Auth JWT + gestion basique des véhicules
- Premier scaffold frontend Leptos
