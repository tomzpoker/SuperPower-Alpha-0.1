# PROGRESS — SuperPower Alpha

> Source de vérité : distinguer le code écrit du code compilé/testé. Une fonctionnalité ne passe à `DONE` qu'après validation locale et revue.

## État actuel

| Champ | Valeur |
|---|---|
| **Version cible** | `0.1.0-alpha.1` |
| **Sprint en cours** | 00 — Bootstrap et 01 — Noyau (première tranche) |
| **Statut** | IN_PROGRESS — logique de base vérifiée partiellement ; build natif et SQLCipher à exécuter |
| **Dernière mise à jour** | 2026-10-12 |
| **Prochaine action** | Après ce correctif : réinstaller npm et relancer `npm test`, `npm run typecheck`, `npm run build`; installer Strawberry Perl puis relancer `cargo test --workspace` et `npm run tauri:dev`. Consigner le résultat avant le lot A. |
| **Reprise par une IA** | `AGENTS.md` → `docs/HANDOFF.md` |
| **Dépôt cible** | https://github.com/tomzpoker/SuperPower-Alpha-0.1 (vide lors de l'initialisation de cette livraison) |

## Sprints

| # | Nom | Statut | Fichier | Notes |
|---|---|---|---|---|
| 00 | Bootstrap | IN_PROGRESS | `SPRINTS/SPRINT_00_BOOTSTRAP.md` | Workspace/UI/CI présents dans les sources ; build natif pas encore vérifié |
| 01 | Noyau | IN_PROGRESS | `SPRINTS/SPRINT_01_NOYAU.md` | Sociétés, comptes de base, écritures équilibrées et audit initial codés ; validation locale en attente |
| 02 | Moteur de tâches | TODO | `SPRINTS/SPRINT_02_MOTEUR_TACHES.md` | Choisir une solution durable après prototype |
| 03 | Import bancaire | TODO | `SPRINTS/SPRINT_03_IMPORT_BANCAIRE.md` | Parser CSV et rapprochement non livrés |
| 04 | Documents | TODO | `SPRINTS/SPRINT_04_DOCUMENTS.md` | Le fichier de sprint est présent ; aucun pipeline livré |
| 05 | Gestion locative | TODO | `SPRINTS/SPRINT_05_GESTION_LOCATIVE.md` | Fichier recréé car absent du ZIP source ; baux/TVA par opération à spécifier |
| 06 | Trésorerie | TODO | `SPRINTS/SPRINT_06_TRESORERIE.md` | Projection non livrée |
| 07 | Fiscalité | TODO | `SPRINTS/SPRINT_07_FISCALITE.md` | Déclarations non livrées ; sources officielles documentées |
| 08 | Sync mobile | TODO | `SPRINTS/SPRINT_08_SYNC_MOBILE.md` | Non livré |
| 09 | Stocks | TODO | `SPRINTS/SPRINT_09_STOCKS.md` | Extension facultative |

## Vérifications effectuées (revue initiale + retour Windows du 2026-10-12)

| Contrôle | Résultat | Réserve |
|---|---|---|
| `scripts/check_docs.py` | OK dans la revue initiale | À relancer sur ce ZIP corrigé |
| `npm install` | Réussi sur le poste Windows | 6 vulnérabilités signalées ; scripts `esbuild` bloqués. Résultat obtenu avant le bump de Vitest. |
| `npm test` | 3/3 réussis sur Windows | Résultat obtenu avant le bump de Vitest. À rejouer sur ce ZIP. |
| `npm run typecheck` | ÉCHEC avant correctif | `DashboardSummary` inutilisé corrigé ; conflit Vite 6/Vitest 2 corrigé dans `package.json`, à revalider. |
| `npm run build` | ÉCHEC car `typecheck` échouait | Pas de bundle confirmé avant correctif ; à revalider. |
| `cargo test -p superpower-core` | 8/8 réussis sur Windows | Compilation et tests Rust réels du noyau. |
| `cargo test --workspace` | ÉCHEC avant compilation applicative | `openssl-sys`: `perl` introuvable dans l'environnement Windows. |
| `npm run tauri:dev` | ÉCHEC pendant compilation native | Vite démarre, mais le même problème OpenSSL/Perl arrête Tauri. |
| SQLCipher natif / tests de verrouillage | Non validés | Besoin de réussir le build workspace et de tester base + mauvais/bon mot de passe. |

Constats de la revue : immutabilité des écritures et journal « append-only » **non appliqués en base** (seulement par l'absence d'endpoint) ; bus d'événements non branché ; docs de sprints 00/01/04 en partie obsolètes ou mal formatées (corrigé) ; voir `docs/HANDOFF.md` §5.

## Livré dans la tranche alpha

- Identité de dépôt et structure React/Vite + Tauri/Rust.
- Design system CSS à cinq thèmes, navigation desktop/mobile et page Cockpit.
- Saisie de sociétés avec configuration indépendante de la forme juridique, de l'IR/IS et des paramètres TVA.
- Base locale conçue pour SQLCipher, clé dérivée via Argon2id, verrouillage/déverrouillage.
- Petit socle de comptes, validation comptable en partie double avec centimes entiers, écriture immuable dans l'interface, audit initial.
- Tests unitaires écrits pour le noyau Rust et parser de montants frontend ; CI déclarée.
- Documentation de base réparée, un sprint 05 ajouté, références fiscales officielles listées.

## Blocages et limites

- Le noyau Rust compile et ses 8 tests ont réussi sur Windows, mais la compilation du workspace complet et le build natif échouent avant compilation de l'application, car `perl` est introuvable pendant la compilation OpenSSL vendored. Aucun claim de build natif réussi n'est fait.
- `npm install` et `npm test` ont réussi sur Windows avant le dernier correctif (3 tests frontend réussis). Le `typecheck` et le build frontend échouaient ; ils doivent être rejoués après la mise à niveau de Vitest dans cette révision.
- Le build natif doit confirmer SQLCipher/OpenSSL et le comportement sur mauvaise clé avant des données réelles. Le prérequis manquant constaté sur le poste Windows est Strawberry Perl dans le `PATH`.
- La couverture SQLCipher, l'audit atomique de toutes les mutations, les sauvegardes/restaurations et les tests de reprise restent à renforcer.
- Le plan de comptes n'est qu'un socle initial. Pas d'import bancaire, FEC fiable, clôture, balance, lettrage, contre-passation, bail, quittance, CA3, 2072, 2044 ou liasse 2065 implémentés.

## Dette technique identifiée

- Le frontend propose dans cette alpha une saisie d'écriture simplifiée à deux lignes.
- Le journal d'audit n'est pas encore transactionnel avec toutes les mutations.
- Les tests d'intégration SQLCipher et migration ne sont pas encore présents.
- Les règles de compatibilité de régimes sont des avertissements de paramétrage, pas un moteur juridique.
- Il manque le verrouillage de période et la contre-passation des écritures validées.
- Immutabilité et audit append-only non appliqués par triggers SQL ; échecs de déverrouillage non tracés.
- Pas de `Cargo.lock` / `package-lock.json` ; CI sans clippy ni `npm ci`.
- `EventBus` du noyau non branché ; `App.tsx` monolithique ; composants/thèmes de la spécification du sprint 00 non découpés.

## Journal des modifications

| Date | Sprint | Action | Auteur |
|---|---|---|---|
| 2026-10-12 | 00/01 | Première tranche source : Tauri, UI, noyau, société, journal équilibré, chiffrement configuré | OpenAI |
| 2026-10-12 | Docs | Correction des wrappers Markdown, noms de projet, modèle TVA et ajout des sources officielles | OpenAI |
| 2026-10-12 | Revue | Audit docs↔code, tests partiels, réparations documentaires et CI | Claude |
| 2026-10-12 | Correctif post-Windows | Suppression import TypeScript inutilisé, migration Vitest vers 4.1.11 pour résoudre le conflit de types Vite 6/Vitest 2, mise à jour du handoff à partir des logs réels ; compilation corrigée non encore revalidée dans l’environnement de préparation | OpenAI |
