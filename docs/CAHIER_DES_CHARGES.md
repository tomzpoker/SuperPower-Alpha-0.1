# Cahier des charges — SuperPower Alpha

## 1. Contexte et objectif
Application locale, modulaire et sécurisée pour gérer plusieurs sociétés françaises (notamment SCI et SARL), anticiper la trésorerie, préparer la comptabilité et aider à la fiscalité sans dépendance cloud permanente. Les données ne quittent pas l'appareil sans action explicite.

## 2. Objectifs fonctionnels
| # | Objectif | Priorité |
|---|---|---|
| F1 | Multi-sociétés | Critique |
| F2 | Import CSV et catégorisation bancaire | Critique |
| F3 | Comptabilité partie double conforme au PCG | Critique |
| F4 | Classement documentaire explicable | Critique |
| F5 | Gestion locative SCI | Critique |
| F6 | Prévisionnel de trésorerie et rapprochement réel | Critique |
| F7 | Préparation TVA/2072/2044/2065 selon dossier | Élevée |
| F8 | Sync mobile par fichiers chiffrés | Élevée |
| F9 | Veille fiscale par prompt utilisateur | Moyenne |
| F10 | Stocks si besoin confirmé | Basse |
| F11 | Paramétrer séparément IR/IS, TVA, exigibilité et périodicité | Critique |
| F12 | Saisir prévisions/réalisations sans connaissance du plan comptable | Critique |

## 3. Périmètre V1
- Noyau multi-sociétés, comptabilité partie double.
- Tâches persistantes, import CSV, documents classés par règles.
- Biens, baux, loyers, échéances et TVA par opération.
- Trésorerie prévisionnelle et rapprochement.
- Préparation fiscale, sync mobile chiffrée, veille par prompt.
- UI desktop, cinq thèmes et parcours opérationnel simplifié.

Hors périmètre : plateforme agréée de facturation, synchronisation bancaire DSP2, paie complète, télétransmission fiscale, multi-utilisateur avancé et LLM local.

## 4. Contraintes techniques
Rust, Tauri 2, SQLite/SQLCipher, React 18, TypeScript, Vite. Pas de Docker/PostgreSQL en production. Les règles financières sont validées côté Rust. Le moteur de tâches sera choisi après prototype d'idempotence et de reprise.

## 5. UX
- Interface clean, simple, futuriste et pastel ; thèmes Aurora, Sakura, Mint, Sunset, Midnight.
- Actions courantes en deux clics maximum.
- Parcours opérationnel partant des faits métier : sens, catégorie, date, montant, sans choix obligatoire de comptes débit/crédit.
- Prévu, réalisé, écart et historique restent séparés.
- Annulation traçable ; un dépassement ne doit pas être masqué.
- Feedback immédiat, navigation clavier, contraste AA.

## 6. Personas
- Gérant autonome qui suit une SCI familiale et/ou une SARL.
- Collaborateur qui a besoin d'un parcours simplifié.
- Expert-comptable qui consulte les exports et écritures.

## 7. Cas d'usage
### UC-01 — Import mensuel
Importer le CSV bancaire, analyser les colonnes, proposer des catégories avec confiance, demander validation des éléments incertains, éviter les doublons.

### UC-02 — Facture fournisseur
Déposer une pièce, extraire les données, proposer un classement et une écriture ; l'utilisateur vérifie et confirme.

### UC-03 — Échéances locatives
Générer les échéances à partir du bail, suivre encaissements partiels, retards et justificatifs.

### UC-04 — Préparation fiscale
Agrégation annuelle traçable, préremplissage des cases applicables, vérification humaine et export du brouillon.

### UC-06 — Prévision et réalisation
1. Choisir la société et le sens (encaissement ou décaissement).
2. Choisir une catégorie de pilotage, décrire l'opération, saisir date/montant prévu.
3. Lors du mouvement réel, ajouter date et montant ; plusieurs réalisations partielles possibles.
4. Calculer total réalisé, écart et état.
5. Annuler une réalisation avec motif sans effacer la trace.

**Limite S06-A :** pas d'écriture PCG ou de TVA générée. Une catégorie de pilotage n'est pas une qualification comptable/fiscale. La comptabilisation automatique exige des règles validées par type d'opération.

## 8. Critères d'acceptation globaux
- Fonctions critiques testées ; aucune donnée envoyée sans action explicite.
- Données natives chiffrées ; contrôles Rust non contournables par l'UI.
- Documentation permettant une reprise par un développeur.
- Couverture cible >80 % sur le code métier critique, non déclarée atteinte.

## 9. Règle fiscale
Ne jamais inférer l'exonération TVA du seul statut SCI/IR ou l'exigibilité d'un libellé. Le traitement dépend du bien, de l'opération, du bail, du preneur, des options et du régime applicable. Les simulations doivent préciser hypothèses et sources officielles.

## 10. Glossaire
PCG : Plan comptable général ; FEC : Fichier des écritures comptables ; SCI : société civile immobilière ; IR : impôt sur le revenu ; IS : impôt sur les sociétés ; TVA : taxe sur la valeur ajoutée ; CA3 : déclaration TVA du régime réel normal ; 2044 : revenus fonciers ; 2072 : déclaration SCI à l'IR ; DSP2 : directive services de paiement ; PA : plateforme agréée.
