# Guide d'exploitation — VPS LimTrack

## Informations générales

| Élément | Valeur |
|---------|--------|
| Fournisseur | OVH |
| IP | `164.132.40.109` |
| OS | Debian 12 (Bookworm) |
| Utilisateur | `limtrack` |
| Répertoire app | `/opt/limtrack/` |
| Backend | `https://api.limtrack.app` |

---

## Se connecter au VPS

```bash
ssh limtrack@164.132.40.109
```

---

## Services Docker

L'application tourne via Docker Compose. Tous les fichiers sont dans `/opt/limtrack/`.

### Voir l'état des services

```bash
cd /opt/limtrack
docker compose ps
```

Tu dois voir 5 containers en état `Up` :

| Container | Rôle |
|-----------|------|
| `limtrack-postgres-1` | Base de données PostgreSQL |
| `limtrack-backend-1` | API Axum (Rust) |
| `limtrack-caddy-1` | Reverse proxy + TLS Let's Encrypt |
| `limtrack-adminer-1` | Interface web PostgreSQL |
| `limtrack-uptime-kuma-1` | Monitoring |

---

## Redémarrer les services

### Redémarrer tout le stack

```bash
cd /opt/limtrack
docker compose restart
```

### Redémarrer un service spécifique

```bash
docker compose restart backend
docker compose restart caddy
docker compose restart postgres
```

### Arrêter et relancer complètement

```bash
docker compose down
docker compose up -d
```

---

## Si le VPS redémarre (coupure, maintenance OVH)

Docker est configuré en `restart: unless-stopped` — tous les containers redémarrent automatiquement au boot du VPS. Aucune action manuelle requise.

Pour vérifier que tout est reparti :

```bash
ssh limtrack@164.132.40.109
cd /opt/limtrack
docker compose ps
```

Si un container est `Restarting` ou `Exited`, consulter les logs :

```bash
docker compose logs backend --tail=50
docker compose logs caddy --tail=50
docker compose logs postgres --tail=50
```

---

## Déployer un nouveau backend

### Déploiement automatique (recommandé)

Tout push sur la branche `main` touchant `backend/**`, `common/**`, `Cargo.toml`, `Cargo.lock` ou `Dockerfile.vps` déclenche automatiquement le pipeline GitHub Actions :

1. Build de l'image Docker → `ghcr.io/tsodev/limtrack-backend:latest`
2. SSH sur le VPS → `docker compose pull backend && docker compose up -d --no-deps backend`

Suivre le déploiement : **github.com/TSODev/limtrack → Actions**

### Déploiement manuel (si besoin)

```bash
ssh limtrack@164.132.40.109
cd /opt/limtrack
docker compose pull backend
docker compose up -d --no-deps backend
docker compose logs backend --tail=20
```

### Déclencher le pipeline sans modifier le code

Sur GitHub → Actions → "Deploy Backend → OVH VPS" → **Run workflow** → Run.

---

## Adminer — interface PostgreSQL

Adminer n'est pas exposé sur internet. Il faut ouvrir un tunnel SSH depuis ton Mac.

### Ouvrir le tunnel

```bash
ssh -L 8080:localhost:8080 limtrack@164.132.40.109
```

Laisser ce terminal ouvert, puis ouvrir dans le navigateur :

**http://localhost:8080**

### Connexion Adminer

| Champ | Valeur |
|-------|--------|
| Système | `PostgreSQL` |
| Serveur | `postgres` |
| Utilisateur | `limtrack` |
| Mot de passe | *(POSTGRES_PASSWORD dans `/opt/limtrack/.env`)* |
| Base de données | `limtrack` |

### Fermer le tunnel

Fermer le terminal ou `Ctrl+C`.

---

## Uptime Kuma — monitoring

Uptime Kuma surveille les services en continu et envoie des alertes email en cas de panne.

### Ouvrir le tunnel

```bash
ssh -L 3001:localhost:3001 limtrack@164.132.40.109
```

Puis ouvrir dans le navigateur :

**http://localhost:3001**

### Monitors configurés

| Monitor | Type | URL / Cible |
|---------|------|-------------|
| LimTrack Backend | HTTP | `https://api.limtrack.app/api/user/register` |
| PostgreSQL | TCP | `postgres:5432` |
| Caddy TLS | HTTP | `https://api.limtrack.app` |

---

## Consulter les logs

```bash
ssh limtrack@164.132.40.109
cd /opt/limtrack

# Logs en temps réel
docker compose logs -f

# Logs d'un service spécifique
docker compose logs backend --tail=100
docker compose logs caddy --tail=100
docker compose logs postgres --tail=100
```

---

## Fichiers importants sur le VPS

| Fichier | Contenu |
|---------|---------|
| `/opt/limtrack/.env` | Secrets (POSTGRES_PASSWORD, JWT_SECRET, RESEND_API_KEY, IOS_ACTIVATION_KEY) |
| `/opt/limtrack/docker-compose.yml` | Configuration des services Docker |
| `/opt/limtrack/Caddyfile` | Configuration du reverse proxy |
| `/home/limtrack/.ssh/github_actions` | Clé SSH utilisée par GitHub Actions pour déployer |

---

## Mettre à jour docker-compose.yml ou Caddyfile

```bash
# Depuis ton Mac
scp docker-compose.yml limtrack@164.132.40.109:/opt/limtrack/
scp Caddyfile limtrack@164.132.40.109:/opt/limtrack/

# Puis sur le VPS
ssh limtrack@164.132.40.109
cd /opt/limtrack
docker compose up -d        # relit docker-compose.yml
docker compose reload caddy # ou : docker compose restart caddy
```

---

## Backup de la base de données

### Backup automatique (cron)

Un backup compressé est généré chaque jour à **2h du matin** dans `/opt/limtrack/backups/`.  
Les fichiers de plus de 30 jours sont supprimés automatiquement.

```bash
# Voir les backups existants
ls -lh /opt/limtrack/backups/
```

### Backup manuel

```bash
ssh limtrack@164.132.40.109
cd /opt/limtrack
docker compose exec -T postgres pg_dump -U limtrack limtrack | gzip > /opt/limtrack/backups/limtrack_$(date +%Y%m%d)_manual.sql.gz
```

### Récupérer un backup sur ton Mac

```bash
scp limtrack@164.132.40.109:/opt/limtrack/backups/limtrack_20260608.sql.gz ~/Desktop/
```

### Restaurer un backup

```bash
ssh limtrack@164.132.40.109
cd /opt/limtrack
gunzip -c /opt/limtrack/backups/limtrack_20260608.sql.gz | docker compose exec -T postgres psql -U limtrack -d limtrack
```
