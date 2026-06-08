# Changelog

Toutes les modifications notables de ce projet sont documentées ici.

Format basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/).

---

## [Unreleased]

### Ajouté
- **Page `/support`** : page publique de support (sans authentification) avec FAQ, contact email et lien GitHub Issues. Répond à l'exigence Apple Guideline 1.5 (Support URL fonctionnelle).
- **Plaques multi-pays** : le formulaire d'ajout de véhicule supporte désormais les formats France, Belgique, Luxembourg et Suisse via un sélecteur de pays (🇫🇷🇧🇪🇱🇺🇨🇭) à gauche du champ. Formatage automatique adapté au pays choisi (`AA-000-AA` / `0-AAA-000` / `AA 0000` / `AA 000000`), placeholder, pattern HTML5 et texte d'aide réactifs.

### Infrastructure
- **Migration OVH VPS** : backend et PostgreSQL migrés de Railway + NeonDB vers un VPS OVH auto-hébergé (Debian 12, 4 vCores / 8 Go RAM / 75 Go SSD, Roubaix — RGPD France).
  - Stack : Docker Compose + Caddy (TLS Let's Encrypt automatique) + GitHub Actions (CI/CD push-to-deploy)
  - `Dockerfile.vps` : image multi-stage `rust:slim` → `debian:bookworm-slim`, `SQLX_OFFLINE=true`
  - `docker-compose.yml` : postgres + backend + caddy + adminer + uptime-kuma
  - `scripts/setup-vps.sh` : provisionnement automatisé Debian 12
  - Adminer : accès BDD via tunnel SSH (`ssh -L 8080:localhost:8080 limtrack@164.132.40.109`)
  - Uptime Kuma : monitoring + alertes email Resend, accès via tunnel SSH (port 3001)
  - Backup BDD : `pg_dump` quotidien à 2h (cron), rétention 30 jours, `/opt/limtrack/backups/`
  - Secrets : `.env` sur VPS (sans Infisical), `DATABASE_URL` pointe sur postgres local
  - Frontend Cloudflare Pages : inchangé

### Ajouté
- **Broadcast messages** : système de messages ponctuels envoyés à tous les utilisateurs, affichés une seule fois après connexion (banner bas d'écran, auto-dismiss 10s + bouton ✕). Suivi par ID en localStorage.
  - `GET /api/broadcasts/active` — retourne le broadcast actif le plus récent, filtré selon `is_ios` si `exclude_ios = true`
  - Migration `010` : table `broadcasts (id, message, created_at, expires_at, exclude_ios)`
  - Flag `exclude_ios` : masque le message pour les comptes iOS App Store (conformité règle Apple 3.1.1 — pas de sollicitation de dons)
  - CLI `send-broadcast` : `--message`, `--days` (optionnel), `--exclude-ios`
- **`--help` sur tous les CLIs** : intégration de `clap` (derive API) sur `gen-tokens`, `assign-license`, `notify-expiry` et `send-broadcast`. Chaque CLI expose une description courte, une description longue et la liste des options typées.
- **`backend/src/bin/CLI.md`** : documentation complète des utilitaires — options, variables d'environnement, exemples de commandes et format de sortie.

---

## [1.1.3] — 2026-06-07

### Ajouté
- **Confirmation avant archivage** : un clic sur "Archiver" ouvre désormais une modale explicative avant de lancer l'opération. L'encadré rappelle que les données sont conservées et restent consultables dans la section "Archivés".

### Corrigé
- **Chevron section "Archivés"** : la flèche dépliante dans le panneau liste des véhicules était trop grande — réduite de `w-3.5 h-3.5` à `w-2.5 h-2.5`, trait affiné (`stroke-width 2 → 1.5`).
- **Bouton "Archiver" invisible** : `bg-amber-600` absent du CSS Tailwind compilé causait un texte blanc sur fond blanc. CSS reconstruit.

---

## [1.1.2] — 2026-06-07

### Ajouté
- **Archivage de véhicules** : un véhicule dont la LOA est terminée peut être archivé (owner uniquement) sans perdre l'historique. Il disparaît de la liste principale et apparaît dans une section repliée "Archivés (N)". Le désarchivage remet le véhicule dans la liste active.
  - `GET /api/vehicles/archived` — liste des véhicules archivés
  - `PATCH /api/vehicles/:id/archive` — archiver (owner)
  - `PATCH /api/vehicles/:id/unarchive` — désarchiver (owner)
  - Migration `009` : colonne `archived_at TIMESTAMPTZ NULL` sur `vehicles`
  - `GET /api/vehicles` filtre désormais `archived_at IS NULL`
  - Bouton Partager masqué sur un véhicule archivé
- **Suppression de contrats** : bouton poubelle sur chaque carte contrat LOA et assurance (visible owner uniquement). Modale de confirmation "Cette action est irréversible" avant suppression effective.
  - `DELETE /api/vehicles/:id/contracts/loa/:contract_id` (owner)
  - `DELETE /api/vehicles/:id/contracts/insurance/:contract_id` (owner)
- **Suppression de relevés km** : bouton poubelle discret sur chaque ligne de l'historique (visible owner et editor). Confirmation inline dans la ligne (boutons "Non" / "Oui, supprimer") sans bloquer l'écran.
  - `DELETE /api/vehicles/:id/mileage/:entry_id` (owner ou editor)

### Amélioré
- **Login** : le champ identifiant accepte désormais le nom d'utilisateur **ou** l'adresse email (recherche insensible à la casse sur `email`).

### Corrigé
- **Renouvellement de licence gratuite** : `POST /api/license/request` autorise désormais une nouvelle demande si le jeton précédemment envoyé a déjà été utilisé (activé). Nécessaire pour les utilisateurs ayant des LOA de 3-4 ans dont la licence annuelle arrive à expiration. L'anti-doublon reste actif tant que le jeton en cours n'a pas été consommé.

---

## [1.1.1] — 2026-06-06

### Ajouté
- **Réinitialisation du mot de passe** : flux complet "mot de passe oublié" par email.
  - `POST /api/user/forgot-password` (public) — cherche l'utilisateur par email, génère un token UUID, stocke son hash SHA-256 en base avec expiry 1h, envoie un email Resend contenant le lien de reset. Répond toujours `200` (ne révèle pas si l'email existe).
  - `POST /api/user/reset-password` (public) — vérifie le token (hash SHA-256 + expiry), valide la force du nouveau mot de passe (zxcvbn ≥ 3/4), met à jour le hash bcrypt, efface le token.
  - **Migration `008`** : colonnes `password_reset_token TEXT` et `password_reset_expires_at TIMESTAMPTZ` ajoutées à `users`.
  - **Frontend** : page `/forgot-password` (formulaire email, message générique) + page `/reset-password` (lit `?token=` depuis l'URL, formulaire double saisie, redirection `/login` après succès).
  - **Lien** "Mot de passe oublié ?" ajouté à côté du label "Mot de passe" sur la page de connexion.
  - Les deux routes sont exemptées du middleware 402 (licence).

---

## [1.1.0-appstore] — 2026-06-05

### Soumission App Store iOS

- **App soumise pour vérification** le 2026-06-05 (Apple ID : 6777175237, build 2)
- **Bundle ID** : `fr.tsodev.limtrack`
- **Prix** : €3.99 achat unique (lancement recommandé €1.99)
- **Catégorie** : Utilitaires / Finance

### Ajouté — Préparation technique
- **`ITSAppUsesNonExemptEncryption = false`** dans Info.plist — déclare l'usage de chiffrement standard exempté (HTTPS/TLS iOS), supprime l'obligation de documentation ANSSI pour la France
- **`ExportOptions.plist`** : méthode `app-store-connect`, signing automatique, team `9BF83RHWX9`
- **Icônes sans canal alpha** : toutes les tailles régénérées depuis `LimTrackx1024.png` via conversion JPEG intermédiaire (requis par App Store Connect)
- **`tauri.conf.json`** : `productName` LimTrack, `version` 1.1.0, `identifier` fr.tsodev.limtrack
- **`project.yml`** : bundle ID, PRODUCT_NAME et version synchronisés avec tauri.conf.json
- **Simulateurs créés** : iPhone 13 Pro Max (1284×2778, screenshots App Store) + iPad Pro 13" iOS 18.1 (screenshots iPad requis)
- **`docs/APPSTORE.md`** : fiche complète App Store Connect (mots-clés, descriptions, screenshots, prix, compte review)

### Corrigé
- **Build number** incrémenté à `2` après rejet Transporter (signing + alpha)

---

## [1.1.0] — 2026-06-05

### Ajouté — Préparation App Store iOS
- **Seed App Store review** : `sql/seed/seed_appstore_review.sql` — compte `apple.reviewer / AppReview2024!` avec 5 véhicules couvrant tous les cas d'usage iOS (LOA saine ✅, alerte km ⚠️, alerte date ⚠️, expiré+dépassé ❌, partagé 👁). Script `import_appstore_review.sh`.
- **Guide screenshots** : `docs/appstore-screenshots.md` — procédure complète Simulator, credentials, checklist tests, plan des 9 screenshots, tailles requises App Store (iPhone 15 Plus 6.7" obligatoire).

### Ajouté — Contrats LOA
- **Prix/km dépassement LOA** : champ optionnel `price_per_extra_km` (Float) sur les contrats LOA (migration `006`). Renseignable à la création ou via le bouton "€/km" sur chaque carte contrat. Accepte virgule et point comme séparateur décimal.
- **Estimation du coût de dépassement** : affiché en rouge/orange sur le widget dashboard, la liste détaillée et le rapport PDF — coût réel si dépassé, coût projeté si risque. Calcul : km_excess × prix/km.
- **Édition prix/km** : `PATCH /api/vehicles/:id/contracts/loa/:contract_id` — seul `price_per_extra_km` est modifiable (autres champs verrouillés — termes légaux).

### Ajouté — Rapport de flotte
- **GET /api/companies/:id/fleet-report** : endpoint dédié retournant véhicules + contrats actifs/dépassés en 3 requêtes SQL (`ANY($1)`) — 1 seul appel HTTP depuis le frontend.
- **PDF flotte enrichi** : chaque véhicule affiche ses contrats actifs avec km consommés, restants, projection, statut coloré (actif/risque/dépassé) et coût estimé si prix/km renseigné.
- **Types partagés** : `FleetReportVehicle` + `FleetReportContract` dans `common`.

### Ajouté — App Store iOS
- **Version Personal iOS** (`is_ios = true`) : flag en base, migration `007`. Masque la section Flotte dans la navbar et le profil. Sections Licence et Flotte dans le profil pilotées par `is_ios` (DB) et non par `is_tauri()` — fonctionne aussi depuis un navigateur web.
- **Conformité Apple (règle 3.1.1)** : sections "Licence gratuite" et "Soutenir le projet" (Ko-fi/GitHub Sponsors) masquées sur la page À propos pour les comptes iOS. Détection via `is_ios` du profil + cache localStorage (pas de flash au chargement).
- **Activation lifetime iOS** : `POST /api/ios/activate` — accordé au premier lancement, vérifié par `IOS_ACTIVATION_KEY` (Infisical), idempotent.
- **Page `/privacy`** : politique de confidentialité RGPD (obligatoire App Store).
- **Exception AGPL v3 App Store** : ajoutée dans `licence.md`.

### Corrigé — Safe area iOS
- **CSS variable `--nav-top`** : remplace `env(safe-area-inset-top)` inline dans tous les navbars. En contexte Tauri (classe `tauri-ios` posée par JS synchrone), utilise `max(env(safe-area-inset-top), 44px)` pour couvrir le Dynamic Island iPhone 15 Plus.
- **`overscroll-behavior-y: none`** sur le body — bloque le rubber-band iOS qui faisait remonter le contenu derrière l'encoche.
- **Bottom sheet** : boutons Ajouter/Rejoindre extraits du container scrollable (`hide_actions` prop sur `Vehicle_list`) et placés directement dans le flex column du panneau, suivis d'un spacer `env(safe-area-inset-bottom)`.
- **Panneau notifications** : position `top: calc(var(--nav-top) + 3.5rem)` — toujours sous la navbar quelle que soit la safe area. Bouton ✕ ajouté dans l'en-tête.
- **Champs username** : `autocapitalize="none"` + `autocorrect="off"` sur login et register — désactive la majuscule automatique iOS.

### Corrigé
- **CORS** : méthode `PATCH` ajoutée aux méthodes autorisées.

### Supprimé
- `frontend/src/pages/signup.rs` — page orpheline non référencée.

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
