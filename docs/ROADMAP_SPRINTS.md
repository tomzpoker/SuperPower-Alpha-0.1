# Roadmap des sprints — SuperPower Alpha

Séquence de livraison, pas promesse de calendrier. Les sprints fondateurs restent à valider selon leur définition de Done.

## Vue d'ensemble
| # | Sprint | Objectif | Dépendances | Statut |
|---|---|---|---|---|
| 00 | Bootstrap | Setup projet, CI, thèmes, coffre local | — | IN_PROGRESS |
| 01 | Noyau | Entités, partie double, stockage chiffré | 00 | IN_PROGRESS |
| 02 | Moteur de tâches | Planification durable, retries, idempotence | 01 | TODO |
| 03 | Import bancaire | CSV, catégorisation, rapprochement | 01, 02 | TODO |
| 04 | Documents | Extraction, classification | 01, 02 | TODO |
| 05 | Gestion locative | Biens, lots, baux, loyers, TVA par opération | 01, 02, 03 | TODO |
| 06 | Trésorerie | Prévisionnel/réalisé puis projection et rapprochement | 01 pour S06-A ; 03/05 pour rapprochement | IN_PROGRESS |
| 07 | Fiscalité | TVA, déclarations préparatoires, veille et FEC | 01, 05, 06 | TODO |
| 08 | Sync mobile | Fichiers chiffrés, résolution de conflits | 01, 02 | TODO |
| 09 | Stocks | Articles, inventaire, valorisation | 01 | TODO |

## Lot courant : S06-A — flux métiers guidés
Le premier incrément ajoute les prévisions ponctuelles, les réalisations partielles, l'écart, l'annulation auditée et la migration SQLite/SQLCipher v1→v2. L'utilisateur choisit le sens, la catégorie de pilotage, la date et le montant sans sélectionner de compte PCG.

S06-A ne crée volontairement aucune écriture comptable ni TVA. Une catégorie ne suffit pas à déterminer le compte, la ventilation d'un emprunt ou l'exigibilité TVA. Voir [Sprint 06](SPRINTS/SPRINT_06_TRESORERIE.md) et [ADR-0001](DECISIONS/ADR-0001-flux-previsionnels.md).

## Chemin critique
\`\`\`text
00 → 01 → 02 → 03 → 05 → 06 → 07
          ├────→ 04
          └────→ 08
01 ─────────────→ 09 (parallélisable)
Lot UX S06-A : 01 → S06-A → rapprochement/import (03) et gestion locative (05)
\`\`\`

## Principes
- Chaque sprint produit une démo reproductible et des tests.
- Les migrations sont versionnées, transactionnelles et testées sur copie ; préserver les données existantes.
- Toute règle comptable/fiscale est validée côté Rust et sourcée.
- Tester persistance, idempotence et reprise avant de choisir une dépendance de moteur de tâches.
- Aucune donnée ne quitte la machine sans action explicite.

## Définition de Done
- [ ] User stories couvertes.
- [ ] Tests unitaires/intégration réussis.
- [ ] \`cargo fmt --all -- --check\`, \`cargo clippy --workspace --all-targets -- -D warnings\`, \`cargo test --workspace\` réussis.
- [ ] \`npm test\`, \`npm run typecheck\), \`npm run build\` réussis.
- [ ] Tests manuels et sécurité documentés.
- [ ] Documentation et \`PROGRESS.md\` à jour.
- [ ] Diff relu et validé avant merge.

La couverture visée est ≥80 % sur le nouveau code métier critique ; elle n'est pas déclarée atteinte.

## Convention de branches
\`\`\`text
main
├── sprint/00-bootstrap
├── sprint/01-noyau
├── sprint/02-moteur-taches
└── feat/guided-cash-flows (lot UX S06-A)
\`\`\`
Commits conseillés : \`[SPRINT-XX] feat(scope): description\`, \`fix\`, \`test\`, \`docs\`, \`chore\`.

## Blocages
1. Consigner dans \`PROGRESS.md\`.
2. Créer une issue avec reproduction minimale.
3. Ne pas contourner un problème de sécurité en supprimant une validation ou le chiffrement.
4. Ajouter un test de non-régression après correction.
