# PROGRESS — SuperPower Alpha

> Source de vérité : distinguer le code présent du code compilé/testé. Une fonctionnalité ne passe à `DONE` qu'après validation CI, parcours métier et revue.

## État actuel
| Champ | Valeur |
|---|---|
| Version | `0.1.0-alpha.1` |
| Main de référence | `5078de2` — socle précédent |
| Lot en cours | S06-A — flux prévisionnels et réalisations guidées |
| Branche | `feat/guided-cash-flows` |
| Statut | IN_PROGRESS — CI après renforcement du test de migration à confirmer ; essai natif manuel restant |
| Prochaine action | Vérifier la CI du dernier commit, puis tester l'application et la migration sur une copie de base v1 SQLCipher |
| Reprise | `AGENTS.md` → `docs/HANDOFF.md` |
| Dépôt | https://github.com/tomzpoker/SuperPower-Alpha-0.1 |

## Sprints
| # | Nom | Statut | Fichier | Notes |
|---|---|---|---|---|
| 00 | Bootstrap | IN_PROGRESS | `SPRINTS/SPRINT_00_BOOTSTRAP.md` | Coffre et CI disponibles ; revue sécurité complète restante |
| 01 | Noyau | IN_PROGRESS | `SPRINTS/SPRINT_01_NOYAU.md` | Sociétés et écritures simples ; PCG complet et clôture non livrés |
| 02 | Moteur de tâches | TODO | `SPRINTS/SPRINT_02_MOTEUR_TACHES.md` | Prototype durable/idempotent à décider |
| 03 | Import bancaire | TODO | `SPRINTS/SPRINT_03_IMPORT_BANCAIRE.md` | CSV/rapprochement non livrés |
| 04 | Documents | TODO | `SPRINTS/SPRINT_04_DOCUMENTS.md` | Pipeline non livré |
| 05 | Gestion locative | TODO | `SPRINTS/SPRINT_05_GESTION_LOCATIVE.md` | Biens/baux/TVA par opération non livrés |
| 06 | Trésorerie | IN_PROGRESS | `SPRINTS/SPRINT_06_TRESORERIE.md` | S06-A : prévu/réalisé ; projection et rapprochement restent à faire |
| 07 | Fiscalité | TODO | `SPRINTS/SPRINT_07_FISCALITE.md` | Déclarations non livrées |
| 08 | Sync mobile | TODO | `SPRINTS/SPRINT_08_SYNC_MOBILE.md` | Non livré |
| 09 | Stocks | TODO | `SPRINTS/SPRINT_09_STOCKS.md` | Extension facultative |

## Validation automatisée
Le run GitHub Actions `38316144097` a validé les trois tâches de CI du code S06-A précédent : formatage/tests Rust, vérifications frontend et build/test natif Windows. Le test de migration a ensuite été renforcé afin d'utiliser le schéma v1 complet avec une société, une écriture, ses lignes et une entrée d'audit préexistantes. Le formatage de ce nouveau test a été corrigé ; la CI de la dernière révision doit confirmer le résultat.

| Contrôle | Portée et état |
|---|---|
| `cargo fmt --all -- --check` | Passé sur la révision précédente ; revalidation en cours après ajout du test de préservation |
| Tests Rust du domaine | Passés sur la révision précédente ; nouveaux tests à confirmer dans la dernière CI |
| Tests workspace / migration | Test de préservation ajouté ; résultat de la dernière CI à confirmer |
| Frontend | Typecheck, tests et bundle de production passés sur la révision précédente ; revalidation en cours |
| Build Tauri Windows | Passé sur la révision précédente ; revalidation en cours |
| Vérification Markdown et liens | Passée sur la révision précédente ; revalidation en cours |
| Migration sur une vraie copie SQLCipher v1 | **À faire manuellement** avant fusion |
| Parcours de l'interface native | **À faire manuellement** avant de considérer le lot livré |

## Livré dans main
- Application native Tauri + React/TypeScript + noyau Rust.
- Coffre local SQLCipher/Argon2id.
- Gestion multi-entités des sociétés, socle de comptes et saisie technique d'écritures équilibrées.
- CI Windows, cache Rust et diagnostics.

## Lot S06-A sur la branche de travail
- Domaine Rust pour flux prévisionnels et réalisations partielles.
- Migration versionnée `user_version 1 → 2`, transactionnelle et sans reconstruction des tables v1.
- Commandes Tauri de liste/création/modification/annulation/réalisation avec audit transactionnel.
- Écran Trésorerie « Prévisionnel et réalisé », sans saisie de comptes, avec écart, états et historique.
- Mode aperçu navigateur cohérent mais volatil.
- Spécification : [Sprint 06](SPRINTS/SPRINT_06_TRESORERIE.md), [ADR-0001](DECISIONS/ADR-0001-flux-previsionnels.md).

## Limites / risques connus
- S06-A ne crée pas encore d'écritures comptables ni de TVA.
- Récurrences, projection 12/24/36 mois, alertes et rapprochement bancaire automatique non livrés.
- Baux, import bancaire, grand livre/balance, FEC validé, clôture et fiscalité automatisée non livrés.
- Plan de comptes limité au socle illustratif.
- Triggers d'immutabilité/append-only, sauvegarde/restauration chiffrée, audit sécurité complet et tests de mauvaise clé à renforcer.
- `App.tsx` reste monolithique ; la couverture supérieure à 80 % n'est pas démontrée.

## Journal des modifications
| Date | Lot | Action | Statut |
|---|---|---|---|
| 2026-10-12 | 00/01 | Première tranche : Tauri, UI, sociétés, écritures, coffre | Sur main |
| 2026-10-12 | CI | Formatage, cache Windows et diagnostics | Sur main |
| 2026-10-13 | 06-A | Prévu/réalisé, migration v2 et documentation métier | Branche de travail, validation manuelle restante |
| 2026-10-13 | 06-A | Renforcement du test de migration pour confirmer la conservation des données v1 | CI de la dernière révision à confirmer |
