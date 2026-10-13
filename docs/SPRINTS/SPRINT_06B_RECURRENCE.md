# Sprint 06-B — Flux prévisionnels récurrents

## Métadonnées

| Champ | Valeur |
|---|---|
| Lot parent | Sprint 06 — Trésorerie |
| Prérequis | S06-A fusionné dans main |
| Statut | SPÉCIFIÉ — à développer ; aucune récurrence n'est encore implémentée |
| Issue | [#2 — Configurer les flux récurrents](https://github.com/tomzpoker/SuperPower-Alpha-0.1/issues/2) |

## Objectif

À la création ou à la modification d'une prévision, l'utilisateur choisit **Ponctuel**, **Mensuel**, **Trimestriel** ou **Annuel**. Il ne doit pas ressaisir manuellement les loyers, assurances et autres opérations qui reviennent à date fixe.

Ce lot étend le suivi prévu/réalisé de S06-A. Il ne produit toujours aucune écriture comptable et ne calcule pas automatiquement la TVA.

## User stories et critères d'acceptation

### US-06-B01 — Choisir le type de flux

- Le formulaire propose les quatre choix : ponctuel, mensuel, trimestriel et annuel.
- « Ponctuel » est le choix par défaut pour préserver le comportement des flux actuels.
- Une récurrence peut avoir une date de fin facultative ; la date de fin est incluse.
- Le formulaire explique clairement la fréquence choisie et la prochaine échéance.

### US-06-B02 — Générer les échéances récurrentes

- Une récurrence génère des échéances visibles dans le prévisionnel et dans les totaux.
- Chaque occurrence a sa propre date prévue, son montant prévu, son état, ses réalisations et son historique.
- La génération est idempotente : recharger, relancer l'application ou ouvrir plusieurs fois l'écran ne crée aucun doublon.
- Une récurrence sans date de fin est matérialisée sur une fenêtre glissante de 36 mois ; les échéances suivantes sont ajoutées à mesure que la fenêtre avance. Une date de fin borne cette génération.
- Les flux ponctuels préexistants après migration ne génèrent aucune occurrence supplémentaire.

### US-06-B03 — Calendrier fiable

- Le calcul reste ancré sur la date de départ et ne dérive pas d'un mois au suivant.
- Si le jour n'existe pas dans un mois cible, l'échéance tombe le dernier jour de ce mois.
- Exemple mensuel à partir du 31 janvier : 28/29 février puis 31 mars, et non 28/29 mars.
- Les échéances trimestrielles et annuelles suivent la même règle ; les années bissextiles sont testées.

### US-06-B04 — Gérer une occurrence sans corrompre la série

- Enregistrer ou annuler une réalisation affecte uniquement l'occurrence choisie.
- Annuler une occurrence n'arrête pas la série et ne modifie pas ses autres échéances.
- Arrêter une récurrence empêche la création de nouvelles échéances au-delà de la date de fin choisie ; les échéances et réalisations historiques restent conservées.
- La modification d'une échéance ne doit jamais changer silencieusement les montants, dates ou réalisations des autres occurrences. Le comportement de modification « cette échéance / cette échéance et les suivantes » doit être explicitement traité dans l'interface et les tests avant livraison.

### US-06-B05 — Cohérence des données

- Toute série est rattachée à une société existante ; les occurrences héritent de cette société.
- Les montants demeurent des entiers en centimes.
- L'audit enregistre la création, la modification et l'arrêt d'une série.
- Les commandes Rust revalident les règles ; la validation frontend seule ne suffit pas.
- Une nouvelle migration versionnée est transactionnelle et conserve le schéma v2, ses sociétés, écritures, journaux d'audit, prévisions et réalisations.
- Aucun traitement PCG ou TVA n'est inféré depuis la fréquence ou le libellé.

## Modèle de données proposé

Ajouter une table de séries de récurrence en migration v2 → v3. Elle stocke la fréquence, la date de début, la date de fin facultative, l'état actif/arrêté et les paramètres du modèle : société, libellé, sens, catégorie, montant, tiers et référence.

Chaque échéance reste une ligne indépendante dans flux_previsionnels et garde son propre historique dans flux_realisations. Ajouter un lien nullable entre une échéance et sa série ; NULL désigne une prévision ponctuelle existante ou nouvelle. Garantir l'unicité d'une occurrence par série et date. Les noms définitifs des colonnes seront validés lors de l'implémentation.

La génération doit être déterministe, bornée à 36 mois glissants au-delà de la date courante pour les séries sans fin, et sûre à répéter au démarrage ou à la consultation. Une interruption en cours de génération ne doit laisser ni doublon ni série partiellement corrompue.

## Couverture de tests requise

- Domaine : fréquence ponctuelle et récurrente, dates de fin, génération mensuelle/trimestrielle/annuelle, mois de 28/29/30/31 jours, années bissextiles.
- Persistance : v2 → v3 sans perte, contrainte d'unicité des occurrences, génération idempotente et rollback en cas d'échec.
- Commandes : création/modification/arrêt d'une série, société incorrecte refusée, occurrence annulée sans effet sur les autres.
- Frontend : les quatre choix, date de fin, fréquence visible, totaux par occurrences, arrêt de récurrence et données conservées après redémarrage.
- CI complète : Rust, TypeScript, tests frontend, vérification documentaire et build Tauri Windows.
- Essai natif dans un coffre temporaire ; ne jamais utiliser le coffre habituel comme cible de migration de test.

## Hors périmètre

Projection financière avancée au-delà des occurrences générées, rapprochement bancaire, import bancaire, alertes avancées, création d'écritures PCG, calcul de TVA, baux et automatisation fiscale.
