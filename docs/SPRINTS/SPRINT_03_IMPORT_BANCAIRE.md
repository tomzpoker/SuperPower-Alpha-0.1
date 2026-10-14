# Sprint 03 — Import bancaire

## Métadonnées
| Champ | Valeur |
|---|---|
| **Numéro** | 03 |
| **Nom** | Import bancaire |
| **Durée estimée** | 4–5 jours |
| **Dépendances** | Sprints 01 et 02 |
| **Branche Git** | `sprint/03-import-bancaire` (base `main`) |
| **Issue** | [#9 — Import bancaire CSV avec aperçu et détection des doublons](https://github.com/tomzpoker/SuperPower-Alpha-0.1/issues/9) |
| **Statut** | IN_PROGRESS — parseur de domaine dans la PR #10 ; CI #211 verte pour la prise en charge des débits CA déjà négatifs. Persistance SQLCipher et interface à venir |

## Objectif
Importer des relevés bancaires CSV avec un aperçu avant confirmation. Normaliser les dates et montants sans flottants, conserver la traçabilité des lignes sources et empêcher qu'un import répété duplique les transactions.

## Invariants non négociables
- L'aperçu est sans effet de bord : aucune écriture en base avant la confirmation explicite.
- Un import ne génère jamais automatiquement d'écriture PCG, de rapprochement validé ou de calcul de TVA.
- Le mapping des colonnes est explicite ; les indices sont basés sur zéro et ne sont pas devinés depuis le contenu.
- Les montants sont stockés en centimes entiers. Une date ou un montant ambigu/invalide est signalé, pas corrigé en silence.
- Les doublons sont identifiés de façon déterministe et traçable ; aucune ligne différente ne doit être supprimée automatiquement.
- Les données CSV sont non fiables ; les erreurs de format doivent devenir des messages métier et non des paniques.

## User Stories
- US-03-01 : Configurer un mapping de colonnes (date, libellé, montant ou débit/crédit, référence facultative).
- US-03-02 : Prévisualiser et valider les lignes normalisées avant leur persistance.
- US-03-03 : Identifier les lignes invalides avec le numéro d'enregistrement et le motif.
- US-03-04 : Réimporter un même relevé sans créer de doublons.
- US-03-05 : Consulter les transactions importées et leur source ; annuler un aperçu sans persister de données.

## Tranches techniques
- [x] S03-A : Parser CSV indépendant de la base et de l'interface, avec délimiteurs virgule/point-virgule/tabulation, guillemets doubles, champs multilignes, mapping explicite et aperçu.
- [x] S03-A : reconnaissance des en-têtes régionaux Crédit Agricole, encodage Windows-1252/UTF-8, colonnes date opération/date valeur, montant ou débit/crédit, récapitulatifs carte signalés sans suppression automatique.
- [x] S03-A : profil LCL sans en-tête (date, montant signé, type, chèque éventuel, libellé), détection des lignes de métadonnées de compte/solde et avertissement de validation humaine.
- [x] S03-A : Normaliser les dates ISO/françaises, convertir en centimes entiers, accepter les colonnes Débit déjà négatives et collecter les erreurs par ligne.
- [x] S03-B : Contrôles frontend, formatage et tests de domaine validés sur le correctif CA. Le build complet Windows/Tauri est opt-in pour éviter de le relancer sur chaque modification ; le déclencher manuellement ou avec le label PR `run-windows-ci` avant validation native.
- [ ] S03-C : Définir la persistance SQLCipher : compte/source, empreinte stable de transaction, provenance du fichier et migration additive transactionnelle.
- [ ] S03-D : Ajouter les commandes Tauri de prévisualisation/confirmation, avec protection contre les doubles imports.
- [ ] S03-E : Ajouter l'écran d'import (sélection du fichier, mapping explicite, aperçu, erreurs, confirmation/annulation).
- [ ] S03-F : Couvrir l'idempotence, la conservation des données et l'absence de génération implicite d'écritures PCG/TVA.

## Tranche S03-A — parseur orienté Crédit Agricole et LCL
Le module `superpower-core::bank_import` n'accède ni au système de fichiers ni à la base. La nouvelle entrée `parse_bank_csv_bytes` prend les octets originaux du fichier, reconnaît UTF-8 (avec ou sans BOM), Windows-1252/Latin-1 et UTF-16, détecte le séparateur, normalise les noms de colonnes sans tenir compte des accents/majuscules, puis produit un aperçu et les erreurs par ligne.

**Crédit Agricole :** les exports varient selon la Caisse régionale. Le parseur reconnait les en-têtes comportant Date/Date opération, Libellé/Libellé opération, Montant signé ou Débit + Crédit, ainsi que les colonnes optionnelles Date valeur et Référence. Date opération est prioritaire sur Date valeur. Les champs multilignes et les montants français sont conservés. Si une ligne ressemble à un récapitulatif « Dépenses carte / Prélèvement carte », un avertissement demande de vérifier le cumul avec le détail des achats ; aucune opération n'est supprimée automatiquement.

**LCL :** en l'absence d'en-tête, le parseur sait reconnaître le motif `date ; montant signé ; type ; numéro de chèque éventuel ; libellé`. Les lignes de métadonnées en début/fin de fichier (date, solde, numéro de compte), généralement plus courtes que les lignes d'opération, sont ignorées. Un avertissement signale que les colonnes ont été inférées et doivent être revues dans l'aperçu.

Montants pris en charge : entiers, décimales à un ou deux chiffres, virgule française, point décimal, groupements de milliers par espaces ou séparateurs cohérents, symbole euro et préfixe/suffixe EUR. Pour les exports à deux colonnes, un débit positif est normalisé en négatif et un débit déjà négatif reste négatif ; les crédits négatifs sont signalés comme invalides. L'en-tête UTF-8 avec BOM est retiré avant parsing (y compris si le premier champ est entre guillemets) ; les fichiers de plus de 10 Mio sont rejetés avant parsing. Si le format est ambigu, la ligne est rejetée plutôt que devinée.

Les tests unitaires du parseur couvrent les formats français/internationaux, les en-têtes BOM UTF-8, les exports Crédit Agricole dont la colonne Débit contient déjà un nombre négatif, les colonnes débit/crédit manquantes, les dates impossibles, les montants ambigus, les guillemets échappés et les champs multilignes. La CI #211 sur le correctif du signe est verte. Le changement de clé de cache a provoqué un premier cache miss ; le run a rempli un cache d’environ 809 Mo. Le prochain run doit vérifier que ce cache réduit bien la durée des builds sans changement de dépendances.

## Échantillons et limitation connue
Un premier extrait Crédit Agricole a été fourni au format « Date opération;Libellé;Débit euros;Crédit euros » et a mis en évidence que certains exports portent déjà un montant négatif dans la colonne Débit. Ce cas est maintenant couvert par un test de régression utilisant des données fictives. Les exports Crédit Agricole peuvent varier d'une Caisse régionale à l'autre ; LCL peut modifier son interface ou ses en-têtes. La validation finale doit encore inclure un relevé complet expurgé par banque, notamment un export LCL réel, avant d'ajouter la persistance.

## Critères d'acceptation globaux
- Un CSV valide produit un aperçu avant toute écriture en base.
- Les dates impossibles, montants mal formés, lignes incomplètes et mappings contradictoires sont signalés.
- Les guillemets CSV échappés et les champs contenant des retours à la ligne sont interprétés sans découper la ligne par erreur.
- L'import de la même transaction est idempotent, tout en conservant les lignes sources et les cas de collisions pour inspection.
- Les migrations SQLCipher sont additives, transactionnelles et testées sur une base de test ; le coffre habituel n'est jamais une cible de migration expérimentale.
- L'annulation de l'aperçu ne modifie aucune donnée persistée.
