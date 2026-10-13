# PROGRESS — SuperPower Alpha

> Source de vérité : distinguer le code présent du code compilé/testé. Une fonctionnalité ne passe à `DONE` qu'après validation CI, parcours métier et revue.

## État actuel
| Champ | Valeur |
|---|---|
| Version | `0.1.0-alpha.1` |
| Main de référence | `41d26e6` — Sprint 06-A fusionné, CI verte |
| Lot en cours | S06-B — spécification des flux récurrents, avant développement |
| Branche | `main` ; branche de développement S06-B à créer |
| Statut | S06-A fusionné ; CI main #40 entièrement verte ; vérification native migration et test ciblé de refus métier encore ouverts. S06-B spécifié, pas implémenté. |
| Prochaine action | Terminer les contrôles résiduels S06-A en environnement isolé, puis développer `docs/SPRINTS/SPRINT_06B_RECURRENCE.md` |
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
| 06 | Trésorerie | S06-A fusionné ; S06-B spécifié | `SPRINTS/SPRINT_06_TRESORERIE.md` et `SPRINTS/SPRINT_06B_RECURRENCE.md` | S06-A : prévu/réalisé ; récurrence non encore implémentée |
| 07 | Fiscalité | TODO | `SPRINTS/SPRINT_07_FISCALITE.md` | Déclarations non livrées |
| 08 | Sync mobile | TODO | `SPRINTS/SPRINT_08_SYNC_MOBILE.md` | Non livré |
| 09 | Stocks | TODO | `SPRINTS/SPRINT_09_STOCKS.md` | Extension facultative |

## Validation automatisée

Le run GitHub Actions [38326327437](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38326327437) est terminé avec succès sur `main`, commit `41d26e6` : frontend, domaine Rust et build Tauri Windows sont tous verts.

| Contrôle | Portée et état |
|---|---|
| `cargo fmt --all -- --check` | Passé en CI sur `b65085c` |
| Tests Rust du domaine | Passés en CI sur `b65085c` |
| Tests workspace / migration automatisée | Passés en CI sur `b65085c`, dont préservation de données représentatives et idempotence de la migration du schéma |
| Frontend | Typecheck, tests et bundle de production passés en CI sur `b65085c` |
| Build Tauri Windows | Passé en CI sur `b65085c` |
| Vérification Markdown et liens | Passée en CI sur `b65085c` |
| Parcours natif Trésorerie (2026-10-13) | **Réussi sur les scénarios observés** : loyer prévu modifié à 1 400 € sans écraser les 400 € réalisés ; assurance prévue à 240 € ; réalisation de 300 € enregistrée puis annulée depuis l'historique ; les lignes annulées restent visibles avec motif et sont exclues des totaux ; prévision assurance annulée et visible via le filtre |
| Réouverture d'une copie isolée du coffre | **Réussie selon le retour utilisateur** ; les données existantes sont restées visibles après réouverture, sans les fusionner dans le coffre habituel |
| Migration réelle depuis une base dont la version v1 est explicitement vérifiée | **À confirmer** : la copie s'ouvre et se rouvre, mais la valeur initiale de `PRAGMA user_version` n'a pas été relevée indépendamment |
| État `depasse` visible avant annulation | **Confirmé par l'utilisateur** : il a vu l'étiquette « Réalisé au-delà du prévu » lors de la réalisation de 300 € pour une prévision de 240 €, avant d'annuler cette réalisation |
| Refus technique d'une réalisation sur prévision annulée | À vérifier au niveau de la commande métier ; l'interface a masqué les boutons d'action après annulation |
| Vérification directe de `user_version = 2` et des deux tables S06-A | À faire si un outil SQLCipher adapté est disponible ; ne pas ouvrir le coffre original directement |

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
- Récurrences non livrées ; S06-B est spécifié dans [Sprint 06-B](SPRINTS/SPRINT_06B_RECURRENCE.md). La fenêtre de 36 mois concerne la génération initiale des occurrences récurrentes, pas encore une projection financière dédiée.
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
