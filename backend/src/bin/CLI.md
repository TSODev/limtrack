# CLIs LimTrack — Documentation

Tous les utilitaires sont exécutés depuis le répertoire `backend/` et chargent automatiquement les secrets via Infisical (ou `.env` en local).

```bash
cd backend
cargo run --bin <nom> -- [options]
cargo run --bin <nom> -- --help   # aide complète
```

---

## gen-tokens

Génère un ou plusieurs jetons de licence et les insère en base de données. Les jetons sont affichés en clair **une seule fois** — ils ne peuvent pas être récupérés depuis la base (seul le hash SHA-256 est stocké).

### Options

| Option | Type | Défaut | Description |
|--------|------|--------|-------------|
| `--count <N>` | entier | `1` | Nombre de jetons à générer |
| `--days <N>` | entier | `30` | Durée en jours — valeurs autorisées : `30`, `90`, `180`, `365` |
| `--lifetime` | flag | — | Jeton illimité (≈ 100 ans). Remplace `--days` |
| `--fleet` | flag | — | Type `fleet` (gestion de flotte). Par défaut : `personal` |

### Exemples

```bash
# 5 jetons personal de 30 jours
cargo run --bin gen-tokens -- --count 5 --days 30

# 1 jeton fleet de 365 jours
cargo run --bin gen-tokens -- --count 1 --days 365 --fleet

# 1 jeton fleet lifetime (illimité)
cargo run --bin gen-tokens -- --count 1 --lifetime --fleet
```

### Sortie

```
Génération de 2 jeton(s) 365 j [personal]...

Jeton (en clair)                  Durée        Type    Statut
------------------------------------------------------------------------
ABCD-EFGH-JKLM-NPQR               365 j    personal  OK
STUV-WXYZ-2345-6789               365 j    personal  OK

Conservez ces jetons en lieu sûr. Ils ne peuvent pas être récupérés depuis la base.
```

---

## assign-license

Assigne un jeton de licence à un utilisateur existant. Le jeton doit exister en base et ne pas avoir été utilisé. La durée est cumulée si l'utilisateur a déjà une licence active.

Fonctionne en **mode manuel** (un utilisateur) ou en **mode batch** (fichier CSV).

### Options

| Option | Type | Description |
|--------|------|-------------|
| `--email <email>` | string | Email de l'utilisateur (mode manuel, requis avec `--token`) |
| `--token <XXXX-XXXX-XXXX-XXXX>` | string | Jeton de licence (mode manuel, requis avec `--email`) |
| `--file <chemin.csv>` | chemin | Fichier CSV à traiter en batch |

### Format CSV (mode batch)

Sans en-tête, une ligne par assignation :

```csv
ami@example.com,ABCD-EFGH-JKLM-NPQR
autre@example.com,STUV-WXYZ-2345-6789
```

### Exemples

```bash
# Mode manuel
cargo run --bin assign-license -- --email user@example.com --token ABCD-EFGH-JKLM-NPQR

# Mode batch
cargo run --bin assign-license -- --file batch.csv
```

### Sortie

```
Email                                Token                      Résultat
--------------------------------------------------------------------------------
user@example.com                     ABCD-EFGH-JKLM-N           OK — 365 j — expire le 2027-06-07
autre@example.com                    STUV-WXYZ-2345-6           ERREUR: Jeton déjà utilisé
```

---

## notify-expiry

Envoie un email de notification aux utilisateurs dont la licence expire dans **7**, **15** ou **30** jours. Un anti-doublon 24h empêche d'envoyer deux emails le même jour au même utilisateur.

Utilisé normalement en tâche planifiée (cron VPS), mais peut être déclenché manuellement.

### Options

Aucune option — lance directement les notifications.

### Variables d'environnement requises

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | URL de connexion PostgreSQL (VPS OVH) |
| `RESEND_API_KEY` | Clé API Resend (expéditeur `noreply@limtrack.app`) |

### Exemple

```bash
cargo run --bin notify-expiry
```

### Sortie

```
✓ RESEND_API_KEY présente
✓ Connexion PostgreSQL OK
  → Email envoyé à user@example.com (expire dans 7 jours)
✓ Terminé
```

---

## send-broadcast

Crée un message broadcast affiché à tous les utilisateurs après leur connexion. Le message s'affiche **une seule fois** par utilisateur (suivi par ID en localStorage), sous forme de banner en bas d'écran avec auto-dismiss à 10 secondes.

Seul le broadcast le plus récent non expiré est retourné par l'API.

### Options

| Option | Type | Description |
|--------|------|-------------|
| `--message <texte>` | string | **Requis.** Texte du message à afficher |
| `--days <N>` | entier | Durée d'affichage en jours. Sans cette option : pas d'expiration |
| `--exclude-ios` | flag | Masque le message pour les comptes iOS App Store |

> **Règle Apple 3.1.1** : tout message contenant un lien de don (Ko-fi, GitHub Sponsors…) ou une invitation à payer en dehors de l'App Store **doit** utiliser `--exclude-ios`.

### Exemples

```bash
# Message visible par tous, sans expiration
cargo run --bin send-broadcast -- --message "Nouvelle version disponible !"

# Message temporaire (maintenance)
cargo run --bin send-broadcast -- --message "Maintenance samedi 23h–01h UTC" --days 2

# Message de don — exclu des comptes iOS
cargo run --bin send-broadcast -- \
  --message "Si LimTrack vous est utile, un don Ko-fi est possible !" \
  --days 30 \
  --exclude-ios
```

### Sortie

```
✓ Broadcast créé
  ID          : a1b2c3d4-...
  Message     : Nouvelle version disponible !
  Expiration  : aucune
  Exclure iOS : non
```
