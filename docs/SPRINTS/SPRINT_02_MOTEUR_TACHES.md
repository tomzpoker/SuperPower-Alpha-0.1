# Sprint 02 — Moteur de Tâches

## Métadonnées
| Champ | Valeur |
|---|---|
| **Numéro** | 02 |
| **Nom** | Moteur de Tâches |
| **Durée estimée** | 3–4 jours |
| **Dépendances** | Sprint 01 |
| **Branche Git** | `sprint/02-moteur-taches` |
| **Statut** | IN_PROGRESS — prototype persistant isolé, intégration applicative à faire |

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
- Cette tranche n'est pas encore intégrée au coffre de l'application : elle n'ajoute pas de tables au schéma applicatif, ne change pas PRAGMA user_version, et n'est pas exposée en IPC ou dans l'interface.
- Cron, registre de handlers, commandes Tauri, interface liste/calendrier et reprise du widget historique restent à faire.

### Contrôles avant clôture

- [ ] cargo fmt --all -- --check
- [ ] cargo test -p task-engine
- [ ] cargo test --workspace
- [ ] Vérifier les tests sur fichiers SQLCipher réouverts : idempotence, reprise sur expiration de bail et historique des erreurs.
- [ ] Intégrer les tables dans une migration applicative transactionnelle v3 → v4, puis exposer les opérations via IPC.
- [ ] Ajouter handlers enregistrables, calcul/validation cron et contrôles de planification.
- [ ] Ajouter les commandes et l'interface de tâches (liste, calendrier, historique) avant de considérer les user stories livrées.
