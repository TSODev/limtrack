# App Store — Screenshots & Tests iOS

## Lancer le Simulator

**Terminal 1 — serveur statique**
```bash
cd frontend
trunk build --release
python3 -c "
import http.server, socketserver, os
class H(http.server.SimpleHTTPRequestHandler):
    def guess_type(self, p):
        return 'application/wasm' if str(p).endswith('.wasm') else super().guess_type(p)
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy','same-origin')
        self.send_header('Cross-Origin-Embedder-Policy','require-corp')
        super().end_headers()
    def log_message(self, *a): pass
os.chdir('dist')
with socketserver.TCPServer(('',1430),H) as s: s.serve_forever()
"
```

**Terminal 2 — Tauri iOS**

Pour les **screenshots App Store** (1284 × 2778 — accepté par App Store Connect) :
```bash
cargo tauri ios dev "F50045E7-028E-485C-912C-C35154674374" --no-dev-server-wait
```
Puis **▶ Run** dans Xcode sur **iPhone 13 Pro Max** (1284 × 2778).

Pour les **tests de développement** (6.1") :
```bash
cargo tauri ios dev "77F8FC35-195B-4C78-9690-28CF71ECDE54" --no-dev-server-wait
```
Puis **▶ Run** dans Xcode sur **iPhone 13 Pro** (1170 × 2532).

**Prendre un screenshot dans le Simulator :** `Cmd+S` → sauvegardé sur le Bureau.

---

## Compte de review App Store

| Champ    | Valeur                          |
|----------|---------------------------------|
| Login    | `apple.reviewer`                |
| Mot de passe | `AppReview2024!`            |
| Email    | `appstore-review@limtrack.app`  |

> Compte secondaire (partage) : `demo.friend` / `DemoFriend2024!`

---

## Véhicules disponibles

| Plaque      | Véhicule              | État                                           |
|-------------|-----------------------|------------------------------------------------|
| AR-001-AA   | Renault Clio V        | LOA saine 55% km, assurance AXA ✅             |
| AR-002-BB   | Volkswagen Golf VIII  | LOA à 87% km → alerte dépassement proche ⚠️   |
| AR-003-CC   | Peugeot 208           | LOA expire dans ~26 jours → alerte date ⚠️    |
| AR-004-DD   | Toyota C-HR           | LOA expirée + 1 500 km dépassés (300 €) ❌    |
| AR-005-EE   | Citroën C3 Aircross   | Partagé par `demo.friend` (rôle viewer) 👁     |

---

## Checklist tests iOS

### Safe area
- [ ] Navbar principale : titre "LimTrack" sous l'encoche (pas dedans)
- [ ] Scroll vers le haut : pas de contenu qui passe derrière l'encoche
- [ ] Pages secondaires (Profil, À propos) : bouton "Retour" accessible et cliquable
- [ ] Bottom sheet (liste véhicules) : boutons "Ajouter" / "Rejoindre" au-dessus du home indicator

### Compte iOS (`apple.reviewer`)
- [ ] Page Profil : **pas** de section Licence
- [ ] Page Profil : **pas** de section Gestion de flotte
- [ ] Page À propos : **pas** de section "Licence gratuite" ni lien `/request-license`
- [ ] Page À propos : **pas** de boutons Ko-fi / GitHub Sponsors
- [ ] Navbar principale : **pas** de lien "Flotte"

### Fonctionnalités à screenshotter

| # | Écran | Véhicule | Points clés |
|---|-------|----------|-------------|
| 1 | Liste des véhicules (bottom sheet ouvert) | — | 5 véhicules, indicateurs couleur, boutons sur 1 ligne |
| 2 | Dashboard principal | Renault Clio (AR-001-AA) | LOA saine, graphe trajectoire idéale, assurance AXA |
| 3 | Dashboard — alerte km | Volkswagen Golf (AR-002-BB) | Barre km à 87%, warning orange |
| 4 | Dashboard — alerte date | Peugeot 208 (AR-003-CC) | LOA expire J-26, badge date orange |
| 5 | Dashboard — état critique | Toyota C-HR (AR-004-DD) | LOA expirée, km dépassé, coût €/km affiché |
| 6 | Vue partagée | Citroën C3 (AR-005-EE) | Badge "viewer", données en lecture seule |
| 7 | Ajout d'un relevé kilométrique | N'importe quel véhicule | Formulaire saisie km |
| 8 | Page Profil | — | Préférences notifications, pas de section Licence/Flotte |
| 9 | Page À propos | — | Version app, contact, sans section dons ni licence |

---

## Tailles de screenshots requises par l'App Store

| Résolution    | Appareil                  | Obligatoire | UUID Simulator |
|---------------|---------------------------|-------------|----------------|
| 1284 × 2778   | **iPhone 13 Pro Max** (6.7") | ✅ Oui | `F50045E7-028E-485C-912C-C35154674374` |
| 1290 × 2796   | iPhone 14/15 Pro Max, 15 Plus | ❌ Non accepté par App Store Connect | — |
| 1170 × 2532   | iPhone 13 Pro (6.1")      | ❌ Non accepté | `77F8FC35-195B-4C78-9690-28CF71ECDE54` |

> App Store Connect exige 1284 × 2778 ou 1242 × 2688. Le 1290 × 2796
> (iPhone 15 Plus) n'est pas dans la liste acceptée.

---

## Import du seed (si base réinitialisée)

```bash
cd sql/seed && ./import_appstore_review.sh
```
