# ADR-0001 — Séparer flux métier, réalisation et écriture comptable

- **Statut :** ACCEPTÉ pour S06-A ; règles comptables/fiscales automatiques différées.
- **Date :** 2026-10-13.
- **Portée :** UI, domaine Rust, SQLCipher, IPC, trésorerie et comptabilité.

## Contexte
Le parcours précédent demandait de choisir un compte au débit, un compte au crédit et un montant. Le gérant sait décrire « loyer prévu », « assurance payée » ou « paiement partiel », mais ne doit pas connaître le plan comptable. Prévision, paiement réel et écriture comptable sont trois objets différents.

## Décision
1. Ajouter \`superpower_core::flows\` pour sens/catégories, DTO, validation et états.
2. Persister les prévisions dans \`flux_previsionnels\`, chaque réalisation dans \`flux_realisations\`.
3. Autoriser plusieurs réalisations partielles ; ne pas écraser le prévu.
4. Annuler une réalisation en ajoutant date/motif, sans suppression physique.
5. Conserver les prévisions annulées et bloquer la suppression d'une société ayant un historique de flux.
6. Migrer \`user_version 1 → 2\` en transaction à l'initialisation/déverrouillage ; refuser une version future inconnue.
7. Ne pas produire d'écriture PCG ou de TVA depuis le seul libellé/catégorie/sens. Une couche future justifiera comptes, nature de l'opération, paramètres TVA et provenance.
8. Aucun service cloud ni dépendance externe ajoutés pour ce parcours.

## Contrat métier
### Prévision
- Société existante requise ; libellé 1–160 caractères ; date \`AAAA-MM-JJ\` ; montant positif en centimes.
- Sens \`encaissement\` ou \`decaissement\`.
- Catégories de pilotage : \`loyer\`, \`charge_courante\`, \`travaux\`, \`impot_taxe\`, \`emprunt\`, \`apport_associe\`, \`autre\`.
- Tiers ≤160, référence ≤100 ; facultatifs.
- Prévision active modifiable ; société stable ; prévision annulée non modifiable/non réalisable.

### Réalisation
- Date ISO, montant positif, référence ≤100, note ≤500.
- Plusieurs réalisations autorisées.
- Total réalisé = somme des lignes non annulées ; dépassement visible.
- Motif d'annulation obligatoire (1–300), date/motif conservés et action auditée.

### États
- \`a_prevoir\) : zéro réalisé, date aujourd'hui/future.
- \`en_retard\) : zéro réalisé, date passée.
- \`partiellement_realise\) : réalisé >0 et inférieur au prévu.
- \`realise\) : égalité prévu/réalisé.
- \`depasse\) : réalisé supérieur au prévu.
- \`annule\) : prévision annulée.
- Écart = prévu − réalisé.

## Schéma v2
\`flux_previsionnels\` stocke le fait planifié ; \`flux_realisations\` les événements réels. Montants entiers avec contraintes SQL, dates ISO, FK \`ON DELETE RESTRICT\`. L'audit de chaque mutation partage la transaction. Les tables v1 ne sont pas détruites.

## Sécurité / fiscalité
Les commandes revalident les données côté Rust. Le mode navigateur est volatil/non chiffré. Le journal global n'a pas encore de triggers append-only. Une réalisation ne détermine pas seule si l'opération est revenu/charge, quelle TVA s'applique ou quand elle est exigible.

Le moteur comptable futur devra traiter multi-lignes, tiers, principal/intérêts d'emprunt, apports/CCA, dépôts/garanties, avoirs/annulations, paiements partiels, ventilation TVA, exercices/journaux et contre-passation.

## Alternatives écartées
- Choisir débit/crédit dès le premier geste : trop technique.
- Créer automatiquement « banque contre produit/charge » : faux/incomplet pour emprunts, apports, dépôts, TVA et opérations mixtes.
- Écraser le prévu par le réel : perd la comparaison.
- Supprimer une réalisation erronée : détruit l'historique.
- Construire récurrences/baux dès le premier lot : retarde la validation du parcours simple.

## Définition de Done

- [x] Migration v1→v2 et idempotence couvertes par test automatisé sur le schéma v1 complet, avec vérification de conservation de données représentatives.
- [x] Domaine Rust et validations couverts par tests unitaires.
- [x] Commandes natives enregistrées et compilation Tauri Windows réussie sur la révision CI précédente.
- [x] UI Trésorerie construite avec tests frontend et typecheck réussis sur la révision CI précédente.
- [x] Mode démo navigateur cohérent et explicitement volatile.
- [x] Contrôles Rust, frontend, bundle de production, build Windows et Markdown passés sur la révision CI précédente.
- [ ] Confirmer que la CI est verte sur la dernière révision après renforcement des tests et mise à jour documentaire.
- [ ] Effectuer le parcours natif sur une copie de base SQLCipher v1 existante et vérifier la préservation réelle des données.
- [ ] Parcourir manuellement le scénario métier prévu/réalisé dans l'application native et consigner les résultats.
