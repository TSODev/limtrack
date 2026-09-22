# Reprise — build iOS App Store v1.5.22

Contexte : préparation de la nouvelle soumission App Store de LimTrack sur le MacBook Pro
tout juste réparé (batterie). Repo cloné dans `~/Development/Rust/limtrack`.

## État de l'environnement Mac (déjà vérifié, tout est en place)
- macOS 15.7.7 (Sequoia) — répond à l'exigence Xcode 26 / macOS 15.6+ depuis le 28/04/2026
- Xcode 26.2 installé
- rustc 1.96.0, cibles iOS déjà installées (`aarch64-apple-ios`, `aarch64-apple-ios-sim`, `x86_64-apple-ios`, `wasm32-unknown-unknown`)
- `tauri-cli` 2.11.2 installé
- **Node.js 24.21.0 installé en local sans sudo** dans `~/.local/node` (pas de Homebrew sur cette machine) — nécessaire au hook Tailwind de `Trunk.toml` (`npx @tailwindcss/cli`). **Il faut l'ajouter au PATH à chaque session** :
  ```bash
  export PATH=$HOME/.local/node/bin:$PATH
  ```
- Identité de signature déjà présente dans le trousseau : "Apple Development: THIERRY ALAIN BERNARD SOULIE (LN28D34XV5)", compte Apple ID `soulie.t@orange.fr` déjà lié à l'équipe payante (teamID `9BF83RHWX9`, Individual, pas de free provisioning)
- Clé d'activation iOS (à exporter avant tout build, vérifiée à la compilation) :
  ```bash
  export IOS_ACTIVATION_KEY=limtrack-ios-2026-zaretta
  ```
- `tauri.conf.json` déjà bumpé à la version `1.5.22` (dernière soumission App Store publiée avant celle-ci : `1.3.2`, tag git `v1.3.2`)

## Où on en est
Le build a été tenté **à distance en SSH** (pont via tunnel inversé + le VPS OVH comme relais, monté spécifiquement pour cette session — plus la peine si tu es en local). Résultat :
- `trunk build --release` (compilation Rust + hook Tailwind) : ✅ réussi
- Compilation Xcode de l'app : ✅ réussie
- **`CodeSign` échoue avec `errSecInternalComponent`**, de façon reproductible, même après avoir déverrouillé le trousseau (`security unlock-keychain` + `security set-key-partition-list`) en local sur le Mac.

**Diagnostic** : ce n'est pas un problème de mot de passe/trousseau verrouillé — c'est l'isolation des "Security Sessions" de macOS : une session SSH n'a pas accès aux clés privées du trousseau pour `codesign`, même trousseau déverrouillé, même utilisateur. C'est une limitation système connue, pas un bug de config LimTrack.

## Prochaine étape (celle qui débloque tout)
Relancer le build **directement dans un Terminal.app local** (session graphique, pas SSH) :
```bash
cd ~/Development/Rust/limtrack/frontend/src-tauri
export PATH=$HOME/.local/node/bin:$PATH
export IOS_ACTIVATION_KEY=limtrack-ios-2026-zaretta
cargo tauri ios build
```
⚠️ Ne jamais faire Product → Archive dans Xcode directement (le script de pré-build attend le WebSocket de `cargo tauri ios build`).

Si `CodeSign` réussit cette fois (ce qui est attendu en local), l'IPA sort dans :
`frontend/src-tauri/gen/apple/build/arm64/LimTrack.ipa`

## Après un build réussi
1. Si App Store Connect refuse pour numéro de build déjà utilisé : incrémenter `CFBundleVersion` dans `frontend/src-tauri/gen/apple/project.yml` (actuellement `"4"`) et rebuilder.
2. Upload via **Transporter** (Mac App Store, gratuit) → glisser le `.ipa` → Deliver.
3. Remplir la fiche sur App Store Connect (notes de version) et soumettre.
4. Une fois soumis avec succès, créer le tag git `v1.5.22` pour tracer cette soumission (convention du projet — les tags ne marquent que les soumissions App Store, pas les déploiements web).

## Nettoyage
Le dossier `.bridge-tmp/` (ce fichier inclus, plus `me_to_mac.pub`, `mac_to_vps.pub`, `mac_username.txt`) a servi à un pont SSH temporaire entre les deux machines — à supprimer du repo une fois cette session terminée (`git rm -r .bridge-tmp && git commit && git push`).

Si tu utilises Little Snitch : le filtre réseau a été désactivé pendant la session de dépannage SSH — à réactiver si tu n'en as plus besoin.
