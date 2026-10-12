# Cahier des Charges — SuperPower Alpha

## 1. Contexte

L'utilisateur gère deux structures en France :
- **Une SCI** (société civile immobilière), généralement imposée à l'IR dans le scénario visé ; le traitement TVA des locaux professionnels, parkings et baux doit être configuré selon la situation réelle, les options et la nature de chaque opération.
- **Une SARL** soumise à l'IS dans le scénario visé ; son régime et sa périodicité de TVA sont des paramètres distincts à confirmer par dossier.

Les outils existants (Indy, Pennylane, Tiime...) sont des SaaS :
- Coûteux à l'usage.
- Dépendants du cloud (souveraineté des données).
- Peu modulables (on ne peut pas ajouter ses propres règles).
- Lourds (nécessitent une connexion permanente).

**Objectif** : créer un outil local, modulaire, léger et sécurisé qui automatise au maximum la gestion des deux structures, avec une expérience utilisateur digne des meilleurs SaaS modernes.

## 2. Objectifs

### 2.1 Objectifs fonctionnels

| # | Objectif | Priorité |
|---|---|---|
| F1 | Gérer plusieurs sociétés (SCI + SARL) dans la même app | Critique |
| F2 | Importer des relevés bancaires CSV et catégoriser automatiquement | Critique |
| F3 | Tenir une comptabilité en partie double conforme au PCG | Critique |
| F4 | Analyser et classer automatiquement les documents (sans LLM local) | Critique |
| F5 | Gérer les baux, loyers, quittances, impayés, révisions ICC (SCI) | Critique |
| F6 | Projeter la trésorerie prévisionnelle avec rapprochement réel | Critique |
| F7 | Préparer les déclarations fiscales (TVA CA3, 2044/2072) | Élevée |
| F8 | Synchroniser un mobile via fichiers chiffrés (mail/Drive) | Élevée |
| F9 | Générer un prompt de veille fiscale à coller dans un LLM web | Moyenne |
| F10 | Gérer les stocks (SARL à l'IS) | Basse (V2) |
| F11 | Paramétrer séparément régime fiscal, régime TVA, exigibilité et périodicité | Critique |

### 2.2 Objectifs non fonctionnels

| # | Objectif | Cible |
|---|---|---|
| NF1 | Légèreté | Binaire < 30 Mo, RAM au repos < 80 Mo |
| NF2 | Fluidité | Navigation < 100 ms, actions < 300 ms |
| NF3 | Sécurité | Chiffrement au repos, zéro port exposé |
| NF4 | Modularité | Ajout d'un module sans toucher au noyau |
| NF5 | Testabilité | Couverture > 80% sur le noyau et les modules critiques |
| NF6 | Portabilité | Windows, macOS, Linux (desktop) + iOS/Android (mobile) |
| NF7 | Accessibilité | Contraste AA, navigation clavier, lecteurs d'écran |

## 3. Périmètre

### 3.1 Dans le périmètre V1

- Noyau multi-sociétés + comptabilité partie double.
- Moteur de tâches persistant (solution à choisir après prototype, voir §4.1 ; `flexiq-core` est une piste historique, absente du dépôt, pas une décision).
- Import bancaire CSV + catégorisation.
- Gestion documentaire intelligente (règles + regex + score).
- Gestion locative SCI (baux, loyers, quittances, impayés, ICC, TF).
- Trésorerie prévisionnelle.
- Fiscalité préparatoire : TVA selon le régime applicable, préparation des formulaires 2072/2044/2065 selon chaque situation et règles de l'exercice.
- Sync mobile par fichiers chiffrés.
- Veille fiscale par prompt + presse-papier.
- UI desktop + mobile avec 5 thèmes.

### 3.2 Hors périmètre V1

- Facturation électronique certifiée PA (obligation légale, nécessite un partenaire).
- Synchronisation bancaire automatique (DSP2, agrégateurs).
- Gestion de paie complète.
- Déclarations fiscales télétransmises automatiquement.
- Multi-utilisateurs avancé (rôles, permissions fines).
- LLM local (explicitement exclu pour raisons de légèreté).

## 4. Contraintes

### 4.1 Techniques

- **Langage backend** : Rust (édition 2021+).
- **Framework desktop** : Tauri 2.
- **Base de données** : SQLite (avec SQLCipher pour le chiffrement).
- **Frontend** : React 18 + TypeScript + Vite et CSS à variables pour les cinq thèmes.
- **Moteur de tâches** : module Rust persistant à sélectionner ou concevoir après validation d'un prototype et de ses garanties d'idempotence.
- **Pas de LLM local**. Pas de Docker en production. Pas de PostgreSQL.

### 4.2 Légales

- Conformité PCG (Plan Comptable Général) français.
- Conformité RGPD (données locales, pas de transfert non consenti).
- Export FEC (Fichier des Écritures Comptables) obligatoire pour les contrôles fiscaux.
- Conservation des pièces justificatives 10 ans.

### 4.3 UX

- Interface **clean, simple, futuriste, pastel**.
- 5 thèmes au choix (Aurora, Sakura, Mint, Sunset, Midnight).
- Navigation en 2 clics maximum pour toute action courante.
- Feedback visuel immédiat (animations subtiles, micro-interactions).
- Mode simplifié pour les tâches opérationnelles.

## 5. Personas

### 5.1 Gérant autonome (persona principal)

- 35–55 ans, gère une SCI familiale et une SARL de services.
- À l'aise avec l'informatique sans être développeur.
- Veut gagner du temps, éviter les erreurs, ne pas dépendre d'un cabinet.
- Utilise un desktop (Windows/macOS) au bureau et un mobile en déplacement.

### 5.2 Conjoint collaborateur (persona secondaire)

- Accès en lecture + validation de certaines tâches.
- Utilise principalement le mobile.
- A besoin d'une interface ultra-simplifiée.

### 5.3 Expert-comptable (persona tertiaire)

- Accès ponctuel en lecture pour vérifier les écritures.
- A besoin d'exports propres (FEC, balance, grand livre).

## 6. Cas d'usage principaux

### UC-01 : Import mensuel du relevé bancaire

1. L'utilisateur télécharge le CSV depuis sa banque.
2. Il le dépose dans le dossier surveillé (ou via l'UI).
3. Le système parse, catégorise automatiquement 90% des transactions.
4. L'utilisateur valide les 10% restants en 2 clics.

### UC-02 : Réception d'une facture fournisseur

1. L'utilisateur photographie la facture ou la dépose.
2. Le système détecte le type, extrait les données, la classe.
3. Notification : "Facture EDF — 45,20 € — Compte 6061 proposé. Valider ?"
4. L'utilisateur clique "Oui". Écriture créée.

### UC-03 : Génération des quittances mensuelles

1. Le 1er du mois, le moteur de tâches déclenche la génération.
2. Les quittances sont créées pour tous les locataires à jour.
3. Les impayés déclenchent automatiquement une relance.
4. L'utilisateur reçoit un récapitulatif.

### UC-04 : Préparation de la déclaration 2044

1. En avril, le système agrège revenus et charges de l'année.
2. Il pré-remplit les cases du formulaire 2044.
3. L'utilisateur vérifie et exporte le PDF.

### UC-05 : Veille fiscale trimestrielle

1. L'utilisateur clique sur "Veille fiscale".
2. Le prompt contextuel est copié dans le presse-papier.
3. Le LLM web s'ouvre. L'utilisateur colle (Ctrl+V) et valide.
4. Il reçoit une analyse personnalisée.

## 7. Critères d'acceptation globaux

- Toutes les fonctionnalités critiques (F1–F6) sont opérationnelles et testées.
- L'application démarre en moins de 2 secondes sur un SSD.
- La RAM au repos ne dépasse pas 80 Mo.
- Aucune donnée ne quitte la machine sans action explicite de l'utilisateur.
- L'UI est jugée "sexy et apaisante" par au moins 3 testeurs.
- Le code est documenté et testé (couverture > 80%).
- Un nouveau développeur peut comprendre l'architecture en 1 heure.

## 8. Risques et mitigations

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Complexité de la compta partie double | Moyenne | Élevé | S'inspirer d'OpenConcerto, tests exhaustifs |
| OCR peu fiable sur scans | Élevée | Moyen | Score de confiance + validation manuelle |
| Sync mobile complexe | Moyenne | Élevé | Commencer par un seul sens (mobile → desktop) |
| Dérive du périmètre | Élevée | Élevé | Sprints courts, MVP strict, PROGRESS.md |
| Performance SQLite sur gros volumes | Faible | Moyen | Indexes, pagination, tests de charge |
| Courbe d'apprentissage Rust | Élevée | Moyen | Commencer par le noyau simple, itérer |

## 9. Règle de conception fiscale

Le logiciel ne doit jamais inférer l'exonération de TVA à partir du seul statut SCI/IR, ni l'exigibilité sur encaissements à partir d'un libellé de bail. Les paramètres généraux de société doivent être complétés, lorsque nécessaire, par le traitement du bien, du lot, du bail, du locataire et de chaque flux. Toute simulation doit préciser ses hypothèses et renvoyer aux sources officielles. Voir `REFERENCES_FISCALES.md`.

## 10. Glossaire

- **PCG** : Plan Comptable Général (norme française).
- **FEC** : Fichier des Écritures Comptables (export obligatoire).
- **SCI** : Société Civile Immobilière.
- **SARL** : Société à Responsabilité Limitée.
- **IR** : Impôt sur le Revenu.
- **IS** : Impôt sur les Sociétés.
- **TVA** : Taxe sur la Valeur Ajoutée.
- **CA3** : déclaration de TVA du régime réel normal ; périodicité et échéance à vérifier dans le dossier fiscal.
- **2044** : Formulaire de déclaration des revenus fonciers.
- **2072** : Formulaire de déclaration pour les SCI à l'IR.
- **ICC** : Indice du Coût de la Construction (révision des loyers).
- **IRL** : Indice de Référence des Loyers (alternative à l'ICC).
- **DSP2** : Directive européenne sur les services de paiement.
- **PA** : Plateforme Agréée (pour la facturation électronique).