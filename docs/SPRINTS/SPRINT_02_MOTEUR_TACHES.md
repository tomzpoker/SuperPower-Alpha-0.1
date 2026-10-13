# Sprint 02 — Moteur de Tâches

## Métadonnées
| Champ | Valeur |
|---|---|
| **Numéro** | 02 |
| **Nom** | Moteur de Tâches |
| **Durée estimée** | 3–4 jours |
| **Dépendances** | Sprint 01 |
| **Branche Git** | `sprint/02-moteur-taches-prototype` |
| **Statut** | IN_PROGRESS — persistance, IPC/UI et worker one-shot présents sur la branche ; runtime de fond et cron restent à faire |

## Contexte hérité
> À remplir après sprint 01 : entités, compta, event bus, UI sociétés/écritures.

## Objectifs
1. Prototyper puis choisir le moteur de tâches (piste historique : `flexiq-core`, **absent du dépôt** ; critères : persistance SQLite, idempotence, reprise après arrêt) et l'isoler dans une crate `task-engine`.
2. Définir les types de tâches du noyau.
3. Permettre aux modules d'enregistrer leurs handlers.
4. Exposer une UI de visualisation des tâches (liste + calendrier).
5. Reprendre et adapter le widget Tom-Alpha-0.1.

## User Stories
- US-02-01 : Visualiser les tâches (liste, calendrier).
- US-02-02 : Créer/modifier/supprimer une tâche manuellement.
- US-02-03 : Planifier une tâche récurrente (cron).
- US-02-04 : Voir l'historique d'exécution et les erreurs.

## Tâches techniques
- T-02-01 : Créer `crates/task-engine`.
- T-02-02 : Intégrer `flexiq-core`.
- T-02-03 : Migrations SQLite pour les tâches.
- T-02-04 : Commandes Tauri (`list_tasks`, `create_task`, `run_task`, ...).
- T-02-05 : UI Liste + Calendrier (FullCalendar).
- T-02-06 : Migrer le widget Tom-Alpha-0.1 de PostgreSQL à SQLite.

## État après sprint (handoff)

### Tranche 1 — prototype du moteur persistant (branche de travail)

- La crate crates/task-engine existe comme crate isolée et utilise rusqlite compilé avec SQLCipher ; aucune base annexe en clair n'est créée.
- Le prototype persiste les définitions de tâches ponctuelles, les exécutions et chaque tentative ; les clés d'idempotence préviennent les doublons.
- Les exécutions en cours portent un bail. Après interruption, un bail expiré est inscrit comme tentative échouée puis peut être repris ; les tentatives épuisées deviennent terminales.
- flexiq-core est absent du dépôt et n'est pas ajouté sans prototype ni vérification de compatibilité.
- La migration transactionnelle v3→v4 est maintenant câblée dans `src-tauri/src/database.rs` et ajoute les tables `task_definitions`, `task_executions` et `task_attempts`. Elle conserve les tables existantes, versionne le schéma et possède un test de conservation d'un flux v3 ; le dernier changement doit encore passer les contrôles.
- Les commandes Tauri (liste/création/statut/historique) et la page UI Tâches sont ajoutées.
- Une nouvelle tranche ajoute un registre de handlers thread-safe et un worker one-shot : matérialisation idempotente des tâches à échéance, exécution hors transaction, historisation du résultat et planification des nouvelles tentatives. Le résultat CI de cette tranche doit être confirmé avant intégration.
- Le démarrage automatique du worker dans Tauri, les handlers métier réels, cron, calendrier, édition complète et reprise du widget historique restent à faire.

### Contrôles avant clôture

- [ ] Rejouer cargo fmt --all -- --check sur le commit de migration.
- [ ] Rejouer cargo test -p task-engine et cargo test --workspace sur le commit de migration.
- [ ] Confirmer le build Tauri Windows et vérifier la migration v3 → v4, le schéma S06 préservé et le refus des versions futures.
- [ ] Vérifier les tests sur fichiers SQLCipher réouverts : idempotence, reprise sur expiration de bail et historique des erreurs.
- [x] Exposer les opérations de consultation/création/statut/historique via IPC (CI verte sur le commit précédent).
- [x] Ajouter une première API de registre de handlers et un worker one-shot ; le nouveau commit doit être validé par la CI.
- [ ] Intégrer le démarrage/arrêt du worker à Tauri, des handlers métier réels et le calcul/validation cron.
- [ ] Ajouter le calendrier, l'édition complète et l'adaptation du widget historique avant de considérer les user stories livrées.

### Résultats locaux (première version, avant migration v3→v4)

L'utilisateur a confirmé `cargo fmt --all -- --check` sans erreur, 3/3 tests `task-engine` réussis et 29/29 tests du workspace réussis. Ces résultats concernent le commit avant le câblage de la migration ; ils ne valident pas le tout dernier commit. Un avertissement d'import inutilisé a été corrigé ensuite.
