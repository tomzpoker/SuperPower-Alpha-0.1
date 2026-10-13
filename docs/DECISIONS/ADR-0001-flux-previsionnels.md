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

## Résultats des essais manuels — 2026-10-13

- Le test natif a été réalisé dans un coffre temporaire distinct. Selon le compte-rendu utilisateur : prévision de loyer modifiée à 1 400 € sans écraser les 400 € réalisés ; prévision d'assurance de 240 € ; réalisation de 300 € enregistrée puis annulée depuis Historique ; l'annulation conserve la ligne et son motif mais l'exclut du réalisé actif ; prévision assurance annulée et affichée via le filtre. L'écran final affiche 1 400 € prévus à encaisser, 0 € prévus à payer et 400 € réalisés actifs. La relance avec le même chemin conserve les données.
- Une copie du couple `superpower.db` / `superpower.salt` a été placée dans un dossier temporaire et ouverte avec la build de cette branche. L'ouverture, la réouverture du même dossier et la présence des données existantes ont été signalées comme réussies.
- Les essais sont restés isolés : l'original n'a pas servi de cible de migration et aucune fusion des données de test dans le coffre habituel n'a été faite.
- Réserve migration : la valeur initiale de `PRAGMA user_version` de la source n'a pas été relevée indépendamment ; l'essai ne prouve donc pas définitivement une migration v1 → v2. Le contrôle direct de `user_version = 2` et de la présence de `flux_previsionnels` / `flux_realisations` reste à faire.
- Résultat métier confirmé : l'utilisateur a vu l'état « Réalisé au-delà du prévu » lorsque 300 € étaient réalisés pour 240 € prévus, avant d'annuler cette réalisation depuis l'historique.
- Réserve métier restante : le refus côté commande métier d'une nouvelle réalisation sur une prévision déjà annulée n'a pas encore été testé séparément ; dans l'interface, les boutons d'action disparaissent après annulation.

## Définition de Done

- [x] Migration v1→v2 et idempotence couvertes par test automatisé sur le schéma v1 complet, avec vérification de conservation de données représentatives.
- [x] Domaine Rust et validations couverts par tests unitaires.
- [x] Commandes natives enregistrées et compilation Tauri Windows réussie sur la révision CI précédente.
- [x] UI Trésorerie construite avec tests frontend et typecheck réussis sur la révision CI précédente.
- [x] Mode démo navigateur cohérent et explicitement volatile.
- [x] Contrôles Rust, frontend, bundle de production, build Windows et Markdown passés sur la révision CI précédente.
- [x] CI verte sur la révision de code `b65085c` (run [38319595762](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38319595762)) ; la mise à jour documentaire doit déclencher un nouveau run.
- [ ] Confirmer une migration réelle v1 → v2 sur une copie dont le `PRAGMA user_version` source est explicitement vérifié, puis vérifier `user_version = 2` et les tables S06-A. Une copie locale a été ouverte puis rouverte, avec données visibles selon le retour utilisateur, mais sa version source n'a pas été relevée indépendamment.
- [x] Modification du loyer à 1 400 € avec conservation des 400 € réalisés ; enregistrement puis annulation depuis Historique de réalisations de 300 € sur les flux de test ; préservation des lignes/motifs et exclusion des montants annulés du cumul ; annulation de la prévision d'assurance et filtre des prévisions annulées — observés et confirmés par l'utilisateur.
- [x] Confirmer visuellement l'état « Réalisé au-delà du prévu » pour 300 € réalisés sur 240 € prévus ; confirmation utilisateur reçue.
- [ ] Confirmer que la commande métier refuse de créer une réalisation sur une prévision annulée (les boutons d'action ont disparu dans l'UI).

## S06-B — Règles de récurrence du prévisionnel

Décision d'extension : les flux peuvent être ponctuels (défaut), mensuels, trimestriels ou annuels. Les séries récurrentes génèrent des échéances indépendantes, sans écrire automatiquement en comptabilité et sans calculer la TVA.

- Les dates sont calculées depuis la date de départ d'origine pour éviter toute dérive de fin de mois ; les jours inexistants sont ramenés au dernier jour du mois cible. Les années bissextiles sont couvertes par tests.
- Sans date de fin, les occurrences sont matérialisées sur un horizon glissant de 36 mois. La matérialisation est idempotente et l'unicité (série, date) est protégée en base.
- La migration SQLCipher v2 → v3 ajoute `flux_recurrences` et un lien nullable depuis `flux_previsionnels`. Les flux existants restent ponctuels ; leurs réalisations et historiques ne sont ni supprimés ni remplacés.
- Une réalisation et une annulation concernent une occurrence seulement. Modifier une occurrence récurrente n'altère pas le modèle des prochaines échéances ; la modification globale d'une série est différée à un lot ultérieur.
- L'arrêt se fait sur une échéance générée et visible dans l'horizon de 36 mois. Les échéances postérieures non réalisées sont annulées mais conservées dans l'historique. L'arrêt est refusé s'il annulerait une échéance postérieure avec une réalisation active.
- La génération automatique des occurrences est auditée dans la transaction. Les règles métier sont validées dans Rust, pas uniquement dans l'interface.

Le suivi des flux demeure séparé des écritures comptables : aucune fréquence, catégorie ou description ne suffit à déduire une écriture PCG ni un traitement de TVA.

