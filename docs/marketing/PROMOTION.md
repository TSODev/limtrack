# LimTrack — Idées de promotion

## Communautés directement concernées (audience LOA)

### Groupes Facebook
- Groupes "LOA / LLD France", "Leasing Auto France" et similaires
- Audience exacte : personnes qui parlent de dépassements km et de frais
- Post simple avec flyer + "gratuit sur limtrack.app"
- Mention partage : "Idéal en couple — chacun saisit ses trajets, un seul suivi commun pour la voiture familiale"

### Forums automobile
- forum.caradisiac.com — section financement/LOA
- forum.auto-titel.fr
- Angle : "j'ai fait une app pour éviter les frais de dépassement LOA"

### Reddit
- r/france, r/voiture
- Post honnête et direct, sans spam
- Angle couple/famille : "Vous êtes deux à conduire la même voiture en LOA ? LimTrack permet de partager le suivi — chacun saisit ses km, les alertes sont communes"

#### Règles Reddit à respecter
- Poster dans **r/voiture en premier** — audience la plus qualifiée, moins risqué qu'un premier post sur r/france
- Attendre un peu avant de répondre aux commentaires, puis répondre à tout
- Ne pas faire de cross-post le même jour — espacer d'au moins une semaine
- Éviter les majuscules excessives et les points d'exclamation (ça sent le marketing)
- La question finale dans le post invite les réponses et alimente l'algorithme

#### Post r/voiture

**Titre :** J'ai fait une app gratuite pour suivre les km de ma LOA — retours bienvenus

**Corps :**
> Comme beaucoup ici, j'ai une voiture en LOA avec un kilométrage contractuel à ne pas dépasser.
>
> Le problème : impossible de savoir en temps réel si je suis "dans les clous" ou si je fonce vers 800 € de frais en fin de contrat. Les calculs manuels dans Excel, c'est vite abandonné.
>
> J'ai donc développé **LimTrack**, une appli web gratuite (et open source) pour ça :
> - On saisit ses relevés km au fur et à mesure
> - L'app affiche en permanence si on est en avance, en retard, ou dans les clous par rapport à la trajectoire idéale
> - Alertes automatiques si on approche du plafond (km restants, projection de dépassement)
> - On peut partager le suivi — utile si vous êtes deux à conduire la même voiture
>
> C'est dispo sur **limtrack.app** (iOS App Store aussi, si ça vous intéresse).
>
> Pas d'inscription obligatoire pour tester, pas d'abonnement, pas de données revendues.
>
> Je suis le dev solo derrière, donc tous les retours — bugs, manques, suggestions — sont vraiment bienvenus. Qu'est-ce qui manque dans votre suivi LOA aujourd'hui ?

#### Post r/france

**Titre :** J'ai créé une app open source gratuite pour éviter les frais de dépassement km en LOA

**Corps :**
> Petit projet perso qui pourrait intéresser des gens ici.
>
> En France, des milliers de conducteurs ont une LOA ou LLD avec un plafond kilométrique. Le dépassement, c'est souvent 0,10 à 0,15 €/km — ça peut vite faire plusieurs centaines d'euros à la restitution.
>
> J'ai développé **LimTrack** pour résoudre ce problème pour moi, et je l'ai mis en ligne gratuitement.
>
> Ce que ça fait :
> - Suivi précis de votre avancement vs la trajectoire idéale de votre contrat
> - Alertes km et date d'expiration
> - Partage avec les autres conducteurs du véhicule
> - Export PDF/CSV de votre historique
>
> **limtrack.app** — gratuit, open source (AGPL v3, code sur GitHub), fait entièrement en Rust.
>
> Je suis le développeur, pas un commercial — si ça vous intéresse ou si vous avez des retours, je suis là.

---

## Communautés tech (levier open source)

### Hacker News — Show HN
- Section "Show HN" : très efficace pour un projet Rust/WASM open source
- Audience mondiale, post en anglais
- Angle : stack technique originale (Rust fullstack, Leptos, WASM, Tauri iOS)
- Poster en semaine, entre 9h et 12h heure US East (le front page tourne vite)
- Ne pas bumper le post ni le cross-poster — une seule soumission

#### Règles Show HN à respecter
- Titre obligatoirement préfixé `Show HN:` — sinon le post est retiré
- Le corps doit être bref : le titre fait le travail, la conversation se passe dans les commentaires
- Répondre à chaque commentaire dans les 2 premières heures — c'est ce qui propulse le post
- Ne pas défendre ni vendre — expliquer, remercier, reconnaître les limites
- Les HN'ers adorent les détails techniques honnêtes (y compris "ça a été galère parce que...")

#### Post Show HN

**Titre :** Show HN: LimTrack – Rust fullstack lease mileage tracker (Leptos/WASM + Axum + Tauri iOS)

**Corps :**
> I built LimTrack to track the mileage on my car lease. In France (and most of Europe), leases have a hard km cap — going over means paying ~€0.12/km at return, which can add up to several hundred euros. I wanted a live view of whether I was on track, with alerts before it's too late.
>
> The stack is 100% Rust:
> - **Backend**: Axum 0.7, SQLx 0.8, PostgreSQL
> - **Frontend**: Leptos 0.6 (WASM), Tailwind CSS v4, Trunk
> - **Mobile**: Tauri v2, shipped on the iOS App Store
> - **Shared types**: single Cargo workspace, `common` crate between frontend and backend
>
> A few things that might interest HN:
> - Leptos + WASM in production works well — the main friction is compile times and the occasional borrow checker fight with reactive closures
> - The Tauri iOS pipeline (`cargo tauri ios build` → IPA → Apple Transporter) actually works end-to-end, though it has sharp edges (never build from Xcode directly, iOS 26 beta crashes WKWebView/WASM)
> - SQLx offline cache (`cargo sqlx prepare`) is needed for Docker builds — easy to forget
>
> It's open source (AGPL v3), self-hosted on a VPS, frontend on Cloudflare Pages.
>
> Live at limtrack.app — GitHub link in profile.
>
> Happy to discuss the Leptos/WASM DX, the Tauri iOS build setup, or the AGPL model for a solo project.

### Product Hunt
- Lancer un mardi ou mercredi matin (heure US East)
- Préparer description + visuels en anglais
- Demander upvotes à son réseau le jour J

### Mastodon / X (Twitter)
- Hashtags : #Rust #Leptos #OpenSource #WASM #Tauri
- Les devs Rust sont curieux des projets WASM en production
- Partager des posts techniques (architecture, retours App Store...)

---

## Créateurs de contenu

### YouTubeurs auto français
- Chaînes spécialisées leasing / bons plans voiture
- Proposition : partenariat gratuit en échange d'une mention ou démo
- Angle : "outil pratique pour leurs abonnés en LOA"

### TikTok / Instagram Reels
- Format court (30-60s) : "comment j'évite 500 € de frais LOA"
- Démonstration live de l'app
- Hook : le montant économisé (ex. "320 € économisés grâce à cette app gratuite")
- Angle couple : "On est deux à conduire notre LOA — voilà comment on suit les km ensemble sans se prendre la tête"

---

## Validation rapide (feedback réel)

### TestFlight beta
- Distribuer via des groupes Facebook à 10-20 inconnus
- Objectif : retours honnêtes avant d'investir plus de temps
- Proposer en échange d'un avis écrit

### Sondage Google Forms
- Post Reddit/Facebook : "utilisez-vous une app pour suivre vos km LOA ?"
- Valide le besoin et mesure la taille du marché potentiel

---

## Priorités suggérées

1. **Groupes Facebook LOA** — audience la plus qualifiée, gratuit, immédiat
2. **Reddit r/voiture** — feedback honnête, communauté tech-friendly
3. **Hacker News Show HN** — visibilité internationale, crédibilité open source
4. **Product Hunt** — à préparer soigneusement (visuels EN, description EN)
5. **TikTok/Reels** — plus long à préparer mais fort potentiel viral

---

## Visuels disponibles

- `social-story-1080x1920.png` — format story (Instagram, Facebook, TikTok)
- `social-square-1080x1080.png` — format carré (Instagram feed, Facebook)
