# Changelog

Toutes les modifications notables de ce projet sont documentées ici.

Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/).

---

## [Unreleased]

### Ajouté — Contrats LOA
- **Prix/km dépassement LOA** : champ optionnel `price_per_extra_km` (Float) sur les contrats LOA (migration `006`). Renseignable à la création ou via le bouton "€/km" sur chaque carte contrat. Accepte virgule et point comme séparateur décimal.
- **Estimation du coût de dépassement** : affiché en rouge/orange sur le widget dashboard, la liste détaillée et le rapport PDF — coût réel si dépassé, coût projeté si risque. Calcul : km_excess × prix/km.
- **Édition prix/km** : `PATCH /api/vehicles/:id/contracts/loa/:contract_id` — seul `price_per_extra_km` est modifiable (autres champs verrouillés — termes légaux).

### Ajouté — Rapport de flotte
- **GET /api/companies/:id/fleet-report** : endpoint dédié retournant véhicules + contrats actifs/dépassés en 3 requêtes SQL (`ANY($1)`) — 1 seul appel HTTP depuis le frontend.
- **PDF flotte enrichi** : chaque véhicule affiche ses contrats actifs avec km consommés, restants, projection, statut coloré (actif/risque/dépassé) et coût estimé si prix/km renseigné.
- **Types partagés** : `FleetReportVehicle` + `FleetReportContract` dans `common`.

### Ajouté — App Store iOS
- **Conformité Apple (règle 3.1.1)** : section dons Ko-fi/GitHub Sponsors masquée dans la version Tauri.
- **Activation lifetime iOS** : `POST /api/ios/activate` — accordé au premier lancement, vérifié par `IOS_ACTIVATION_KEY` (Infisical), idempotent.
- **Section Licence masquée sur iOS** : accès complet inclus dans l'achat App Store.
- **Page `/privacy`** : politique de confidentialité RGPD (obligatoire App Store).
- **Exception AGPL v3 App Store** : ajoutée dans `licence.md`.

### Corrigé
- **CORS** : méthode `PATCH` ajoutée aux méthodes autorisées.

---

## [1.0.0] — 2026-06-05

> Première version stable. Application complète, open source AGPL v3, SaaS ready et Mobile ready (PWA).

### Ajouté
- **Export PDF contrat** : bouton PDF sur chaque carte LOA et Assurance — rapport stylé (statut, km consommés, restants, projection, dates) ouvert dans un nouvel onglet avec impression automatique.
- **Export CSV relevés kilométriques** : bouton CSV sur chaque carte contrat — 7 colonnes : Date, Kilométrage, Écart relevé précédent, Trajectoire idéale, Écart vs idéale, Statut trajectoire (En avance / En retard), Source.
- **Export PDF flotte** : rapport complet de l'entreprise (membres avec rôles, véhicules par organisation) généré côté frontend.
- **Export CSV flotte** : liste des véhicules de la flotte (marque, modèle, immatriculation, année, organisation).
- Tous les exports sont générés **100% côté frontend** (WASM) via Blob + URL.createObjectURL — aucun appel backend supplémentaire.

---

## [0.7.0] — 2026-06-04

### Dashboard administrateur
- **Page `/admin`** : accès réservé aux comptes `is_admin = true` (migration `005`, colonne `users.is_admin`).
- **Bouton Admin** dans la navbar principale — visible uniquement pour les admins, desktop (texte + icône) et mobile (icône seule).
- **Stats globales** : total utilisateurs, en essai, actifs, expirés, demandes de licence.
- **Génération de jeton** : formulaire admin (durée, type, email optionnel) — si l'email correspond à un compte existant, le jeton est assigné directement et la licence appliquée. Feedback visuel vert/orange. Rafraîchissement automatique après succès.
- **Liste utilisateurs** : tableau complet avec statut (trial/active/expired) et date d'expiration.
- **Demandes de licence** : historique des emails passés par `/request-license`.
- **Section Flottes** : entreprises dépliables avec membres (rôle fleet), organisations et véhicules.
- Routes `/api/admin/*` exemptées du middleware licence, protégées par l'extracteur `AdminUser`.

---

## [0.6.0] — 2026-06-04

### Infrastructure
- **Rebrand odo.io → LimTrack** : nouveau nom, nouveau domaine `limtrack.app`
- **Cloudflare Pages** : frontend déployé sur Cloudflare Pages via GitHub Actions, en remplacement de Netlify
- **`api.limtrack.app`** : backend Railway sur le nouveau domaine
- **Resend** : domaine d'envoi migré vers `limtrack.app`, domaine vérifié, DMARC configuré
- **GitHub** : dépôt renommé `TSODev/limtrack`, passé en **public**

### Modèle open source
- **Licence AGPL v3** : migration ELv2 → GNU Affero General Public License v3.0.
- **Licences gratuites sur demande** : `POST /api/license/request` — génère un jeton 365 jours et l'envoie par email. Anti-doublon 1 jeton/email (migration `004`).
- **Page `/request-license`** : formulaire email avec feedback et section dons.
- **Ko-fi** `ko-fi.com/limtrack` et **GitHub Sponsors** `github.com/sponsors/TSODev` intégrés dans `/about` et `/request-license`.

### Amélioré
- **Notice de complexité du mot de passe** : encadré informatif dans les formulaires d'inscription et de changement de mot de passe.
- **Suppression de la contrainte `minlength="8"` côté client** : validation exclusivement par `zxcvbn` côté backend.
- **`CONTACT_EMAIL` centralisé dans `config.rs`**.
- **URL de login via `API_BASE`** : suppression de l'URL hardcodée dans `login.rs`.

---

## [0.5.0] — 2026-06-01

### Ajouté
- **Gestion des secrets via Infisical** : `backend/src/secrets.rs` — fonction `load_secrets()` async appelée au démarrage de tous les binaires (`backend`, `gen-tokens`, `assign-license`, `notify-expiry`). Les secrets (`DATABASE_URL`, `JWT_SECRET`, `RESEND_API_KEY`) sont récupérés depuis [Infisical](https://infisical.com) via l'API REST (`/api/v3/secrets/raw`) et injectés comme variables d'environnement avant tout démarrage.
- **Fallback `.env`** : si `INFISICAL_TOKEN` est absent (développement local), `dotenvy` est utilisé automatiquement — aucun changement de workflow en local.

### Modifié
- **Railway** : `DATABASE_URL`, `JWT_SECRET` et `RESEND_API_KEY` supprimées des variables Railway. Remplacées par `INFISICAL_TOKEN`, `INFISICAL_PROJECT_ID`, `INFISICAL_ENVIRONMENT`, `INFISICAL_URL`.

### Infrastructure
- Instance Infisical EU cloud (`eu.infisical.com`), projet `odo-backend`, environnement `prod`.
- Authentification via Service Token (compatible E2EE Infisical cloud).

---

## [0.4.0] — 2026-06-01

### Ajouté
- **Jetons lifetime** : flag `--lifetime` dans `gen-tokens` — génère un jeton de 36 500 jours (~100 ans), exempt de toute alerte d'expiration.
- **Types de licence `personal` / `fleet`** : colonne `license_type` dans `license_tokens` (migration `002`), flag `--fleet` dans `gen-tokens`. Le panneau "Gestion de flotte" dans le Profil est masqué pour les licences `personal`.
- **Alertes d'expiration in-app** : la cloche de notifications affiche une alerte quand la licence approche de son terme. Seuils adaptatifs selon la durée du jeton : J-7 (30 jours), J-15 (3 mois), J-30 (1 an). Niveau danger à J-3. Jetons lifetime exemptés.
- **Notifications email d'expiration** via [Resend](https://resend.com) : tâche tokio intégrée au backend, déclenchée quotidiennement à 8h00 UTC. Template HTML professionnel avec CTA vers `/profile`. Anti-doublon 24h. Si `RESEND_API_KEY` est absente, les notifications sont silencieusement désactivées.
- **CLI `assign-license`** : assigne un jeton existant à un utilisateur par email (`--email` + `--token`) ou en lot depuis un fichier CSV (`--file batch.csv`, format `email,token`). Cumul de licences respecté. Trace `used_by`/`used_at` en base.
- **CLI `notify-expiry`** : wrapper pour déclencher manuellement l'envoi des notifications email.
- **Migrations** : `002_license_type.sql` (colonne `license_type`), `003_expiry_notif.sql` (colonne `expiry_notif_sent_at`).

### Modifié
- **`LicenseStatus`** (type partagé `common`) : ajout des champs `days_until_expiry` et `license_type`.
- **`GET /api/profile/license`** : retourne désormais `days_until_expiry` (seuil calculé depuis le dernier jeton) et `license_type`.
- **`notifier.rs`** : logique de notification extraite dans un module partagé entre le backend et le binaire `notify-expiry`.

---

## [0.3.1] — 2026-05-28

### Ajouté
- **Notice période d'essai à l'inscription** : encadré informatif "Période d'essai gratuite — 3 mois" affiché dans le formulaire d'inscription avant le bouton de soumission. Le message de succès rappelle également la durée d'essai.
- **Affichage d'erreur dans le panneau "Véhicules de la flotte"** : si le chargement échoue, un message d'erreur rouge est affiché à la place du contenu vide.

### Modifié
- **Middleware licence — mode lecture seule** : à l'expiration, les requêtes `GET` sont désormais autorisées (consultation possible). Seules les écritures (`POST`, `PUT`, `DELETE`, `PATCH`) retournent `402 Payment Required`.
- **Page fleet — layout** : alignement sur la page Profil (`max-w-4xl mx-auto` + `space-y-4 md:space-y-8`). Les panneaux ne prennent plus toute la largeur disponible sur grand écran.
- **Formulaire d'inscription — messages d'erreur** : le frontend lit désormais le corps JSON de la réponse pour afficher le vrai message d'erreur du backend (mot de passe trop faible, email déjà utilisé, etc.) au lieu d'un message générique.

### Corrigé
- **Rafraîchissement automatique du panneau "Véhicules de la flotte"** après affectation ou retrait d'un véhicule : `FleetVehiclesSection` est maintenant notifié via un signal `fleet_refresh` incrémenté par `VehiclesSection` après chaque opération.
- **HTTP 500 sur `GET /api/companies/:id/vehicles`** : SQLx marquait `org_name` (issu d'un `LEFT JOIN`) comme `NOT NULL` dans le cache offline. Un véhicule sans organisation causait un `ColumnDecode { UnexpectedNullError }` à runtime. Corrigé via la syntaxe `"org_name?"` dans les deux requêtes concernées (`list_fleet_vehicles` et `list_org_vehicles`). Cache `.sqlx/` régénéré.

---

## [0.3.0] — 2026-05-28

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
