# Sprint 00 — Bootstrap

## Métadonnées

| Champ | Valeur |
|---|---|
| **Numéro** | 00 |
| **Nom** | Bootstrap |
| **Durée estimée** | 3–5 jours |
| **Dépendances** | Aucune |
| **Branche Git** | `sprint/00-bootstrap` |
| **Statut** | IN_PROGRESS |

## État de livraison au 12 octobre 2026

Ce sprint est **IN_PROGRESS**, pas terminé. Le ZIP de livraison contient désormais un workspace Rust, une application Tauri/React, les cinq thèmes, une première navigation et une CI déclarée. Le build frontend et le build natif n'ont pas pu être validés dans l'environnement de génération ; voir `../REPORT_ALPHA_0.1.md` et `../PROGRESS.md`.

Le paragraphe ci-dessous décrit le contexte historique utilisé pour rédiger les exigences initiales ; il ne décrit pas le contenu actuel du dépôt.

## Écart entre cette spécification et le code livré (revue du 2026-10-12)

| Exigence du sprint | Réalité dans le dépôt |
|---|---|
| Tailwind, shadcn/ui, Zustand, TanStack Query, Framer Motion | Non installés (voir T-00-01) ; CSS à variables + `lucide-react` |
| `ThemeProvider` / `ThemeSwitcher` / `themes.css` / `theme.ts` | Pas de composants dédiés : un `<select>` de thème dans `App.tsx`, persistance `localStorage`, thèmes dans `app.css` |
| Composants `Button`, `Card`, `Modal`… | Classes CSS (`.button`, `.panel`…) ; pas de composants React réutilisables séparés |
| Layout `Sidebar.tsx` / `Header.tsx` / `AppLayout.tsx` | Tout est dans `App.tsx` (~370 lignes) ; à découper avant d'ajouter des modules |
| Logo SVG, favicon | Marque textuelle « S+ » en CSS ; pas de `logo.svg` |
| Crates `task-engine`, `module-*`, `shared` | Absents ; seul `crates/core` existe |
| CI lint + tests + build | `.github/workflows/ci.yml` : docs, typecheck, vitest, build, `cargo test -p superpower-core`, build Tauri Windows. Pas de clippy ni ESLint |
| Tests e2e | Aucun |

## Contexte initial (historique)

Projet vierge. Aucun code, aucune infrastructure.

**Éléments connus au moment de la rédaction :**
- La documentation `docs/`.
- Le widget `Tom-Alpha-0.1`, mentionné comme référence pour la partie tâches ; son code ne fait pas partie de cette livraison.

Les critères non cochés restent à implémenter ou à tester. Ils ne doivent pas être considérés comme satisfaits sur la seule présence des écrans.

## Objectifs du sprint

1. Créer le squelette du projet Tauri 2 + React + TypeScript.
2. Configurer le workspace Rust avec les crates vides.
3. Mettre en place le Design System (5 thèmes, composants de base).
4. Configurer la CI (lint, tests, build).
5. Créer le layout principal (sidebar + header + contenu).
6. Afficher un dashboard vide mais beau avec le thème Aurora.

## User Stories

### US-00-01 : Initialiser le projet Tauri

**En tant que** développeur,
**je veux** un projet Tauri 2 fonctionnel,
**afin de** pouvoir commencer à coder.

**Critères d'acceptation :**
- [ ] `npm create tauri-app@latest` exécuté avec succès.
- [ ] Le projet compile en dev (`npm run tauri dev`).
- [ ] Une fenêtre s'ouvre avec "Hello Tauri".
- [ ] Le frontend utilise React + TypeScript + Vite.
- [ ] Le design system CSS (variables et cinq thèmes) est configuré ; Tailwind n’est pas utilisé dans cette alpha.
- [ ] Le projet est versionné sur Git.

**Tests :**
- [ ] `npm run tauri dev` démarre sans erreur.
- [ ] `cargo build` dans `src-tauri/` réussit.

**Rollback :**
- [ ] Supprimer le dossier et recommencer.

### US-00-02 : Configurer le workspace Rust

**En tant que** développeur,
**je veux** un workspace Cargo avec les crates du projet,
**afin de** organiser le code proprement.

**Critères d'acceptation :**
- [ ] `Cargo.toml` racine déclare un workspace.
- [ ] Crates créés : `core`, `task-engine`, `module-bank`, `module-docs`, `module-rent`, `module-treasury`, `module-tax`, `module-stocks`, `shared`. *État réel : seul `crates/core` existe ; décision : ne créer chaque crate vide qu'au sprint qui en a besoin (pas de crates fantômes).*
- [ ] Chaque crate compile (`cargo build`).
- [ ] Chaque crate a un `lib.rs` (le `pub fn hello()` n'est plus exigé : un crate vide n'a pas d'intérêt).

**Tests :**
- [ ] `cargo build --workspace` réussit.
- [ ] `cargo test --workspace` réussit.

**Rollback :**
- [ ] `git reset --hard` sur le commit précédent.

### US-00-03 : Design System et thèmes

**En tant qu'** utilisateur,
**je veux** une interface pastel et apaisante avec 5 thèmes,
**afin de** prendre plaisir à utiliser l'application.

**Critères d'acceptation :**
- [ ] Variables CSS des 5 thèmes configurées sans framework CSS supplémentaire.
- [ ] Un `ThemeProvider` React gère le thème actif.
- [ ] Le thème est persisté (localStorage).
- [ ] Un `ThemeSwitcher` permet de basculer entre les 5 thèmes.
- [ ] Les composants de base (Button, Card, Input) existent et sont stylés.
- [ ] Le logo SVG est créé et affiché.

**Tests :**
- [ ] Test unitaire : `ThemeProvider` change bien la classe racine.
- [ ] Test e2e : cliquer sur un thème change les couleurs.

**Rollback :**
- [ ] Revenir au thème Aurora par défaut.

### US-00-04 : Layout principal

**En tant qu'** utilisateur,
**je veux** un layout avec sidebar et header,
**afin de** naviguer facilement.

**Critères d'acceptation :**
- [ ] Sidebar (ouverture/fermeture en mobile) avec les entrées : Cockpit, Sociétés, Comptabilité, Documents, Gestion locative, Trésorerie, Fiscalité, Paramètres. *« Inbox » viendra avec le sprint 03 ; recherche globale non livrée ; pas de collapse desktop.*
- [ ] Header avec : breadcrumbs, recherche, notifications, profil, ThemeSwitcher.
- [ ] Zone de contenu scrollable.
- [ ] Responsive (sidebar en bottom tabs sur mobile).
- [ ] Navigation fonctionnelle entre les pages (vides pour l'instant).

**Tests :**
- [ ] Test e2e : navigation entre toutes les pages.
- [ ] Test e2e : collapse/expand de la sidebar.

**Rollback :**
- [ ] Revenir à un layout minimal.

### US-00-05 : CI/CD

**En tant que** développeur,
**je veux** une CI qui vérifie la qualité,
**afin de** éviter les régressions.

**Critères d'acceptation :**
- [ ] GitHub Actions configuré.
- [ ] Contrôle frontend : `npm run typecheck` ; lint Rust/ESLint à ajouter après validation de la configuration.
- [ ] Job `test` : `cargo test` + `vitest`.
- [ ] Job `build` : `cargo build --release` + `npm run build`.
- [ ] La CI passe sur la branche `main`.

**Tests :**
- [ ] Créer une PR, vérifier que la CI tourne.

**Rollback :**
- [ ] Désactiver le workflow.

## Tâches techniques

### T-00-01 : Scaffolding Tauri

Le squelette de départ est maintenant livré à la racine du dépôt. Pour le reproduire dans un dossier vierge :

```powershell
npm create tauri-app@latest SuperPower-Alpha-0.1 -- --template react-ts
cd SuperPower-Alpha-0.1
npm install
```

> **Décision alpha (revue du 2026-10-12).** Le dépôt livré n'utilise **ni Tailwind, ni shadcn/ui, ni Framer Motion, ni Zustand, ni TanStack Query**. Le design system est du CSS à variables (`frontend/src/styles/app.css`, cinq thèmes via `data-theme`), les icônes viennent de `lucide-react`, l'état est local à `App.tsx`. Ces bibliothèques sont des options futures, à n'ajouter que sur décision explicite consignée dans « Notes et décisions ».

- [x] Ajouter Lucide : `npm install lucide-react` (présent dans `package.json`).
- [ ] (Optionnel, non retenu pour l'alpha) Tailwind, shadcn/ui, Framer Motion, Zustand, TanStack Query.

### T-00-02 : Workspace Cargo

```powershell
mkdir crates
# Créer chaque crate avec cargo new --lib
```

- [ ] Cargo.toml racine avec [workspace].
- [ ] Chaque crate déclaré dans members.
- [ ] Dépendances communes dans [workspace.dependencies].

### T-00-03 : Thèmes

- [ ] Créer frontend/src/styles/themes.css avec les 5 thèmes en variables CSS.
- [ ] Créer frontend/src/lib/theme.ts avec le ThemeProvider.
- [ ] Créer frontend/src/components/ThemeSwitcher.tsx.

### T-00-04 : Composants UI de base

- [ ] Button (variants : primary, secondary, ghost, danger).
- [ ] Card / GlassCard.
- [ ] Input, Textarea, Select.
- [ ] Badge / PillStatus.
- [ ] Toast (notifications).
- [ ] Modal / Dialog.

### T-00-05 : Layout

- [ ] Sidebar.tsx (collapsible, responsive).
- [ ] Header.tsx (breadcrumbs, recherche, profil).
- [ ] AppLayout.tsx (wrapper).
- [ ] Navigation interne de l’application ; React Router n’est pas installé dans cette alpha.

### T-00-06 : Logo

- [ ] Créer frontend/src/assets/logo.svg.
- [ ] Décliner en favicon et icône d'app.
- [ ] Intégrer dans le header et l'écran de démarrage.

### T-00-07 : CI

- [ ] .github/workflows/ci.yml.
- [ ] Jobs lint, test, build.

## Ordre d'exécution

1. T-00-01 (scaffolding)
2. T-00-02 (workspace)
3. T-00-03 (thèmes)
4. T-00-04 (composants UI)
5. T-00-05 (layout)
6. T-00-06 (logo)
7. T-00-07 (CI)
8. US-00-01 à US-00-05 (validation)

## Critères de sortie

- [ ] L'app démarre et affiche un dashboard vide mais beau.
- [ ] Les 5 thèmes fonctionnent et sont persistés.
- [ ] La navigation entre les pages fonctionne.
- [ ] La CI passe sur main.
- [ ] Le logo est intégré.
- [ ] PROGRESS.md mis à jour.
- [ ] PR mergée, tag v0.0.1.

## État après sprint (handoff)

*Sprint non clos (IN_PROGRESS). Voir `../HANDOFF.md` pour l'état exact et la suite ; reste à faire pour clore le sprint :*

- Exécuter localement `npm install && npm test && npm run typecheck && npm run build`, puis `npm run tauri:dev`.
- Faire passer la CI (jobs frontend, rust-domain, windows-native) sur une branche réelle.
- Trancher le découpage des composants UI (voir tableau d'écart ci-dessus) et ajouter un logo si souhaité.
- Décocher/cocher les critères de ce fichier uniquement après vérification.

## Notes et décisions

- [2026-10-12] Choix de Tauri 2 plutôt que 1.x pour le support mobile natif.

- [2026-10-12] ~~Choix de shadcn/ui~~ — **annulé** lors de la revue de cohérence : l'alpha utilise du CSS à variables sans framework. À rouvrir seulement par décision explicite.
