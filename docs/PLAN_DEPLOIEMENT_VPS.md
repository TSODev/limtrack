# Plan de déploiement — Migration OVH VPS

**Objectif :** Migrer le backend (Railway) + BDD (NeonDB) vers un VPS OVH auto-hébergé.  
**Stack retenue :** Debian 12 + Docker Compose + Caddy + GitHub Actions (niveau Minimal)  
**Frontend :** Cloudflare Pages — inchangé.

---

## Phase 1 — Provisionner le VPS

- [ ] Commander VPS OVH (4 vCores / 8 Go RAM / 75 Go SSD, image **Debian 12**)
- [ ] Accès SSH root → créer un user dédié `limtrack` avec `sudo`
- [ ] Installer Docker CE (`curl -fsSL https://get.docker.com | sh`)
- [ ] Ajouter `limtrack` au groupe `docker`
- [ ] Configurer `ufw` : ports 22, 80, 443 ouverts uniquement
- [ ] Pointer `api.limtrack.app` → IP du VPS (DNS Cloudflare)

---

## Phase 2 — Dockeriser le backend

- [ ] Créer `backend/Dockerfile` multi-stage :
  - Stage 1 : `rust:slim` → `cargo build --release` avec `SQLX_OFFLINE=true`
  - Stage 2 : `debian:bookworm-slim` → copier le binaire seul (~10 Mo)
- [ ] Vérifier que les 4 binaires CLI sont inclus (ou image séparée)
- [ ] Tester le build local

---

## Phase 3 — Docker Compose + Caddy

- [ ] Créer `docker-compose.yml` avec 3 services :
  - `postgres` (image officielle, volume persistant)
  - `backend` (image buildée, variables via `.env`)
  - `caddy` (image officielle, TLS automatique Let's Encrypt)
- [ ] Créer `Caddyfile` : reverse proxy `api.limtrack.app → backend:3000`
- [ ] Variables d'env : injectées via Infisical au démarrage (comportement existant)
- [ ] Tester `docker compose up -d` en local

---

## Phase 4 — GitHub Actions → SSH deploy

- [ ] Créer `.github/workflows/deploy-backend.yml`
- [ ] Trigger : push sur `main` (ou tag `v*`)
- [ ] Pipeline :
  1. Build image Docker → push sur GitHub Container Registry (`ghcr.io`)
  2. SSH sur VPS → `docker compose pull && docker compose up -d`
- [ ] Secrets GitHub à configurer : `VPS_HOST`, `VPS_USER`, `VPS_SSH_KEY`

---

## Phase 5 — Migration BDD

- [ ] `pg_dump` NeonDB → fichier SQL
- [ ] Copier sur VPS (`scp`)
- [ ] `psql` → importer dans PostgreSQL local
- [ ] Vérifier intégrité (tables, migrations, users)
- [ ] Mettre à jour `DATABASE_URL` dans Infisical → pointer sur `localhost:5432`

---

## Phase 6 — Mise en production

- [ ] Régénérer le cache SQLx si nécessaire (`cargo sqlx prepare`)
- [ ] Basculer le DNS `api.limtrack.app` → IP VPS
- [ ] Vérifier que Caddy obtient bien le certificat Let's Encrypt
- [ ] Tester toutes les routes critiques (auth, vehicles, license)
- [ ] Activer les backups automatiques OVH (snapshot daily)

---

## Phase 7 — Monitoring + cleanup

- [ ] Installer **Uptime Kuma** (Docker) → alertes email si backend down
- [ ] Configurer `pg_dump` cron → backup BDD quotidien (ex. `/var/backups/limtrack/`)
- [ ] Valider 48h en prod
- [ ] Résilier Railway + NeonDB

---

## Livrables à créer dans le repo

```
backend/Dockerfile
docker-compose.yml
Caddyfile
.github/workflows/deploy-backend.yml
```

---

## Notes

- PostgreSQL sur VPS : `shared_buffers = 2GB`, `effective_cache_size = 6GB`
- `SQLX_OFFLINE=true` requis au build (inchangé vs Railway)
- Infisical : ajouter `VPS` comme environnement ou réutiliser `production`
- Uptime Kuma peut tourner sur le même VPS (port 3001, derrière Caddy)
