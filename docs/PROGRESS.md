# PROGRESS — SuperPower Alpha

> Source de vérité : distinguer le code présent du code compilé/testé. Une fonctionnalité ne passe à `DONE` qu'après validation CI, parcours métier et revue.

## État actuel
| Champ | Valeur |
|---|---|
| Version | `0.1.0-alpha.1` |
| Main de référence | `17966e1` — S06-B fusionné ; CI de la PR #5 verte |
| Lot en cours | Sprint 02 — prototype du moteur de tâches persistant |
| Branche | `sprint/02-moteur-taches-prototype` (prototype en cours ; non fusionné) |
| Statut | S06-B est dans `main`. La branche S02 contient le moteur persistant, cron UTC, worker natif stoppable et handler local `general/reminder`. La CI #152 est verte sur `8ac4417`. Le lot courant ajoute un champ de message accessible pour les rappels, avec conservation des autres données JSON ; CI à repasser. |
| Prochaine action | Valider le champ de message et ses tests, puis effectuer un essai natif isolé : rappel à échéance, historique réussi, verrouillage/déverrouillage. Ensuite, traiter un handler métier idempotent pour documents ou échéances locatives. |
| Reprise | `AGENTS.md` → `docs/HANDOFF.md` |
| Dépôt | https://github.com/tomzpoker/SuperPower-Alpha-0.1 |

## Sprints
| # | Nom | Statut | Fichier | Notes |
|---|---|---|---|---|
| 00 | Bootstrap | IN_PROGRESS | `SPRINTS/SPRINT_00_BOOTSTRAP.md` | Coffre et CI disponibles ; revue sécurité complète restante |
| 01 | Noyau | IN_PROGRESS | `SPRINTS/SPRINT_01_NOYAU.md` | Sociétés et écritures simples ; PCG complet et clôture non livrés |
| 02 | Moteur de tâches | IN_PROGRESS | `SPRINTS/SPRINT_02_MOTEUR_TACHES.md` | Migrations v3→v5, page liste/calendrier, cron persisté et worker one-shot ; worker de fond et handlers métier non livrés |
| 03 | Import bancaire | TODO | `SPRINTS/SPRINT_03_IMPORT_BANCAIRE.md` | CSV/rapprochement non livrés |
| 04 | Documents | TODO | `SPRINTS/SPRINT_04_DOCUMENTS.md` | Pipeline non livré |
| 05 | Gestion locative | TODO | `SPRINTS/SPRINT_05_GESTION_LOCATIVE.md` | Biens/baux/TVA par opération non livrés |
| 06 | Trésorerie | S06-A et S06-B fusionnés ; validation native restante | `SPRINTS/SPRINT_06_TRESORERIE.md` et `SPRINTS/SPRINT_06B_RECURRENCE.md` | Récurrences intégrées dans `main` ; migration v2→v3 et essai métier natif à confirmer |
| 07 | Fiscalité | TODO | `SPRINTS/SPRINT_07_FISCALITE.md` | Déclarations non livrées |
| 08 | Sync mobile | TODO | `SPRINTS/SPRINT_08_SYNC_MOBILE.md` | Non livré |
| 09 | Stocks | TODO | `SPRINTS/SPRINT_09_STOCKS.md` | Extension facultative |

## Validation automatisée

Le run [#40](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38326327437) a validé S06-A. Pour S06-B, le run de PR [#96](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38332638917) est vert sur `cd764cb`. Le run post-fusion [#97](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38333882817) est également terminé avec succès sur `17966e1`.

| Contrôle | Portée et état |
|---|---|
| `cargo fmt --all -- --check` | S06-B passé sur le run #96 |
| Tests Rust du domaine | S06-B passé sur le run #96 et le run post-fusion #97 |
| Tests workspace / migrations automatisées | CI #146 verte sur le commit `881fcd5`, incluant le workspace et les migrations ; une nouvelle CI est requise pour le test de panique ajouté au lot de reprise |
| Frontend | Typecheck, tests et bundle S06-B passés sur le run #96 et le run post-fusion #97 |
| Build Tauri Windows | CI #146 verte sur le commit `881fcd5` ; revalidation complète requise après le nouveau test de panique |
| Vérification Markdown et liens | Passée en CI #146 sur le commit `881fcd5` ; documentation de reprise mise à jour dans le lot courant |
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

## Sprint 06-B livré dans main
- Fréquences ponctuelle, mensuelle, trimestrielle et annuelle.
- Génération idempotente et horizon glissant de 36 mois, sans dérive de calendrier.
- Arrêt d'une série distinct de l'annulation d'une occurrence ; historique conservé.
- Migration SQLCipher v2→v3 intégrée ; validation native sur copie isolée requise.

## Limites / risques connus
- Sur la branche S02, la CI #152 valide le worker local `general/reminder` sur `8ac4417`, ainsi que Rust, frontend, build Windows et migrations. Le champ de message accessible et ses tests sont dans le lot courant ; nouvelle validation requise. Les handlers documents/import bancaire/locatif, le fuseau cron configurable, l'adaptation du widget et l'essai natif restent à faire.
- S06-A ne crée pas encore d'écritures comptables ni de TVA.
- S06-B est livré dans `main` par la PR [#5](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/5). La fenêtre glissante de 36 mois part d'aujourd'hui ; une première échéance hors fenêtre est conservée et les suivantes sont générées lorsque l'horizon l'atteint. Cela ne remplace pas une projection financière dédiée.
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

| 2026-10-13 | 06-B | Fréquences ponctuelle/mensuelle/trimestrielle/annuelle, générateur idempotent et migration v2→v3 | Fusionné dans main (commit `17966e1`) ; CI PR #96 verte ; validation native migration à faire |

| 2026-10-13 | 02 | Prototype du registre de tâches/exécutions avec idempotence et reprise par bail | Branche `sprint/02-moteur-taches-prototype`, validation automatisée à réaliser avant intégration |

| 2026-10-13 | 02 | Migrations v3→v5, cron UTC, worker one-shot et suppression contrôlée | CI #149 entièrement verte sur `5d249e8` |
| 2026-10-13 | 02 | Worker natif stoppable et handler local `general/reminder` | CI #151 entièrement verte sur `8b8c0b5` |

| 2026-10-13 | 02 | Durcissement des erreurs worker/coffre et mise en file des rappels UI | CI #152 entièrement verte sur `8ac4417` |
| 2026-10-13 | 02 | Champ de message dédié pour les rappels et tests de transformation du payload | Commit courant : CI requise |

| 2026-10-13 | 02 | Ajout des commandes Tauri de création/liste/statut/historique et d'une page UI Tâches | Branche non fusionnée ; CI du dernier commit à confirmer |
