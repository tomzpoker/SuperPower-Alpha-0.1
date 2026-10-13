# Sprint 06 — Trésorerie, prévisions et réalisations guidées

## Métadonnées
| Champ | Valeur |
|---|---|
| Numéro | 06 |
| Branche du lot initial | \`feat/guided-cash-flows\` |
| Statut | S06-A et S06-B intégrés dans `main` ; CI S06-B verte sur le run #96 ; validation native migration v2→v3 restante |
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
- Migration : v1→v2 depuis le schéma v1 complet, préservation vérifiée d'une société, d'une écriture, de ses lignes et d'une entrée d'audit, tables présentes, idempotence, rejet de version future. Ce test reste en mémoire et ne remplace pas un essai sur copie SQLCipher réelle.
- Native : société inconnue, flux annulé, réalisation invalide, audit transactionnel.
- Frontend : créer/modifier, réaliser partiellement puis compléter, annuler réalisation et vérifier cumul.
- Non-régression : sociétés et écritures v1 préservées.
- Contrôles : \`python scripts/check_docs.py\`, \`cargo fmt --all -- --check\`, \`cargo test -p superpower-core\`, \`cargo test --workspace\`, \`npm test\`, \`npm run typecheck\`, \`npm run build\), CI Windows.

## Hors périmètre
Flux récurrents, projection 12/24/36 mois, alertes, import/rapprochement bancaire, baux, comptabilisation PCG multi-lignes, lettrage, TVA, fiscalité, FEC, OCR et synchronisation mobile.

La récurrence est planifiée dans le lot suivant : [Sprint 06-B — Flux prévisionnels récurrents](SPRINT_06B_RECURRENCE.md), suivi dans [l'issue GitHub #2](https://github.com/tomzpoker/SuperPower-Alpha-0.1/issues/2). Elle ne fait pas partie de S06-A et n'est pas encore implémentée.

## Résultats du parcours natif (2026-10-13)
Selon le test manuel utilisateur, dans un coffre temporaire distinct :
- Modification du loyer test à 1 400 € avec les 400 € réalisés conservés ; écart affiché 1 000 €.
- Deux réalisations de 300 € saisies sur des flux de test puis annulées depuis Historique ; les lignes annulées et leur motif restent visibles et ne comptent plus dans le cumul.
- Prévision assurance à 240 € annulée ; affichée avec le filtre des prévisions annulées et les boutons de modification/réalisation ne sont plus proposés.
- Totaux finaux observés : 1 400 € prévus à encaisser, 0 € prévus à payer, 400 € réalisés actifs.
- L'utilisateur confirme avoir vu l'état « Réalisé au-delà du prévu » pour 300 € réalisés sur 240 € prévus, avant d'annuler la réalisation depuis l'historique.
- Le refus métier de réaliser sur une prévision annulée doit encore être confirmé séparément ; les boutons disparaissaient déjà dans l'interface après annulation.

## État de livraison
S06-A et S06-B sont intégrés dans `main`. La CI complète du code S06-B est verte sur le run [#96](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38332638917) ; le run post-fusion #97 reste à confirmer. La copie native précédemment ouverte et rouverte n'avait pas de version source `PRAGMA user_version` relevée indépendamment : ne pas considérer cet ancien essai comme une preuve formelle de migration v1→v2. Pour S06-B, valider v2→v3 sur une copie dont la version source est connue, sans toucher au coffre habituel.
