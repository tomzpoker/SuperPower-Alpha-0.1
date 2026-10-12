# Références fiscales — points de vigilance

**Dernière vérification des liens : 12 octobre 2026.** Les pages administratives évoluent ; la date exacte des obligations doit être vérifiée pour l'exercice et la situation du contribuable. Cette page documente des principes généraux, pas un avis fiscal individualisé.

## 1. SCI à l'IR

- Une SCI relevant de l'IR dépose sa déclaration de résultats de société n° 2072 selon le régime qui lui est applicable ; la situation des associés et leur quote-part doivent être traitées séparément. Ne pas assimiler l'enregistrement de la société à la déclaration personnelle de chaque associé.
- La déclaration 2044 concerne les revenus fonciers des personnes qui doivent la remplir ; elle ne remplace pas la déclaration 2072 de la SCI.
- Une SCI peut exercer certaines activités ou options pouvant modifier le traitement TVA ou fiscal : ne pas enregistrer la mention « SCI IR » comme preuve d'exonération de TVA.

Source de départ : [impots.gouv.fr — Résultat imposable IR / IS](https://www.impots.gouv.fr/professionnel/resultat-imposable-limpot-sur-le-revenu-ir-ou-limpot-sur-les-societes).

## 2. Location de locaux professionnels et TVA

- Les locations de locaux nus sont en principe exonérées de TVA dans le cadre général décrit par le BOFiP ; la location nue de locaux à usage professionnel peut faire l'objet d'une option à la TVA dans les conditions applicables.
- L'application de l'option doit être vérifiée selon l'immeuble, le bail, l'usage et la situation du preneur. Les parkings, locaux équipés, meublés ou situations particulières peuvent relever d'analyses spécifiques : le logiciel ne doit pas extrapoler automatiquement une règle d'une catégorie à une autre.
- Pour les prestations de services, la TVA est en règle générale exigible à l'encaissement, sauf option valable pour les débits et règles particulières. Cette règle doit être paramétrée et sourcée, non déduite de la seule périodicité déclarative.

Sources officielles :

- [BOFiP — Locations immobilières soumises à TVA sur option](https://bofip.impots.gouv.fr/bofip/730-PGP.html/identifiant%3DBOI-TVA-CHAMP-50-10-20120912)
- [BOFiP — Exigibilité TVA des prestations de services](https://bofip.impots.gouv.fr/bofip/283-PGP.html/identifiant%3DBOI-TVA-BASE-20-20-20120912)
- [impots.gouv.fr — Obligation de créer un espace professionnel dans certains cas](https://www.impots.gouv.fr/professionnel/questions/je-suis-non-professionnel-suis-je-dans-lobligation-de-creer-un-espace)

## 3. SARL à l'IS

- Une société soumise à l'IS dépose normalement une déclaration annuelle de résultats n° 2065 et la liasse correspondant à son régime d'imposition.
- Les échéances dépendent notamment de la date de clôture, du régime et du calendrier fiscal en vigueur ; éviter de coder en dur une date annuelle sans règle versionnée et source officielle.

Sources :

- [impots.gouv.fr — Imposition des résultats](https://www.impots.gouv.fr/professionnel/imposition-des-resultats)
- [impots.gouv.fr — Résultat imposable IR ou IS](https://www.impots.gouv.fr/professionnel/resultat-imposable-limpot-sur-le-revenu-ir-ou-limpot-sur-les-societes)

## 4. Exigences de conception pour le moteur fiscal

Chaque calcul ou tâche fiscale devra enregistrer :

- la société, l'exercice, la période et le formulaire visés ;
- les transactions/documentations sources et leur traçabilité ;
- les hypothèses, options et paramètres pris en compte ;
- la source officielle et sa date de consultation ;
- la version de la règle de calcul ;
- les montants agrégés et leur rapprochement au grand livre ;
- un statut clair : estimation, préparation à contrôler, ou prêt pour révision.

Le moteur ne devra jamais télétransmettre une déclaration dans cette version sans parcours distinct, contrôles complets et autorisation explicite. La validation par un professionnel demeure recommandée, particulièrement pour l'option TVA immobilière, la qualification de certains loyers et les régimes dérogatoires.
