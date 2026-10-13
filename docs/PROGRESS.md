# PROGRESS — SuperPower Alpha

> Source de vérité : distinguer le code présent du code compilé/testé. Une fonctionnalité ne passe à `DONE` qu'après validation CI, parcours métier et revue.

## État actuel
| Champ | Valeur |
|---|---|
| Version | `0.1.0-alpha.1` |
| Main de référence | `7f87caf` — S06-A fusionné, PR de préparation S06-B également fusionnées |
| Lot en cours | S06-B — récurrences en cours d'implémentation dans la PR #5 |
| Branche | `feat/s06b-recurrence-domain` → [PR #5](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/5) |
| Statut | S06-A fusionné. S06-B implémenté sur la branche de PR #5 : frontend et Rust verts ; build Tauri Windows/tests workspace et essai natif de migration v2→v3 encore à confirmer. |
| Prochaine action | Confirmer la CI complète du dernier commit de la PR #5, puis tester la migration v2→v3 uniquement sur une copie SQLCipher isolée. |
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
| 06 | Trésorerie | S06-A fusionné ; S06-B en PR #5 | `SPRINTS/SPRINT_06_TRESORERIE.md` et `SPRINTS/SPRINT_06B_RECURRENCE.md` | Formulaire, générateur et persistance de récurrence sur branche ; pas encore validés/fusionnés |
| 07 | Fiscalité | TODO | `SPRINTS/SPRINT_07_FISCALITE.md` | Déclarations non livrées |
| 08 | Sync mobile | TODO | `SPRINTS/SPRINT_08_SYNC_MOBILE.md` | Non livré |
| 09 | Stocks | TODO | `SPRINTS/SPRINT_09_STOCKS.md` | Extension facultative |

## Validation automatisée

Le run [#40](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38326327437) est vert sur le commit S06-A `41d26e6`. Pour S06-B, le run [#88](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38331494863) a confirmé frontend et Rust sur le commit `14e755b`; le build Tauri Windows et les tests workspace étaient encore en cours au dernier relevé.

| Contrôle | Portée et état |
|---|---|
| `cargo fmt --all -- --check` | S06-A passé sur `b65085c` ; S06-B passé sur le run #88 |
| Tests Rust du domaine | S06-A passé sur `b65085c` ; S06-B passé sur le run #88 |
| Tests workspace / migration automatisée | Passés en CI sur `b65085c`, dont préservation de données représentatives et idempotence de la migration du schéma |
| Frontend | S06-A passé sur `b65085c` ; typecheck/tests/bundle S06-B passés sur le run #88 |
| Build Tauri Windows | S06-A passé sur `b65085c` ; S06-B en cours sur le run #88 |
| Vérification Markdown et liens | S06-A passée sur `b65085c` ; S06-B passée sur le run #88 |
| Parcours natif Trésorerie (2026-10-13) | **Réussi sur les scénarios observés** : loyer prévu modifié à 1 400 € sans écraser les 400 € réalisés ; assurance prévue à 240 € ; réalisation de 300 € enregistrée puis annulée depuis l'historique ; les lignes annulées restent visibles avec motif et sont exclues des totaux ; prévision assurance annulée et visible via le filtre |
| Réouverture d'une copie isolée du coffre | **Réussie selon le retour utilisateur** ; les données existantes sont restées visibles après réouverture, sans les fusionner dans le coffre habituel |
| Migration réelle depuis une base dont la version v1 est explicitement vérifiée | **À confirmer** : la copie s'ouvre et se rouvre, mais la valeur initiale de `PRAGMA user_version` n'a pas été relevée indépendamment |
| État `depasse` visible avant annulation | **Confirmé par l'utilisateur** : il a vu l'étiquette « Réalisé au-delà du prévu » lors de la réalisation de 300 € pour une prévision de 240 €, avant d'annuler cette réalisation |
| Refus technique d'une réalisation sur prévision annulée | Test unitaire du garde-fou partagé passé en CI sur la PR #4 ; test d'intégration invoquant la commande avec une prévision annulée en base native encore souhaitable |
| Vérification directe du schéma SQLCipher sur copie isolée | La version source de la copie précédente n'a pas été relevée indépendamment. Pour S06-B, vérifier `user_version = 3`, `flux_recurrences` et `recurrence_id` après migration v2→v3 ; ne jamais utiliser le coffre original comme cible |

Les essais manuels ont utilisé `SUPERPOWER_TEST_DATA_DIR` et des dossiers temporaires. La base source n'a pas été remplacée ni fusionnée avec les données des tests.


## Livré dans main
- Application native Tauri + React/TypeScript + noyau Rust.
- Coffre local SQLCipher/Argon2id.
- Gestion multi-entités des sociétés, socle de comptes et saisie technique d'écritures équilibrées.
- CI Windows, cache Rust et diagnostics.

## Sprint 06-A livré dans main
- Domaine Rust pour flux prévisionnels et réalisations partielles.
- Migration versionnée `user_version 1 → 2`, transactionnelle et sans reconstruction des tables v1.
- Commandes Tauri de liste/création/modification/annulation/réalisation avec audit transactionnel.
- Écran Trésorerie « Prévisionnel et réalisé », sans saisie de comptes, avec écart, états et historique.
- Mode aperçu navigateur cohérent mais volatil.
- Spécification : [Sprint 06](SPRINTS/SPRINT_06_TRESORERIE.md), [ADR-0001](DECISIONS/ADR-0001-flux-previsionnels.md).

## Limites / risques connus
- S06-A ne crée pas encore d'écritures comptables ni de TVA.
- Récurrences en implémentation dans [PR #5](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/5), mais pas encore livrées dans `main`. La fenêtre glissante de 36 mois sert à matérialiser les échéances ; elle ne remplace pas une projection financière dédiée.
- Projection financière dédiée, alertes et rapprochement bancaire automatique non livrés.
- Baux, import bancaire, grand livre/balance, FEC validé, clôture et fiscalité automatisée non livrés.
- Plan de comptes limité au socle illustratif.
- Triggers d'immutabilité/append-only, sauvegarde/restauration chiffrée, audit sécurité complet et tests de mauvaise clé à renforcer.
- `App.tsx` reste monolithique ; la couverture supérieure à 80 % n'est pas démontrée.

## Journal des modifications
| Date | Lot | Action | Statut |
|---|---|---|---|
| 2026-10-12 | 00/01 | Première tranche : Tauri, UI, sociétés, écritures, coffre | Sur main |
| 2026-10-12 | CI | Formatage, cache Windows et diagnostics | Sur main |
| 2026-10-13 | 06-A | Prévu/réalisé, migration v2 et documentation métier | Fusionné dans main ; CI #40 verte ; vérification native de migration encore ouverte |
| 2026-10-13 | 06-A | Renforcement du test automatisé de migration v1 → v2 | CI verte sur main, run #40 |
| 2026-10-13 | 06-A | Parcours natif Trésorerie et copie de coffre isolée | Scénarios observés réussis ; version source de la copie migrée non relevée indépendamment |

| 2026-10-13 | 06-B | Fréquences ponctuelle/mensuelle/trimestrielle/annuelle, générateur idempotent et migration v2→v3 | PR #5 ouverte ; frontend/Rust verts, build Windows/workspace tests et essai natif migration à confirmer |
