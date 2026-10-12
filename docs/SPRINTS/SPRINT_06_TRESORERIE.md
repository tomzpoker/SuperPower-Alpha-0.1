# Sprint 06 — Trésorerie, prévisions et réalisations guidées

## Métadonnées
| Champ | Valeur |
|---|---|
| Numéro | 06 |
| Branche du lot initial | \`feat/guided-cash-flows\` |
| Statut | IN_PROGRESS — CI et parcours natif à valider |
| Décision | [ADR-0001](../DECISIONS/ADR-0001-flux-previsionnels.md) |

## Problème utilisateur
Le gérant sait que « le loyer doit arriver », « l'assurance est payée » ou « le locataire n'a réglé qu'une partie ». Il ne doit pas connaître les comptes PCG ou le débit/crédit pour noter ces faits dans le prévisionnel.

## Objectif S06-A
Créer/modifier une prévision ponctuelle ; saisir plusieurs réalisations partielles ; calculer prévu/réalisé/écart/état ; annuler sans effacer l'historique ; migrer v1 vers v2 sans perte de données.

**Limite explicite :** ce lot suit le prévu/réalisé. Il ne crée pas d'écriture comptable et ne calcule pas la TVA. Une catégorie de pilotage ne suffit pas à déduire l'écriture exacte (principal/intérêts, tiers, dépôt, TVA, ventilation multi-lignes).

## User stories / critères d'acceptation
### US-06-A01 — Créer une prévision
- Société existante requise.
- Sens : encaissement attendu ou paiement attendu.
- Catégorie : loyer, charge courante, travaux, impôt/taxe, emprunt, apport d'associé, autre.
- Libellé 1–160 caractères ; date ISO ; montant positif en centimes.
- Tiers facultatif ≤160 caractères ; référence facultative ≤100.
- Validation Rust et audit dans la transaction.

### US-06-A02 — Enregistrer le réel
- Plusieurs réalisations par flux ; le prévu ne change pas.
- Date ISO, montant positif, référence ≤100, note ≤500.
- Total réalisé = somme des réalisations non annulées.
- Écart = prévu − réalisé ; il peut être négatif.
- Un dépassement est accepté mais signalé.
- Une prévision annulée refuse toute nouvelle réalisation.

### US-06-A03 — Corriger sans effacer
- Annulation avec motif obligatoire (1–300 caractères).
- La réalisation reste en base avec date et motif ; son montant est exclu du cumul.
- Annulation et audit transactionnels ; seconde annulation refusée.

### US-06-A04 — Modifier/annuler une prévision
- Une prévision active peut être modifiée ; ses réalisations sont conservées.
- La société ne peut pas changer après création.
- Une prévision annulée ne peut plus être modifiée/réalisée.
- La suppression d'une société portant un flux, même annulé, est refusée.

### US-06-A05 — État calculé
- \`a_prevoir\` : aucun réalisé, date aujourd'hui/future.
- \`en_retard\` : aucun réalisé, date passée.
- \`partiellement_realise\` : réalisé positif inférieur au prévu.
- \`realise\` : réalisé égal au prévu.
- \`depasse\` : réalisé supérieur au prévu.
- \`annule\` : prévision annulée, prioritaire.

## Modèle / migration
\`PRAGMA user_version\` passe de 1 à 2 en transaction à l'initialisation et au déverrouillage.
- \`flux_previsionnels\` : id, société, libellé, sens, catégorie, date, montant centimes, tiers, référence, statut, horodatages.
- \`flux_realisations\` : id, flux parent, date réelle, montant centimes, référence, note, date/motif d'annulation.
- FK \`ON DELETE RESTRICT\), index société/date et flux/date.
- Les tables v1 ne sont pas reconstruites ; une version future inconnue est refusée.

## Contrat IPC
| Commande | Arguments | Résultat |
|---|---|---|
| \`list_flux\` | \`societeId?\` | \`FluxView[]\`, limite 500 |
| \`create_flux\` | \`input: FluxInput\` | \`FluxView\` |
| \`update_flux\` | \`id\`, \`input: FluxInput\` | \`FluxView\`, société inchangée |
| \`cancel_flux\` | \`id\` | flux annulé |
| \`register_flux_realisation\` | \`fluxId\`, \`input: RealisationFluxInput\` | vue recalculée |
| \`cancel_flux_realisation\` | \`id\`, \`motifAnnulation\` | vue recalculée, motif conservé |

Propriétés IPC en \`camelCase\`, enums en \`snake_case\), erreurs françaises. Les commandes sont enregistrées dans le handler Tauri et le mode démo de \`api.ts\`.

## Interface
- Résumé prévu à encaisser, prévu à payer, total réalisé actif.
- Cartes : sens, libellé, échéance, catégorie, société, statut, prévu/réalisé/écart.
- Création/modification sans numéro de compte.
- Réalisations partielles datées, référence/note facultatives.
- Historique, annulation avec motif, filtre des prévisions annulées.
- Mode navigateur démonstratif, volatil/non chiffré ; avertissement visible sur l'absence de comptabilisation/TVA.

## Tests
- Domaine : états futur/retard/partiel/réalisé/dépassement/annulé ; montants invalides ; libellé vide ; dates invalides.
- Migration : v1→v2, tables présentes, idempotence, rejet version future.
- Native : société inconnue, flux annulé, réalisation invalide, audit transactionnel.
- Frontend : créer/modifier, réaliser partiellement puis compléter, annuler réalisation et vérifier cumul.
- Non-régression : sociétés et écritures v1 préservées.
- Contrôles : \`python scripts/check_docs.py\`, \`cargo fmt --all -- --check\`, \`cargo test -p superpower-core\`, \`cargo test --workspace\`, \`npm test\`, \`npm run typecheck\`, \`npm run build\), CI Windows.

## Hors périmètre
Flux récurrents, projection 12/24/36 mois, alertes, import/rapprochement bancaire, baux, comptabilisation PCG multi-lignes, lettrage, TVA, fiscalité, FEC, OCR et synchronisation mobile.

## État de livraison
IN_PROGRESS jusqu'à réussite de la CI de branche et du parcours natif.
