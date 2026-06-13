#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="$SCRIPT_DIR/../.env"

if [[ ! -f "$ENV_FILE" ]]; then
    echo "Erreur : fichier .env introuvable ($ENV_FILE)" >&2
    exit 1
fi

source "$ENV_FILE"

if [[ -z "${DATABASE_URL:-}" ]]; then
    echo "Erreur : DATABASE_URL non défini dans .env" >&2
    exit 1
fi

echo "Import de seed_fleet_demo.sql vers PostgreSQL..."
psql "$DATABASE_URL" -f "$SCRIPT_DIR/seed_fleet_demo.sql"
echo "Import terminé."
