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
| **Statut** | IN_PROGRESS — première tranche de domaine publiée dans la PR #10 : parseur CSV pur, mapping explicite, aperçu et erreurs par ligne. CI à confirmer ; persistance SQLCipher et interface à venir |

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
- [x] S03-A : Normaliser les dates ISO/françaises, gérer les montants signés et les colonnes débit/crédit, convertir en centimes entiers, collecter les erreurs par ligne.
- [ ] S03-B : Étendre les tests et vérifier la tranche de parseur en CI (formatage, tests du workspace et build Tauri Windows).
- [ ] S03-C : Définir la persistance SQLCipher : compte/source, empreinte stable de transaction, provenance du fichier et migration additive transactionnelle.
- [ ] S03-D : Ajouter les commandes Tauri de prévisualisation/confirmation, avec protection contre les doubles imports.
- [ ] S03-E : Ajouter l'écran d'import (sélection du fichier, mapping explicite, aperçu, erreurs, confirmation/annulation).
- [ ] S03-F : Couvrir l'idempotence, la conservation des données et l'absence de génération implicite d'écritures PCG/TVA.

## Tranche S03-A — parseur de domaine
Le module `superpower-core::bank_import` n'accède ni au système de fichiers ni à la base. Il reçoit le contenu CSV, un délimiteur, l'indication d'en-tête et un `CsvColumnMapping` explicite. Il retourne les en-têtes, les lignes valides normalisées et les erreurs associées aux enregistrements rejetés.

Montants pris en charge : entiers, décimales à un ou deux chiffres, virgule française, point décimal, groupements de milliers par espaces ou séparateurs cohérents, symbole euro et préfixe/suffixe EUR. L'en-tête UTF-8 avec BOM est reconnu ; les fichiers de plus de 10 Mio sont rejetés avant parsing. Si le format est ambigu, la ligne est rejetée plutôt que devinée.

Les tests unitaires du parseur couvrent les formats français/internationaux, les en-têtes BOM UTF-8, les colonnes débit/crédit manquantes, les dates impossibles, les montants ambigus, les guillemets échappés et les champs multilignes. Le résultat CI sur le head courant doit être vérifié avant de déclarer cette tranche validée.

## Critères d'acceptation globaux
- Un CSV valide produit un aperçu avant toute écriture en base.
- Les dates impossibles, montants mal formés, lignes incomplètes et mappings contradictoires sont signalés.
- Les guillemets CSV échappés et les champs contenant des retours à la ligne sont interprétés sans découper la ligne par erreur.
- L'import de la même transaction est idempotent, tout en conservant les lignes sources et les cas de collisions pour inspection.
- Les migrations SQLCipher sont additives, transactionnelles et testées sur une base de test ; le coffre habituel n'est jamais une cible de migration expérimentale.
- L'annulation de l'aperçu ne modifie aucune donnée persistée.
