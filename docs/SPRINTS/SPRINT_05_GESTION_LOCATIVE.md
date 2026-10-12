# Sprint 05 — Gestion locative

## Métadonnées

| Champ | Valeur |
|---|---|
| **Numéro** | 05 |
| **Nom** | Gestion locative |
| **Statut** | TODO |
| **Dépendances** | Sprints 01, 02 et 03 |
| **Branche proposée** | `sprint/05-gestion-locative` |

## Objectifs

1. Gérer les biens, immeubles, lots, locataires et baux avec historique.
2. Générer les appels de loyers, suivre les règlements et détecter les impayés sans dupliquer les encaissements bancaires.
3. Produire quittances et reçus uniquement selon l'état réel du règlement.
4. Gérer l'indexation selon la clause du bail et l'indice applicable (ICC, ILC, ILAT ou autre selon contrat/activité), sans imposer un indice universel.
5. Permettre un traitement TVA explicite au niveau pertinent pour chaque local, bail et flux.

## Modèle indispensable

- **Bien / immeuble / lot** : adresse, destination, nature, lots, période de détention et pièces justificatives.
- **Locataire / preneur** : dénomination, identifiants, coordonnées et statut de TVA si renseigné.
- **Bail** : dates, destination, loyer HT/TTC selon traitement, dépôt de garantie, périodicité, répartition des charges, clauses d'indexation, échéances et pièces associées.
- **Traitement TVA par opération** : qualification de la location, exonération/assujettissement/option et justificatif, taux paramétré, exigibilité, dates d'encaissement, ventilation des règlements et état de contrôle.
- **Appel de loyer** et **règlement** restent des objets distincts. Un loyer appelé ne signifie pas qu'il est encaissé ; un encaissement de banque ne doit pas être enregistré une deuxième fois lors de son rapprochement.

## User stories

### US-05-01 — Référentiel locatif

En tant que gestionnaire, je veux relier les baux à un bien, un lot et un locataire afin de centraliser les obligations du contrat.

Critères : identité des entités contrôlée ; dates cohérentes ; historique conservé ; pièces rattachables ; tests d'invariants de dates.

### US-05-02 — Appels et encaissements

En tant que gestionnaire, je veux distinguer loyer attendu, règlement reçu et solde restant dû afin d'éviter les faux impayés et les doubles écritures.

Critères : échéancier traçable ; règlements partiels possibles ; impayé calculé par solde ; rapprochement bancaire idempotent.

### US-05-03 — TVA sur les flux locatifs

En tant que gestionnaire, je veux que le traitement TVA soit documenté pour chaque flux afin de préparer correctement les déclarations sans déduction générale au niveau de la SCI.

Critères : option/exonération justifiée ; mode d'exigibilité et périodicité distincts ; TVA exigible calculée à partir des événements pertinents ; règles sourcées et versionnées ; validation manuelle de tout cas ambigu.

### US-05-04 — Révision contractuelle

En tant que gestionnaire, je veux recalculer un loyer selon la clause et l'indice explicitement enregistrés afin d'éviter d'utiliser le mauvais indice.

Critères : source de l'indice ; trimestre/base ; formule et plafond contractualisés ; valeur précédente et nouvelle conservées ; aperçu avant application.

## Tests à prévoir

- Bail actif, futur, terminé et données de dates invalides.
- Règlement complet, partiel, excédentaire et annulé.
- Absence de double encaissement lors de l'import puis du rapprochement.
- TVA : location exonérée, option valide, encaissement partiel et TVA exigible ; option absente ou incertaine signalée sans calcul silencieux.
- Révision avec indice valide, mauvais indice et clause sans indexation.

## Garde-fous

Les règles de TVA immobilière ne seront pas codées depuis le seul statut « SCI à l'IR ». Référence initiale : `../REFERENCES_FISCALES.md`. Les cas spéciaux doivent être revus avec les sources officielles et un professionnel compétent.
