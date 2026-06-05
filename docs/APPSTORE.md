# App Store Connect — Fiche LimTrack

## Informations générales

| Champ | Valeur |
|---|---|
| Nom de l'app | LimTrack |
| Sous-titre | Suivi LOA & kilométrage auto |
| Catégorie principale | Utilitaires |
| Catégorie secondaire | Finance |
| Âge minimum | 4+ |
| Prix | €3.99 (Tier 4) — achat unique |
| Pays de référence | France |

> **Prix de lancement recommandé** : €1.99 les 2-3 premières semaines pour générer des avis,
> puis passage à €3.99. Changeable à tout moment sans nouvelle soumission.

---

## Mots-clés (98 / 100 caractères)

```
LOA,LLD,leasing,kilométrage,assurance,flotte,suivi,alerte,dépassement,contrat,véhicule,relevé,auto
```

**Justification :**

| Mot-clé | Intention |
|---|---|
| `LOA` | Intention forte — utilisateurs cherchant exactement cette solution |
| `LLD` | Location Longue Durée — autre terme courant du même besoin |
| `leasing` | Terme générique compris de tous |
| `kilométrage` | Terme central de l'app |
| `assurance` | Contrats assurance avec limite km |
| `dépassement` | Pain point principal — peur de dépasser |
| `alerte` | Fonctionnalité distinctive |
| `flotte` | Ouvre aux gestionnaires de flotte |
| `contrat` | Requête large mais pertinente |
| `véhicule` | Fallback générique |
| `relevé` | Utilisateurs qui cherchent à loguer leurs km |
| `auto` | Boost sur les recherches "assurance auto", "suivi auto" |

---

## Description courte (~170 caractères)

```
Ne dépassez plus jamais vos km LOA. Suivez vos contrats, enregistrez vos relevés, recevez des alertes avant le seuil critique.
```

---

## Description complète

```
Vous avez signé un LOA ? Évitez les pénalités de dépassement.

Chaque kilomètre compte quand vous êtes en contrat LOA ou LLD.
LimTrack surveille vos kilométrages, vous alerte avant les seuils
critiques et vous donne une vision claire de votre trajectoire —
contrat après contrat.

Fini les mauvaises surprises à la restitution.

──────────────────────────────────────────

📊 SUIVI LOA EN TEMPS RÉEL

• Enregistrez vos contrats LOA et LLD (durée, km autorisés, date de fin)
• Visualisez la trajectoire idéale vs. vos relevés réels sur un graphe clair
• Estimez le coût d'un dépassement si vous avez renseigné le tarif au km
• Suivez plusieurs véhicules en parallèle

──────────────────────────────────────────

🔔 ALERTES AVANT LE SEUIL

• Alerte quand vous approchez du plafond kilométrique (seuil personnalisable)
• Alerte d'échéance de contrat pour anticiper la restitution ou le renouvellement
• Alertes assurance : limite annuelle km et date d'expiration du contrat
• Personnalisez vos seuils dans les préférences de l'app

──────────────────────────────────────────

📋 CONTRATS LOA ET ASSURANCE

• Suivi simultané d'un contrat LOA et d'un contrat assurance par véhicule
• Vue instantanée du statut : ✅ Sain · ⚠️ Risque · ❌ Dépassé
• Historique complet des relevés avec date et source

──────────────────────────────────────────

📁 EXPORT ET PARTAGE

• Export PDF de vos contrats : rapport détaillé avec statuts et projections
• Export CSV de vos relevés : trajectoire idéale, écarts, historique complet
• Partagez un véhicule avec un proche via un code de partage (lecture seule
  ou édition)

──────────────────────────────────────────

🔒 SIMPLE, PRIVÉ, OPEN SOURCE

LimTrack est open source (AGPL v3) et hébergé en Europe. Aucune publicité,
aucun tracking comportemental. Vos données restent les vôtres.

──────────────────────────────────────────

POUR QUI ?

→ Particuliers en LOA, LLD ou crédit-bail automobile
→ Conducteurs avec une assurance à kilométrage limité
→ Tout utilisateur voulant suivre ses km contractuels au plus près

──────────────────────────────────────────

Des questions ? Un formulaire de contact est disponible directement
dans l'app (section À propos).
```

---

## Nouveautés (première soumission)

```
Première version de LimTrack sur l'App Store.

• Suivi des contrats LOA, LLD et assurance auto
• Relevés kilométriques avec graphe trajectoire idéale
• Alertes personnalisables : km et échéances de contrat
• Estimation du coût de dépassement au km
• Export PDF et CSV
• Partage de véhicule avec un proche
```

---

## Screenshots

Voir `docs/appstore-screenshots.md` pour la procédure complète.

**Ordre recommandé des 9 screenshots (iPhone 15 Plus, 1290 × 2796) :**

| # | Écran | Véhicule | Message clé |
|---|---|---|---|
| 1 | Liste des véhicules | — | Tous vos véhicules d'un coup d'œil |
| 2 | Dashboard — LOA saine | Renault Clio (AR-001-AA) | Trajectoire idéale en temps réel |
| 3 | Dashboard — alerte km | Volkswagen Golf (AR-002-BB) | Alerte à 87% du plafond |
| 4 | Dashboard — alerte date | Peugeot 208 (AR-003-CC) | Contrat expire dans 26 jours |
| 5 | Dashboard — état critique | Toyota C-HR (AR-004-DD) | LOA expirée + coût dépassement |
| 6 | Vue partagée | Citroën C3 (AR-005-EE) | Partagé en lecture seule |
| 7 | Ajout d'un relevé | N'importe quel véhicule | Saisie rapide en 2 secondes |
| 8 | Page Profil | — | Préférences de notifications |
| 9 | Page À propos | — | Open source, hébergé en Europe |

**Compte de test :** `apple.reviewer` / `AppReview2024!`

---

## Informations de contact pour Apple

| Champ | Valeur |
|---|---|
| Email support | thierry.soulie@tsodev.fr |
| URL support | https://limtrack.app |
| URL politique de confidentialité | https://limtrack.app/privacy |
| URL marketing (optionnel) | https://limtrack.app |

---

## Compte de review Apple (notes de révision)

À renseigner dans le champ "Notes pour l'évaluateur" :

```
Test account:
  Username: apple.reviewer
  Password: AppReview2024!

This is a personal mileage tracking app for LOA (French car leasing)
contracts. The account is pre-loaded with 5 vehicles in various states
(healthy, warning, expired) to demonstrate all features.

The app requires an internet connection to sync with the backend API.
```
