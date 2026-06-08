#!/usr/bin/env bash
# setup-vps.sh — Provisionnement initial OVH VPS pour LimTrack
# À exécuter en root sur un Debian 12 vierge :
#   bash setup-vps.sh
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
info()    { echo -e "${GREEN}[✓]${NC} $*"; }
warning() { echo -e "${YELLOW}[!]${NC} $*"; }
die()     { echo -e "${RED}[✗]${NC} $*" >&2; exit 1; }

[[ $EUID -ne 0 ]] && die "Ce script doit être exécuté en root (sudo bash setup-vps.sh)"

APP_USER="limtrack"
APP_DIR="/opt/limtrack"

echo ""
echo "═══════════════════════════════════════════"
echo "  LimTrack — Provisionnement VPS Debian 12"
echo "═══════════════════════════════════════════"
echo ""

# ── Étape 1 : Utilisateur limtrack ──────────────────────────────────────────

if id "$APP_USER" &>/dev/null; then
    warning "L'utilisateur '$APP_USER' existe déjà — ignoré"
else
    adduser --disabled-password --gecos "" "$APP_USER"
    usermod -aG sudo "$APP_USER"
    info "Utilisateur '$APP_USER' créé"
fi

# Copier les clés SSH autorisées depuis root
if [[ -f /root/.ssh/authorized_keys ]]; then
    mkdir -p /home/$APP_USER/.ssh
    cp /root/.ssh/authorized_keys /home/$APP_USER/.ssh/authorized_keys
    chown -R $APP_USER:$APP_USER /home/$APP_USER/.ssh
    chmod 700 /home/$APP_USER/.ssh
    chmod 600 /home/$APP_USER/.ssh/authorized_keys
    info "Clés SSH copiées depuis root → $APP_USER"
else
    warning "Aucune clé SSH trouvée dans /root/.ssh/authorized_keys — à configurer manuellement"
fi

# ── Étape 2 : Pare-feu ufw ──────────────────────────────────────────────────

apt-get install -y ufw -q
ufw --force reset > /dev/null
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp   comment "SSH"
ufw allow 80/tcp   comment "HTTP (Caddy)"
ufw allow 443/tcp  comment "HTTPS (Caddy)"
ufw allow 443/udp  comment "HTTP/3 (Caddy)"
ufw --force enable
info "Pare-feu ufw configuré (22, 80, 443)"

# ── Étape 3 : Désactiver le login SSH root ───────────────────────────────────

if grep -q "^PermitRootLogin yes" /etc/ssh/sshd_config 2>/dev/null || \
   grep -q "^#PermitRootLogin" /etc/ssh/sshd_config 2>/dev/null; then
    sed -i 's/^#\?PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config
    systemctl restart sshd
    info "Login SSH root désactivé"
else
    warning "PermitRootLogin déjà configuré — vérifier /etc/ssh/sshd_config"
fi

# ── Étape 4 : Docker CE ─────────────────────────────────────────────────────

if command -v docker &>/dev/null; then
    warning "Docker déjà installé ($(docker --version)) — ignoré"
else
    apt-get update -q
    curl -fsSL https://get.docker.com | sh
    info "Docker CE installé"
fi

usermod -aG docker "$APP_USER"
systemctl enable docker
systemctl start docker
info "Utilisateur '$APP_USER' ajouté au groupe docker"

# ── Étape 5 : Répertoire de l'application ───────────────────────────────────

mkdir -p "$APP_DIR"
chown "$APP_USER:$APP_USER" "$APP_DIR"
info "Répertoire $APP_DIR créé"

# ── Étape 6 : Clé SSH pour GitHub Actions ───────────────────────────────────

GH_KEY="/home/$APP_USER/.ssh/github_actions"
if [[ -f "$GH_KEY" ]]; then
    warning "Clé GitHub Actions déjà présente — ignorée"
else
    sudo -u "$APP_USER" ssh-keygen -t ed25519 -C "github-actions-limtrack" -f "$GH_KEY" -N ""
    cat "$GH_KEY.pub" >> /home/$APP_USER/.ssh/authorized_keys
    chmod 600 /home/$APP_USER/.ssh/authorized_keys
    info "Clé SSH GitHub Actions générée"
fi

# ── Étape 7 : Template .env ─────────────────────────────────────────────────

ENV_FILE="$APP_DIR/.env"
if [[ -f "$ENV_FILE" ]]; then
    warning ".env déjà présent dans $APP_DIR — ignoré"
else
    cat > "$ENV_FILE" <<'ENVEOF'
# LimTrack — variables d'environnement VPS
# Remplir toutes les valeurs avant de lancer docker compose up

POSTGRES_PASSWORD=CHANGEME_mot_de_passe_fort
JWT_SECRET=CHANGEME_secret_jwt_64_caracteres_minimum
RESEND_API_KEY=CHANGEME_re_xxxx
IOS_ACTIVATION_KEY=CHANGEME_cle_ios
ENVEOF
    chmod 600 "$ENV_FILE"
    chown "$APP_USER:$APP_USER" "$ENV_FILE"
    info "Template .env créé dans $APP_DIR/.env"
fi

# ── Résumé final ─────────────────────────────────────────────────────────────

echo ""
echo "═══════════════════════════════════════════"
echo "  Provisionnement terminé"
echo "═══════════════════════════════════════════"
echo ""
echo "  Prochaines étapes manuelles :"
echo ""
echo "  1. Copier docker-compose.yml et Caddyfile sur le VPS :"
echo "       scp docker-compose.yml Caddyfile $APP_USER@<IP>:$APP_DIR/"
echo ""
echo "  2. Remplir les secrets dans $APP_DIR/.env :"
echo "       nano $APP_DIR/.env"
echo ""
echo "  3. Ajouter ces secrets dans GitHub Actions :"
echo "       VPS_HOST  = <IP du VPS>"
echo "       VPS_USER  = $APP_USER"
echo "       VPS_SSH_KEY = contenu de :"
echo ""
cat "$GH_KEY" 2>/dev/null || echo "       (clé non générée — voir $GH_KEY)"
echo ""
echo "  4. Quand la BDD est migrée, lancer :"
echo "       cd $APP_DIR && docker compose up -d"
echo ""
