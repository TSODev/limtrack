# Plan de déploiement — Migration OVH VPS

**Objectif :** Migrer le backend (Railway) + BDD (NeonDB) vers un VPS OVH auto-hébergé.  
**Stack retenue :** Debian 12 + Docker Compose + Caddy + GitHub Actions (niveau Minimal)  
**Frontend :** Cloudflare Pages — inchangé.  
**VPS :** `164.132.40.109` — OVH Debian 12, user `limtrack`

---

## Phase 1 — Provisionner le VPS ✅

- [x] Commander VPS OVH (4 vCores / 8 Go RAM / 75 Go SSD, image **Debian 12**)
- [x] Accès SSH root → créer un user dédié `limtrack` avec `sudo` (`scripts/setup-vps.sh`)
- [x] Installer Docker CE (`curl -fsSL https://get.docker.com | sh`)
- [x] Ajouter `limtrack` au groupe `docker`
- [x] Configurer `ufw` : ports 22, 80, 443 ouverts uniquement
- [x] Pointer `api.limtrack.app` → `164.132.40.109` (DNS Cloudflare, DNS only, TTL 300)

---

## Phase 2 — Dockeriser le backend ✅

- [x] Créer `Dockerfile.vps` multi-stage (`rust:slim` + `debian:bookworm-slim`)
- [x] `SQLX_OFFLINE=true` au build, port 3000
- [x] Build validé via GitHub Actions

---

## Phase 3 — Docker Compose + Caddy ✅

- [x] `docker-compose.yml` : postgres + backend + caddy + adminer + uptime-kuma
- [x] `Caddyfile` : reverse proxy `api.limtrack.app → backend:3000`, TLS auto
- [x] Variables d'env via `.env` sur le VPS (sans Infisical)
- [x] Adminer accessible via tunnel SSH (`ssh -L 8080:localhost:8080 limtrack@164.132.40.109`)
- [x] Uptime Kuma accessible via tunnel SSH (`ssh -L 3001:localhost:3001 limtrack@164.132.40.109`)

> **Note :** `POSTGRES_PASSWORD` doit être URL-safe — utiliser `openssl rand -hex 32` (pas base64).

---

## Phase 4 — GitHub Actions → SSH deploy ✅

- [x] `.github/workflows/deploy-backend.yml` créé
- [x] Build image → `ghcr.io/tsodev/limtrack-backend:latest`
- [x] SSH deploy : `docker compose pull backend && docker compose up -d --no-deps backend`
- [x] Secrets GitHub configurés : `VPS_HOST`, `VPS_USER`, `VPS_SSH_KEY`
- [x] VPS authentifié sur ghcr.io (PAT `read:packages`, no expiration)
- [x] `docker/setup-buildx-action` configuré avec `driver: docker` — évite le pull `moby/buildkit` depuis Docker Hub (timeout réseau intermittent sur les runners GitHub Actions)

---

## Phase 5 — Migration BDD ✅

- [x] `pg_dump` NeonDB → `limtrack_dump.sql`
- [x] Copié sur VPS via `scp`
- [x] Importé dans PostgreSQL local (`docker compose exec postgres psql`)
- [x] Intégrité vérifiée en production (app fonctionnelle, toutes migrations appliquées)

---

## Phase 6 — Mise en production ✅

- [x] Basculer le DNS `api.limtrack.app` → `164.132.40.109` (Cloudflare, DNS only, TTL 300)
- [x] Caddy a obtenu le certificat Let's Encrypt (tls-alpn-01)
- [x] `curl https://api.limtrack.app/api/user/register` → HTTP 405 ✅
- [x] Routes critiques testées en production (auth, vehicles, contracts, license, fleet)
- [ ] Activer les snapshots automatiques OVH (Control Panel → Backup)

---

## Phase 7 — Monitoring + cleanup ✅

- [x] Uptime Kuma déployé (port 3001, tunnel SSH)
- [x] Monitors configurés : Backend (`http://backend:3000/health`), PostgreSQL TCP (`postgres:5432`)
- [x] Alertes email configurées via SMTP Resend (`noreply@limtrack.app` → `thierry.soulie@tsodev.fr`)
- [x] `pg_dump` cron configuré — backup quotidien à 2h dans `/opt/limtrack/backups/`, rétention 30 jours
- [x] Production stable — app en ligne depuis v1.2.0 (2026-06-09)
- [ ] Résilier Railway + NeonDB (optionnel — comptes désactivés)

## Phase 8 — Sécurité complémentaire ✅

- [x] **Fail2ban** installé (`sudo apt install fail2ban`)
- [x] `/etc/fail2ban/jail.local` créé depuis `jail.conf` — section `[sshd]` : `backend = systemd`, `maxretry = 3`, `findtime = 5m`, `bantime = 30m`
- [x] Jail SSH actif — 3 IPs bannies dès la mise en service (bots détectés immédiatement)
- [ ] Snapshots automatiques OVH (Control Panel → Backup)

---

## Migrations appliquées en production

| Migration | Contenu |
|-----------|---------|
| 001 | Schéma initial (users, vehicles, contracts, mileage) |
| 002 | `license_tokens.license_type` (personal/fleet) |
| 003 | `users.expiry_notif_sent_at` |
| 004 | `license_requests` (anti-doublon formulaire public) |
| 005 | `users.is_admin` |
| 006 | `contracts_loa.price_per_extra_km` |
| 007 | `users.is_ios` |
| 008 | `users.password_reset_token` + `password_reset_expires_at` |
| 009 | `vehicles.archived_at` |
| 010 | `broadcasts` |
| 011 | `contracts_insurance.auto_renew` |
| 012 | `VIEW v_contract_status` |
| 013 | `users.license_type` |

---

## Fichiers créés

```
Dockerfile.vps
docker-compose.yml
Caddyfile
scripts/setup-vps.sh
.github/workflows/deploy-backend.yml
docs/PLAN_DEPLOIEMENT_VPS.md
```

---

## Notes opérationnelles

- **POSTGRES_PASSWORD** : utiliser `openssl rand -hex 32` — la base64 contient `/` qui casse l'URL PostgreSQL
- **ghcr.io** : le VPS doit être authentifié avec un PAT GitHub (`read:packages`) via `docker login ghcr.io`
- **Adminer** : `ssh -L 8080:localhost:8080 limtrack@164.132.40.109` → http://localhost:8080
- **Uptime Kuma** : `ssh -L 3001:localhost:3001 limtrack@164.132.40.109` → http://localhost:3001
- **Deploy automatique** : tout push sur `main` touchant `backend/**`, `common/**`, `Cargo.toml`, `Cargo.lock` ou `Dockerfile.vps` déclenche le CI/CD
- **Buildx** : driver `docker` utilisé (pas `docker-container`) — pas de cache GHA mais pas de dépendance Docker Hub
- **Migrations** : à appliquer manuellement sur le VPS via Adminer ou `docker compose exec postgres psql -U limtrack -d limtrack`
