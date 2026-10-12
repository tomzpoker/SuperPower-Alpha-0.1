# Sprint 03 — Import Bancaire

## Métadonnées
| Champ | Valeur |
|---|---|
| **Numéro** | 03 |
| **Nom** | Import Bancaire |
| **Durée estimée** | 4–5 jours |
| **Dépendances** | Sprints 01, 02 |
| **Branche Git** | `sprint/03-import-bancaire` |
| **Statut** | TODO |

## Contexte hérité
> Noyau + compta + tâches opérationnels.

## Objectifs
1. Importer un CSV bancaire (mapping configurable par banque).
2. Créer des transactions bancaires.
3. Catégoriser automatiquement (règles regex + historique).
4. UI Inbox de validation rapide.
5. Rapprochement transaction ↔ écriture.

## User Stories
- US-03-01 : Configurer un mapping CSV par banque.
- US-03-02 : Importer un CSV et voir les transactions.
- US-03-03 : Catégoriser automatiquement (règles).
- US-03-04 : Valider en masse dans l'Inbox.
- US-03-05 : Créer une règle de catégorisation.

## Tâches techniques
- T-03-01 : Parser CSV (csv crate + serde).
- T-03-02 : Modèle `TransactionBancaire` + migrations.
- T-03-03 : Moteur de règles (regex, priorité).
- T-03-04 : Suggestion basée sur l'historique.
- T-03-05 : UI Import (drag & drop).
- T-03-06 : UI Inbox (validation rapide).
- T-03-07 : UI Règles.

## État après sprint (handoff)
> À remplir.