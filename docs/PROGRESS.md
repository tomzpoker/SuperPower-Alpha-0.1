# PROGRESS — SuperPower Alpha

> Source de vérité : distinguer le code présent du code compilé/testé. Une fonctionnalité ne passe à \`DONE\` qu'après validation locale/CI, tests de non-régression et revue.

## État actuel
| Champ | Valeur |
|---|---|
| Version | \`0.1.0-alpha.1\` |
| Main de référence | \`5078de2\` — fusion du cache CI Windows |
| CI de main | 3/3 verts, confirmation du propriétaire le 2026-10-13 |
| Lot en cours | S06-A — flux prévisionnels et réalisations guidées |
| Branche | \`feat/guided-cash-flows\` |
| Statut | IN_PROGRESS — tests de cette branche à confirmer |
| Prochaine action | Vérifier la CI de la PR et tester la migration v1→v2 sur copie d'une base native avant merge |
| Reprise | \`AGENTS.md\` → \`docs/HANDOFF.md\` |
| Dépôt | https://github.com/tomzpoker/SuperPower-Alpha-0.1 |

## Sprints
| # | Nom | Statut | Fichier | Notes |
|---|---|---|---|---|
| 00 | Bootstrap | IN_PROGRESS | \`SPRINTS/SPRINT_00_BOOTSTRAP.md\` | Coffre et CI disponibles ; vérification sécurité complète restante |
| 01 | Noyau | IN_PROGRESS | \`SPRINTS/SPRINT_01_NOYAU.md\` | Sociétés/écritures simples ; PCG complet et clôture non livrés |
| 02 | Moteur de tâches | TODO | \`SPRINTS/SPRINT_02_MOTEUR_TACHES.md\` | Prototype durable/idempotent à décider |
| 03 | Import bancaire | TODO | \`SPRINTS/SPRINT_03_IMPORT_BANCAIRE.md\` | CSV/rapprochement non livrés |
| 04 | Documents | TODO | \`SPRINTS/SPRINT_04_DOCUMENTS.md\` | Pipeline non livré |
| 05 | Gestion locative | TODO | \`SPRINTS/SPRINT_05_GESTION_LOCATIVE.md\` | Biens/baux/TVA par opération non livrés |
| 06 | Trésorerie | IN_PROGRESS | \`SPRINTS/SPRINT_06_TRESORERIE.md\` | S06-A : prévu/réalisé ; projection et rapprochement futurs |
| 07 | Fiscalité | TODO | \`SPRINTS/SPRINT_07_FISCALITE.md\` | Déclarations non livrées |
| 08 | Sync mobile | TODO | \`SPRINTS/SPRINT_08_SYNC_MOBILE.md\` | Non livré |
| 09 | Stocks | TODO | \`SPRINTS/SPRINT_09_STOCKS.md\` | Extension facultative |

## Vérifications connues avant S06-A
| Contrôle | Résultat connu | Limite |
|---|---|---|
| \`cargo fmt --all -- --check\` | OK sur main précédent | À rejouer sur S06-A |
| \`cargo test -p superpower-core\` | 8/8 sur main précédent | Tests de flux nouveaux non validés |
| \`cargo check --workspace\` | OK sur main précédent | Pas une validation du nouveau lot |
| \`npm run typecheck\` | OK sur main précédent | À rejouer avec l'interface trésorerie |
| GitHub Actions main | 3/3 verts, confirmation du propriétaire | La branche S06-A doit repasser les checks |
| Migration SQLCipher v1→v2 | Test ajouté dans la branche ; copie native à vérifier | Le résultat main ne prouve pas la migration |

## Livré dans main
- Application native Tauri + React/TypeScript + noyau Rust.
- Coffre local SQLCipher/Argon2id.
- Sociétés multi-entités, socle de comptes et saisie technique d'écritures équilibrées.
- CI Windows avec cache Rust et logs détaillés ; 3/3 verts confirmés.

## Lot S06-A sur la branche de travail (à valider)
- Domaine Rust de flux et réalisations partielles.
- Migration versionnée \`user_version 1 → 2\), ajout de tables sans reconstruction des tables v1.
- Commandes Tauri de liste/création/modification/annulation/réalisation, audit transactionnel.
- Écran Trésorerie « Prévisionnel et réalisé », sans choix de comptes, avec écart, état et historique.
- Mode navigateur cohérent mais volatil.
- Spécification : [Sprint 06](SPRINTS/SPRINT_06_TRESORERIE.md), [ADR-0001](DECISIONS/ADR-0001-flux-previsionnels.md).

## Blocages et limites
- S06-A ne crée pas encore d'écritures comptables ni de TVA.
- Pas de récurrences, projection 12/24/36 mois, alertes ou rapprochement bancaire automatique.
- Pas encore de baux, import bancaire, grand livre/balance, FEC validé, clôture ou fiscalité automatisée.
- Le plan de comptes est un socle illustratif limité.
- Triggers d'immutabilité/append-only, sauvegarde/restauration chiffrée, audit sécurité complet et validation de mauvaise clé à renforcer.
- \`App.tsx\` reste monolithique ; la couverture >80 % n'est pas démontrée.

## Journal des modifications
| Date | Lot | Action | Statut |
|---|---|---|---|
| 2026-10-12 | 00/01 | Première tranche : Tauri, UI, sociétés, écritures, coffre | Sur main |
| 2026-10-12 | CI | Formatage, cache Windows et diagnostics ; 3/3 confirmés ensuite | Sur main |
| 2026-10-13 | 06-A | Parcours prévu/réalisé, migration v2 et documentation métier | Branche de travail, CI à confirmer |
| 2026-10-13 | CI | Le premier contrôle Rust a signalé uniquement des écarts de formatage ; formatage corrigé dans le commit suivant. La nouvelle CI doit confirmer compilation et tests. | À revalider |
