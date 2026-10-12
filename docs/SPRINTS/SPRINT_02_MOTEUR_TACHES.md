# Sprint 02 — Moteur de Tâches

## Métadonnées
| Champ | Valeur |
|---|---|
| **Numéro** | 02 |
| **Nom** | Moteur de Tâches |
| **Durée estimée** | 3–4 jours |
| **Dépendances** | Sprint 01 |
| **Branche Git** | `sprint/02-moteur-taches` |
| **Statut** | TODO |

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
> À remplir.