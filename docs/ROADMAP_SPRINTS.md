# Roadmap des sprints — SuperPower Alpha

Cette roadmap est une séquence de livraison, pas une promesse de calendrier. Les sprints fondateurs sont en cours ; les autres modules ne sont pas implémentés dans l'alpha actuelle.

## Vue d'ensemble

| # | Sprint | Objectif | Dépendances | Statut |
|---|---|---|---|---|
| 00 | Bootstrap | Setup projet, CI, thèmes, layout | — | IN_PROGRESS |
| 01 | Noyau | Entités, partie double, événements, stockage chiffré | 00 | IN_PROGRESS |
| 02 | Moteur de tâches | Planification durable, retries, idempotence | 01 | TODO |
| 03 | Import bancaire | CSV, catégorisation, rapprochement | 01, 02 | TODO |
| 04 | Documents | Extraction, signaux, classification | 01, 02 | TODO |
| 05 | Gestion locative | Biens, lots, baux, loyers, TVA par opération, échéances | 01, 02, 03 | TODO |
| 06 | Trésorerie | Prévisionnel, rapprochement réel | 03, 05 | TODO |
| 07 | Fiscalité | TVA, préparation des déclarations adaptées au dossier, veille et FEC | 01, 05, 06 | TODO |
| 08 | Sync mobile | Fichiers chiffrés, watcher, résolution des conflits | 01, 02 | TODO |
| 09 | Stocks | Articles, inventaire, valorisation (si besoin confirmé) | 01 | TODO |

## Chemin critique

```text
00 → 01 → 02 → 03 → 05 → 06 → 07
          ├────→ 04
          └────→ 08
01 ─────────────→ 09 (parallélisable)
```

## Principes de découpage

- Chaque sprint doit produire une démo reproductible et des tests.
- Les migrations de données doivent être versionnées et réversibles lorsque cela est possible.
- Le stockage, la comptabilité et la fiscalité doivent disposer de validations côté Rust, non uniquement dans l'interface.
- Une règle fiscale en production doit contenir sa source, sa date de vérification, sa période de validité et ses hypothèses.
- Les sprints 03–07 constituent le cœur opérationnel ; les sprints 08–09 sont des extensions après validation du besoin.
- Ne pas intégrer une dépendance de moteur de tâches sans test de compilation, persistance, idempotence et reprise après arrêt.

## Définition de Done

Un sprint est terminé uniquement si :

- [ ] Toutes les user stories du sprint sont couvertes.
- [ ] Tests unitaires et d'intégration réussis.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace` passent.
- [ ] `npm test`, `npm run typecheck` et `npm run build` passent.
- [ ] Tests manuels des parcours et erreurs de sécurité documentés.
- [ ] Documentation et `PROGRESS.md` à jour.
- [ ] Revue du diff et validation avant merge.

La couverture visée est ≥ 80 % sur le nouveau code métier critique ; cette cible n'est pas déclarée atteinte dans l'alpha.

## Convention de branches

```text
main
├── sprint/00-bootstrap
├── sprint/01-noyau
├── sprint/02-moteur-taches
└── ...
```

Chaque sprint vise une branche et une pull request. Commits conseillés :

```text
[SPRINT-XX] feat(scope): description
[SPRINT-XX] fix(scope): description
[SPRINT-XX] test(scope): description
[SPRINT-XX] docs(scope): description
[SPRINT-XX] chore(scope): description
```

## Gestion des blocages

1. Consigner la cause dans `PROGRESS.md`.
2. Créer une issue avec reproduction minimale si le dépôt est publié.
3. Ne pas contourner un blocage de sécurité en retirant une validation ou le chiffrement.
4. Après correction, ajouter un test de non-régression.
