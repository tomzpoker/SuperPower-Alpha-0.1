
# Sprint 01 — Noyau

## Métadonnées

| Champ | Valeur |
|---|---|
| **Numéro** | 01 |
| **Nom** | Noyau |
| **Durée estimée** | 5–7 jours |
| **Dépendances** | Sprint 00 |
| **Branche Git** | `sprint/01-noyau` |
| **Statut** | IN_PROGRESS |


## État de livraison au 12 octobre 2026

Ce sprint demeure **IN_PROGRESS**. Les premières structures société, l'API Tauri, un socle de comptes, la validation des écritures équilibrées, le stockage SQLCipher configuré et l'audit initial sont présents. Les builds/test natifs ne sont pas confirmés. La tranche livrée utilise `rusqlite` et un schéma versionné (`PRAGMA user_version`) ; **SQLx n'est pas une dépendance de ce dépôt**. Les cases ci-dessous décrivent le périmètre cible et doivent rester décochées tant que les critères correspondants ne sont pas réellement satisfaits.

## Contexte hérité

> À remplir après le sprint 00. Doit contenir :
> - Le projet Tauri compile et démarre.
> - Le workspace Rust est en place avec les crates vides.
> - Le Design System est fonctionnel (5 thèmes, composants de base).
> - Le layout principal est en place.
> - La CI passe.

## Objectifs du sprint

1. Créer les entités du noyau (Societe, Exercice, Compte, Journal, Ecriture, LigneEcriture).
2. Implémenter le moteur de comptabilité en partie double.
3. Mettre en place le bus d'événements typé.
4. Configurer SQLite + SQLCipher + migrations.
5. Exposer les premières commandes Tauri (CRUD sociétés, CRUD écritures).
6. Créer l'UI de gestion des sociétés.

## User Stories

### US-01-01 : Créer une société

**En tant qu'** utilisateur,
**je veux** créer une SCI et une SARL,
**afin de** gérer mes deux structures.

**Critères d'acceptation :**
- [ ] Formulaire de création avec : nom, type (SCI/SARL/...), régime fiscal (IR/IS), régime TVA, SIRET.
- [ ] Validation côté formulaire et validation Rust côté backend ; Zod reste une option et n’est pas ajouté dans cette alpha.
- [ ] La société est persistée dans la base locale SQLCipher.
- [ ] La liste des sociétés s'affiche.
- [ ] On peut basculer entre sociétés (sélecteur global dans le header).

**Tests :**
- [ ] Test unitaire : validation des champs.
- [ ] Test d'intégration : création en base.
- [ ] Test e2e : parcours complet de création.

**Rollback :**
- [ ] Supprimer la société (soft delete).

### US-01-02 : Créer une écriture comptable

**En tant qu'** utilisateur,
**je veux** saisir une écriture manuelle,
**afin de** gérer les opérations diverses.

**Critères d'acceptation :**
- [ ] Formulaire avec journal, date, libellé, lignes (compte, débit, crédit).
- [ ] Contrôle d'équilibre débit = crédit en temps réel.
- [ ] Impossible de valider une écriture déséquilibrée.
- [ ] L'écriture est persistée avec ses lignes.
- [ ] La liste des écritures s'affiche avec filtres (journal, date, compte).

**Tests :**
- [ ] Test unitaire : équilibre débit/crédit.
- [ ] Test d'intégration : persistance avec lignes.
- [ ] Test e2e : saisie et validation.

**Rollback :**
- [ ] Statut "brouillon" par défaut, suppression possible. *État réel : seul le statut `validated` existe (contrainte `CHECK` SQL) ; brouillons et contre-passation à concevoir ensemble.*

### US-01-03 : Plan Comptable Général

**En tant qu'** utilisateur,
**je veux** un plan comptable pré-rempli,
**afin de** ne pas avoir à le saisir.

**Critères d'acceptation :**
- [ ] Le PCG français est chargé au démarrage (seed).
- [ ] On peut ajouter/modifier des comptes.
- [ ] Les comptes sont filtrés par société.
- [ ] Recherche par numéro ou libellé.

**Tests :**
- [ ] Test d'intégration : seed du PCG.
- [ ] Test e2e : recherche d'un compte.

**Rollback :**
- [ ] Re-seed depuis un fichier.

## Tâches techniques

### T-01-01 : Entités du noyau

- [ ] `crates/core/src/entities/societe.rs` *(réel : `crates/core/src/society.rs`, `accounting.rs` regroupe compte + ligne + validation ; pas de dossier `entities/`)*
- [ ] `crates/core/src/entities/exercice.rs`
- [ ] `crates/core/src/entities/compte.rs`
- [ ] `crates/core/src/entities/journal.rs`
- [ ] `crates/core/src/entities/ecriture.rs`
- [ ] `crates/core/src/entities/ligne_ecriture.rs`
- [ ] Dériver `Serialize` / `Deserialize` dans le noyau ; isoler les conversions SQL dans l’adaptateur de persistance `rusqlite`.

### T-01-02 : Migrations SQLite

- [ ] Mettre en place des migrations versionnées compatibles avec `rusqlite` et le schéma SQLCipher (aucune dépendance SQLx à ajouter sans décision d’architecture).
- [ ] Migration `001_init.sql` : tables societes, exercices, comptes, journaux, ecritures, lignes_ecriture.
- [ ] Index sur `societe_id`, `date`, `compte_id`.
- [ ] Activer WAL mode.

### T-01-03 : Chiffrement SQLCipher

- [ ] Compiler `rusqlite` avec SQLCipher activé et refuser explicitement le mode SQLite non chiffré.
- [ ] Dériver la clé depuis une phrase de passe (Argon2id).
- [ ] Écran de saisie de la phrase de passe au démarrage.

### T-01-04 : Moteur comptable

- [ ] `crates/core/src/accounting/engine.rs`.
- [ ] Fonction `validate_ecriture(ecriture) -> Result<()>`.
- [ ] Fonction `compute_balance(societe_id, exercice_id) -> Balance`.
- [ ] Fonction `compute_grand_livre(societe_id, compte_id) -> Vec<Ligne>`.
- [ ] Fonction `export_fec(societe_id, exercice_id) -> String`.

### T-01-05 : Event bus

- [ ] `crates/core/src/events/bus.rs`.
- [ ] Bus d'événements typé. *État réel : `crates/core/src/events.rs`, `std::sync::mpsc`, en mémoire ; **non branché** sur les commandes Tauri (aucun `publish` appelé). `tokio::sync::broadcast` seulement si le besoin d'async est confirmé.*
- [ ] Trait `EventBus` + implémentation.
- [ ] Événements de base : `SocieteCreee`, `EcritureValidee`, etc.

### T-01-06 : Commandes Tauri

- [ ] `create_societe`, `list_societes`, `update_societe`, `delete_societe`.
- [ ] `create_ecriture`, `list_ecritures`, `get_ecriture`.
- [ ] `list_comptes`, `search_comptes`.
- [ ] `compute_balance`, `export_fec`.

### T-01-07 : UI Sociétés

- [ ] Page `/societes` avec liste et formulaire.
- [ ] Sélecteur global de société dans le header.
- [ ] Formulaire de création/édition.

### T-01-08 : UI Écritures

- [ ] Page `/ecritures` avec liste, filtres, formulaire.
- [ ] Page `/ecritures/:id` avec détail.
- [ ] Composant `EcritureForm` avec contrôle d'équilibre en temps réel.

## Ordre d'exécution
1. T-01-01 (entités)

2. T-01-02 (migrations)

3. T-01-03 (chiffrement)

4. T-01-04 (moteur)

5. T-01-05 (event bus)

6. T-01-06 (commandes Tauri)

7. T-01-07 (UI sociétés)

8. T-01-08 (UI écritures)

9. US-01-01 à US-01-03 (validation)


## Critères de sortie

- [ ] Création de sociétés fonctionnelle.
- [ ] Saisie d'écritures avec contrôle d'équilibre.
- [ ] PCG seedé.
- [ ] Export FEC fonctionnel.
- [ ] Tests passent, couverture ≥ 80%.
- [ ] `PROGRESS.md` mis à jour.
- [ ] Tag `v0.1.0`.

## État après sprint (handoff)

*Sprint non clos (IN_PROGRESS).* Livré et relu : sociétés (domaine + commandes + UI), validation d'écriture en partie double, 26 comptes de départ, écritures à deux lignes, audit des créations, chiffrement SQLCipher configuré. **Manque encore pour clore** (détails et ordre conseillé dans `../HANDOFF.md`) : exercices et journaux, PCG complet, écritures multi-lignes, immutabilité appliquée en base (triggers), audit de toutes les mutations, migrations versionnées, balance / grand livre / FEC, tests d'intégration SQLCipher, branchement du bus d'événements, `update/delete` société (soft delete).

## Notes et décisions

- [2026-10-12] ~~`Decimal` (rust_decimal)~~ — **remplacé** : tous les montants sont des centimes entiers `i64` (`debit_cents`, `credit_cents`, `total_cents`), conversion/parse côté frontend dans `lib/money.ts` sans flottant. Une éventuelle migration vers `Decimal` (ex. TVA au taux) serait une décision à consigner ici.
- [2026-10-12] Les écritures sont immuables une fois validées (on crée une contre-écriture pour annuler).