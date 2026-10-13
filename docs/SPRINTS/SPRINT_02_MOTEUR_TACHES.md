# Sprint 02 — Moteur de Tâches

## Métadonnées
| Champ | Valeur |
|---|---|
| **Numéro** | 02 |
| **Nom** | Moteur de Tâches |
| **Durée estimée** | 3–4 jours |
| **Dépendances** | Sprint 01 |
| **Branche Git** | `sprint/02-moteur-taches-prototype` |
| **Statut** | IN_PROGRESS — tâches persistées, cron UTC, worker natif supervisé, premier handler local `general/reminder` ; handlers métiers supplémentaires, fuseau local et essai natif restant à livrer |

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
- Les commandes Tauri (liste/création/modification/suppression contrôlée/statut/historique) et la page UI Tâches sont ajoutées. Une tâche ne peut être modifiée ou supprimée qu'avant sa première exécution ; suppression et audit sont atomiques, et une tâche avec historique doit être annulée pour préserver ses résultats.
- La page dispose maintenant d'un calendrier mensuel en grille, avec navigation entre mois et affichage des échéances par statut ; la liste reste l'endroit prévu pour modifier une tâche et consulter son historique.
- Une nouvelle tranche ajoute un registre de handlers thread-safe et un worker one-shot : matérialisation idempotente des tâches à échéance, exécution hors transaction, historisation du résultat et planification des nouvelles tentatives. La CI #127 est verte sur cette tranche.
- Le changement suivant corrige la réactivation : une tâche annulée avant sa première tentative remet en file sa même occurrence, sans créer de doublon. Un type sans handler échoue de façon visible et conserve l'erreur dans l'historique.
- Les tâches conservent une expression cron numérique à cinq champs et son ancre d'origine ; la migration v4→v5 ajoute les colonnes sans perdre les définitions existantes. Le worker one-shot matérialise les occurrences échues avec une clé idempotente par occurrence, avance l'échéance dans la même transaction et borne le rattrapage à 1 000 occurrences par appel et par tâche.
- Le formulaire natif peut prévisualiser cinq échéances suivant l'ancre via le parseur Rust partagé. L'aperçu refuse les expressions invalides et borne le nombre demandé ; le mode navigateur indique clairement que le calcul nécessite la fenêtre Tauri.
- La CI #151 valide le worker démarré après création/déverrouillage, arrêté avant verrouillage et à la fermeture, ainsi que le handler local `general/reminder`. Le lot courant durcit les erreurs de connexion/verrouillage et met les toasts en file pour éviter d'écraser plusieurs rappels échus. Nouvelle CI requise ; un essai manuel natif sur coffre temporaire reste ouvert.

### Contrôles avant clôture

- [x] `cargo fmt --all -- --check` confirmé par le contrôle Rust de la CI #139 sur le commit `8a1528d`.
- [x] `cargo test -p task-engine` et `cargo test --workspace` confirmés par le build/tests Windows de la CI #139.
- [x] Build Tauri Windows et tests de migration/schéma confirmés par la CI #139.
- [x] Tests SQLCipher, idempotence, reprise de bail et historique des erreurs inclus dans les tests du workspace passés par la CI #139.
- [x] Aperçu cron natif et tests de régression de l'ancre validés par la CI #142 sur le commit `a656e83`.
- [x] Exposer les opérations de consultation/création/statut/historique via IPC (CI verte sur le commit précédent).
- [x] Ajouter une première API de registre de handlers et un worker one-shot ; build Tauri Windows et tests workspace confirmés par la CI #146 sur `881fcd5`.
- [x] Persister l'expression et l'ancre cron dans la migration v4→v5, puis matérialiser idempotemment les occurrences échues dans le worker one-shot.
- [x] Ajouter un parseur cron à cinq champs avec validation stricte et tests unitaires.
- [x] Ajouter un mode de worker limité aux handlers enregistrés, avec tests anti-consommation, anti-famine et sans matérialisation cron si le handler manque.
- [x] Intégration du worker natif stoppable et du handler `general/reminder` validée en CI #151.
- [ ] Valider les durcissements du worker/coffre et la file de rappels UI dans la nouvelle CI ; faire un essai manuel natif sur coffre temporaire.
- [x] Ajouter la vue calendrier mensuelle de base et sa navigation.
- [x] Suppression contrôlée avant toute exécution, avec garde-fou transactionnel et audit atomique.
- [ ] Édition complète du cycle de vie et adaptation du widget historique avant de considérer toutes les user stories livrées.

### Validation CI confirmée

- Le commit `8a1528d` a passé les trois jobs de la CI #139 : formatage/tests Rust, contrôles TypeScript/tests/bundle frontend et build Tauri Windows avec tests du workspace/migrations.
- Le commit `a656e83` a passé les trois jobs de la CI #142, qui valide aussi l'aperçu cron natif et le cas de l'ancre alignée sur une occurrence.
- La suppression contrôlée, le test frontend du mode démo, les migrations v3→v5 et l'ensemble du Sprint 02 actuel ont passé la CI #146 sur `881fcd5` : [résultats complets](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38411970774).
- La CI #151 est entièrement verte sur `8b8c0b5`, avec tests du moteur, frontend, build Windows/Tauri et workspace/migrations. Le commit courant ajoute le durcissement du cycle de vie et la mise en file des rappels ; sa CI doit confirmer ces changements.
