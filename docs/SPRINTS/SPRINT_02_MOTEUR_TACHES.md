# Sprint 02 — Moteur de Tâches

## Métadonnées
| Champ | Valeur |
|---|---|
| **Numéro** | 02 |
| **Nom** | Moteur de Tâches |
| **Durée estimée** | 3–4 jours |
| **Dépendances** | Sprint 01 |
| **Branche Git** | `sprint/02-moteur-taches-prototype` |
| **Statut** | IN_PROGRESS — moteur persistant, worker natif et handler `general/reminder` ; récurrences ancrées, calendrier accessible, filtres de liste et historique enrichi ; CI #38450618601 verte sur le parent `f744b1b`, nouveau calcul de durée en attente de CI ; validation native des commandes Tauri encore ouverte |

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
- US-02-03 : Planifier une tâche ponctuelle ou récurrente (mensuelle, trimestrielle, annuelle) ; garder cron comme mode avancé.
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
- La migration transactionnelle v3→v4 est câblée dans `src-tauri/src/database.rs` et ajoute les tables `task_definitions`, `task_executions` et `task_attempts`. Elle conserve les tables existantes, versionne le schéma et possède un test de conservation d'un flux v3 ; les migrations v3→v6 sont validées en CI #38445019655.
- Les commandes Tauri (liste/création/modification/suppression contrôlée/statut/historique) et la page UI Tâches sont ajoutées. La liste dispose d’une recherche insensible aux accents sur titre/module/type et de filtres d’état combinables ; un résultat vide propose de réinitialiser les filtres. L’historique d’exécution peut être actualisé sans fermer le panneau, avec rechargement des détails de la tentative ouverte. Une tâche ne peut être modifiée ou supprimée qu'avant sa première exécution ; suppression et audit sont atomiques, et une tâche avec historique doit être annulée pour préserver ses résultats. L’historique affiche maintenant les heures de début/fin des exécutions et tentatives, ainsi que leur durée à partir des horodatages persistés ; pour une tâche en cours, aucune durée finale n’est inventée.
- La page dispose maintenant d'un calendrier mensuel en grille, avec navigation entre mois et affichage des échéances par statut ; la liste reste l'endroit prévu pour modifier une tâche et consulter son historique. Les fréquences métier mensuelle, trimestrielle et annuelle y projettent désormais toutes les occurrences de la grille depuis l’ancre originale ; les tâches en pause/annulées n’affichent pas d’occurrences hypothétiques et cron personnalisé n’affiche que son échéance courante. Chaque occurrence du calendrier est maintenant un bouton accessible au clavier : sa sélection bascule vers la liste, fait défiler jusqu’à la tâche correspondante et affiche la date précise de l’occurrence choisie. Un test frontend vérifie que la sélection émet l’identifiant de la tâche et la date exacte de l’occurrence et que `aria-pressed` reflète l’état choisi.
- Une nouvelle tranche ajoute un registre de handlers thread-safe et un worker one-shot : matérialisation idempotente des tâches à échéance, exécution hors transaction, historisation du résultat et planification des nouvelles tentatives. La CI #127 est verte sur cette tranche.
- Le changement suivant corrige la réactivation : une tâche annulée avant sa première tentative remet en file sa même occurrence, sans créer de doublon. Un type sans handler échoue de façon visible et conserve l'erreur dans l'historique.
- Les tâches conservent une fréquence métier (ponctuelle, mensuelle, trimestrielle, annuelle) ou une expression cron avancée avec ancre d'origine. La migration v5→v6 ajoute la fréquence sans perdre les tâches existantes et reconnaît les anciennes tâches cron.
- Le calcul repart de l'ancre originale : 31 janvier → 28 février → 31 mars ; le cas de l'année bissextile est testé. Le worker conserve une matérialisation idempotente par occurrence.
- Le formulaire natif peut prévisualiser cinq échéances suivant l'ancre via le parseur Rust partagé. L'aperçu refuse les expressions invalides et borne le nombre demandé ; le mode navigateur indique clairement que le calcul nécessite la fenêtre Tauri.
- La CI #153 est entièrement verte sur `78ace82` : le champ « Message du rappel » est synchronisé avec `payload.message`, les autres propriétés JSON sont conservées et le build Windows/Tauri plus les tests workspace/migrations sont passés.

### Contrôles avant clôture

- [x] `cargo fmt --all -- --check` confirmé par le contrôle Rust de la CI #139 sur le commit `8a1528d`.
- [x] `cargo test -p task-engine` et `cargo test --workspace` confirmés par le build/tests Windows de la CI #139.
- [x] Build Tauri Windows et tests de migration/schéma confirmés par la CI #139.
- [x] Tests SQLCipher, idempotence, reprise de bail et historique des erreurs inclus dans les tests du workspace passés par la CI #139.
- [x] Aperçu cron natif et tests de régression de l'ancre validés par la CI #142 sur le commit `a656e83`.
- [x] Exposer les opérations de consultation/création/statut/historique via IPC (CI verte sur le commit précédent).
- [x] Actualiser l’historique depuis son panneau, y compris les détails de la tentative ouverte, sans fermer la vue.
- [x] Afficher les heures de début/fin et les durées des exécutions et tentatives ; tester les durées manquantes, invalides ou inversées sans inventer de valeur.
- [x] Tester dans le mode aperçu les transitions pause/reprise/annulation/réactivation sans créer de définition de tâche en double.
- [x] Ajouter une première API de registre de handlers et un worker one-shot ; build Tauri Windows et tests workspace confirmés par la CI #146 sur `881fcd5`.
- [x] Persister l'expression et l'ancre cron dans la migration v4→v5, puis matérialiser idempotemment les occurrences échues dans le worker one-shot.
- [x] Ajouter un parseur cron à cinq champs avec validation stricte et tests unitaires.
- [x] Ajouter un mode de worker limité aux handlers enregistrés, avec tests anti-consommation, anti-famine et sans matérialisation cron si le handler manque.
- [x] Intégration du worker natif stoppable et du handler `general/reminder` validée en CI #151.
- [x] Durcissements worker/coffre et file des rappels UI validés par CI #152.
- [x] Champ de message de rappel dédié et tests de synchronisation du payload validés en CI #153 sur `78ace82`.
- [x] Sélecteur de fréquence métier ajouté, cron conservé comme mode avancé.
- [x] Migration v5→v6 additive et conservation des tâches existantes testée.
- [x] Confirmer par CI le calcul ancré, le frontend et la migration v5→v6 (CI #38442503451, commit `fa33766`).
- [x] Tester l’intégration des occurrences mensuelles dans le worker : 31 janvier → 28 février → 31 mars, ancre d’origine conservée et répétition sans doublon (CI #38442503451).
- [x] Essai natif sur coffre temporaire : rappel `general/reminder` à échéance, notification visible, exécution `Terminée`, tentative `#1 · Réussie` et historique confirmé par l'utilisateur.
- [x] Test automatisé SQLCipher AppState : verrouillage/déverrouillage, accès refusé pendant le verrouillage, historique conservé et reprise du worker sans double exécution (CI #38445019655).
- [ ] Vérifier manuellement dans l’application native les commandes Tauri verrouillage/déverrouillage et la reprise du worker sur un coffre temporaire.
- [x] Ajouter la vue calendrier mensuelle de base et sa navigation.
- [x] Rendre les occurrences du calendrier sélectionnables au clavier et à la souris, avec bascule vers la tâche correspondante et mise en évidence de la date choisie.
- [x] Suppression contrôlée avant toute exécution, avec garde-fou transactionnel et audit atomique.
- [ ] Édition complète du cycle de vie et adaptation du widget historique avant de considérer toutes les user stories livrées.

### Validation CI confirmée

- Le commit `8a1528d` a passé les trois jobs de la CI #139 : formatage/tests Rust, contrôles TypeScript/tests/bundle frontend et build Tauri Windows avec tests du workspace/migrations.
- Le commit `a656e83` a passé les trois jobs de la CI #142, qui valide aussi l'aperçu cron natif et le cas de l'ancre alignée sur une occurrence.
- La suppression contrôlée, le test frontend du mode démo, les migrations historiques v3→v5 et l'ensemble du Sprint 02 actuel ont passé la CI #146 sur `881fcd5` : [résultats complets](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38411970774).
- La CI #38449610042 est verte sur `50f289e` : le test frontend du calendrier interactif, les six tests de recherche/filtres, les tests Rust, le build Tauri Windows et les tests workspace/migrations sont tous réussis.
- La CI #38445019655 est verte sur `2c13cc8` : formatage/tests Rust, frontend, build Tauri Windows et tests workspace/migrations. Le test SQLCipher AppState verrouille/déverrouille le coffre temporaire, conserve l’historique de janvier/février et vérifie la reprise sans doublon ainsi que la prochaine occurrence au 31 mars. La CI #38438886275 valide séparément start/stop/restart du contrôleur. Le test natif du rappel simple est réussi ; la vérification manuelle des commandes de verrouillage/déverrouillage reste ouverte. [Résultats CI #38445019655](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38445019655).
