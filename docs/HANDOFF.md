# HANDOFF — reprise de SuperPower Alpha

> Dernière mise à jour : 2026-10-14. S06-B et S02 sont intégrés dans `main`. Le Sprint 03 Import bancaire est maintenant le lot actif. Distinguer le code présent, la validation CI et les essais natifs confirmés par l’utilisateur.

## 1. État du dépôt
- Dépôt : https://github.com/tomzpoker/SuperPower-Alpha-0.1
- `main` de référence : `f5affa71bb3c793a1b545fa10360147a4e35f5f7` (S02 fusionné le 14 octobre 2026 via PR #8).
- S06-A et S06-B sont fusionnés dans `main`; la PR [#5 — S06-B, flux prévisionnels récurrents](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/5) a été fusionnée le 13 octobre 2026.
- Le Sprint 02 a été fusionné dans `main` le 14 octobre 2026 via la [PR #8](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/8), commit `f5affa7`. La CI #194 est verte sur son head documentaire : [résultats complets](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38490686495). La CI code #192 est également verte sur `8292edc`, avec les tests du moteur, du worker et du handler `tresorerie/rappel_flux`, l’historique persistant et l’absence de doublon. Les rappels ponctuel/récurrent et le parcours natif verrouillage/déverrouillage ont été confirmés effectués par l’utilisateur ; ne pas demander de les refaire. Le lot actif est S03 sur `sprint/03-import-bancaire`, PR brouillon [#10](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/10), liée à l’[issue #9](https://github.com/tomzpoker/SuperPower-Alpha-0.1/issues/9). Sa première tranche ajoute un parseur CSV de domaine pur ; la persistance SQLCipher et l’interface sont encore à faire. Ne jamais utiliser le coffre habituel comme cible de migration.
- Le run de PR [#96](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38332638917) est vert sur `cd764cb`, y compris build Tauri Windows et tests workspace. Le run post-fusion [#97](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38333882817) est terminé avec succès sur `17966e1`.
- S06-B conserve les prévisions ponctuelles au-delà de trois ans ; pour les récurrences, l'horizon glissant de 36 mois part d'aujourd'hui.

Le socle de `main` contient le coffre SQLCipher/Argon2id, les sociétés et une saisie comptable technique. S06-A ajoute le suivi des flux de trésorerie prévisionnels et réalisés sans choix de comptes comptables.

## 2. État de validation et contrôles encore ouverts

État de validation au 2026-10-14 : S02 est intégré dans `main`. La CI #194 ([run complet](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38490686495)) valide les jobs Windows/Tauri, Rust et frontend pour le commit documentaire de clôture ; la CI code #192 ([résultats](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38488027381)) valide le moteur, le worker et le handler `tresorerie/rappel_flux`, l’historique et l’absence de doublon. Les tests automatisés couvrent également les récurrences simulées, le cycle SQLCipher AppState, les phrases secrètes incorrectes et le redémarrage du worker. L’utilisateur a confirmé les essais natifs de rappel ponctuel, récurrent et verrouillage/déverrouillage ; ne pas redemander ces essais. La CI du parseur CSV de S03 est suivie dans la PR #10.

Résultats manuels signalés par l’utilisateur jusqu’au 2026-10-14 :
- Test natif Sprint 02 sur un coffre de test : la tâche « RAPPEL TEST » (`general/reminder`) a déclenché une notification visible. L’historique a affiché l’exécution terminée et la tentative réussie. La notification, le worker et l’historisation ont été confirmés par l’utilisateur.
- [x] **Essai natif ponctuel du 14/10/2026 :** tâche `test` (`general/reminder`) planifiée pour `02:22:00`, démarrage et fin enregistrés à `02:22:26` ; exécution terminée, tentative `1/3` réussie, durée affichée `0 s`. Le début est donc observé 26 secondes après l’échéance affichée ; l’horodatage de création n’étant pas plus précis que la date, cette mesure est consignée sans conclure à un défaut de ponctualité.
- Test du rappel récurrent : l’utilisateur a confirmé que le scénario est passé. Les dates métier ancrées sont également couvertes par des tests déterministes en CI ; il n’est pas nécessaire d’attendre plusieurs mois pour valider cette logique.
- Le cycle SQLCipher AppState lock/unlock, l’idempotence du worker et le refus de démarrage avec une phrase secrète incorrecte sont automatisés. Le parcours natif réel lock/unlock a aussi été confirmé comme déjà testé par l’utilisateur ; ne pas lui faire répéter ce contrôle.
- Parcours natif du 2026-10-13 confirmé par l'utilisateur : prévision de loyer modifiée à 1 400 € avec 400 € réalisés conservés ; prévision d'assurance de 240 € ; réalisation de 300 € enregistrée puis annulée via l'historique ; historique conservé avec motif d'annulation ; prévision assurance annulée et affichée via le filtre ; totaux observés : 1 400 € prévus à encaisser, 0 € prévus à payer, 400 € réalisés actifs.
- Une autre réalisation de 300 € liée au loyer est également conservée dans l'historique avec motif d'annulation ; le cumul actif du loyer reste 400 €.
- L'utilisateur a fermé puis relancé l'application avec le même dossier temporaire et confirme que la société et les prévisions/réalisations sont revenues.
- Copie de `superpower.db` et `superpower.salt` dans un dossier temporaire : l'application s'est déverrouillée, les données existantes ont été signalées présentes et la réouverture de cette même copie a fonctionné.
- Aucun test n'a utilisé le coffre habituel comme cible de migration ; il a servi uniquement de source pour copier les fichiers.

S06-A et S06-B sont fusionnés et la CI complète du commit source de S06-B est verte. Deux contrôles restent à distinguer :
1. Sur une copie isolée connue en v2, vérifier qu'une ouverture avec le code S06-B migre vers `PRAGMA user_version = 3`, crée `flux_recurrences` et `recurrence_id`, et conserve les flux/réalisations v2.
2. Le garde-fou « refus d'une réalisation sur prévision annulée » a un test unitaire sur la fonction appelée par la commande dans la PR #4, passé en CI ; un test d'intégration invoquant cette commande avec une base native annulée reste souhaitable.

Ne jamais faire ces essais sur le coffre habituel comme cible : copier la paire SQLCipher dans un dossier temporaire distinct et conserver l'original intact.

Ne jamais demander à l'application de migrer la base originale pour « voir ce qui se passe ». L'application de développement reconnaît `SUPERPOWER_TEST_DATA_DIR` uniquement si elle pointe vers un chemin absolu distinct du coffre habituel ; cette variable est ignorée dans les builds release. Le test décrit plus bas doit être lancé avec cette variable définie afin que la copie, et non le coffre habituel, soit ouverte.

## 3. Contrôles locaux Windows
Depuis la racine du dépôt, exécuter les contrôles séparément et noter le résultat exact :
```powershell
python scripts/check_docs.py
npm install
npm test
npm run typecheck
npm run build
cargo fmt --all -- --check
cargo test -p superpower-core
cargo test --workspace
cargo check --workspace
npm run tauri:dev
```
La CI Windows compile Tauri et exécute les tests du workspace. Éviter de lancer plusieurs compilations Cargo en parallèle. Pour SQLCipher/OpenSSL sous Windows, vérifier la présence de Strawberry Perl avec `where.exe perl` ; ne pas retirer SQLCipher comme contournement.

## 4. Test manuel métier — écran natif
Commencer dans un **coffre de test séparé**. Depuis la racine du dépôt, dans PowerShell, lance cette commande (elle crée un chemin temporaire neuf, l'affiche et démarre la fenêtre native) :

```powershell
$env:SUPERPOWER_TEST_DATA_DIR = Join-Path $env:TEMP ('SuperPower-S06A-UI-' + (Get-Date -Format 'yyyyMMdd-HHmmss')); Write-Output $env:SUPERPOWER_TEST_DATA_DIR; npm run tauri:dev
```

Utiliser une société fictive, une phrase secrète de test, et uniquement des montants/données fictifs. La variable reste définie dans cette fenêtre PowerShell pour que la relance de l'application rouvre le même coffre de test.

1. Initialiser/déverrouiller le coffre de test.
2. Ouvrir Trésorerie → prévisionnel et réalisé.
3. Créer une entrée attendue (ex. loyer) et une sortie attendue (ex. assurance) avec montants/dates distincts.
4. Modifier une prévision et vérifier que l'identité de la société ne peut pas changer et que le prévu est conservé séparément du réel.
5. Enregistrer deux réalisations partielles sur une prévision ; vérifier le total et l'écart prévu moins réalisé.
6. Enregistrer un réel supérieur au prévu ; vérifier que le dépassement est visible et que le montant prévu n'est pas écrasé.
7. Annuler une réalisation avec motif ; vérifier que l'historique la conserve mais l'exclut du total.
8. Annuler une prévision ; vérifier qu'elle refuse de nouvelles réalisations et qu'elle apparaît avec le filtre des éléments annulés.
9. Fermer puis relancer l'application et vérifier que les données enregistrées dans le coffre natif persistent.

### Résultat du parcours natif minimal (2026-10-13)

Le parcours natif Trésorerie est **réussi selon le retour utilisateur** pour les actions observées : création de prévision d'assurance (240 €), modification du loyer à 1 400 € sans écraser les 400 € réalisés, création de réalisations de 300 €, annulation de ces réalisations via la section Historique, maintien des lignes annulées avec leur motif et exclusion du cumul, annulation de la prévision d'assurance et visibilité dans le filtre. L'écran final montrait 1 400 € prévus à encaisser, 0 € à payer et 400 € réalisés actifs.

L'utilisateur confirme avoir vu l'état « Réalisé au-delà du prévu » lors de la réalisation de 300 € sur la prévision d'assurance de 240 €, avant d'annuler cette réalisation depuis l'historique. Ce contrôle visuel est validé. Après l'annulation de la prévision d'assurance, les boutons d'action étaient masqués ; le refus côté commande métier d'une nouvelle réalisation n'a pas encore été testé séparément.

Le mode navigateur est uniquement une démo volatile/non chiffrée ; il ne valide ni le coffre ni la persistance native.

## 5. Test manuel de migration SQLCipher v2 → v3 (S06-B)

À effectuer seulement sur une **copie** d'une base v2 de test. Ne jamais lancer une migration de test sur la base habituelle. Il faut copier ensemble le fichier SQLCipher et son fichier de sel ; `SUPERPOWER_TEST_DATA_DIR` est pris en compte en build debug et pas dans les builds release.

**Préparer une copie isolée dans PowerShell depuis la racine du dépôt** (fermer d'abord SuperPower) :

```powershell
$source=Join-Path $env:APPDATA 'fr.superpower.alpha'; $db=Join-Path $source 'superpower.db'; $salt=Join-Path $source 'superpower.salt'; if (!(Test-Path $db) -or !(Test-Path $salt)) { throw "Base ou fichier de sel introuvable dans $source. Stoppe ici et localise la paire correcte." }; $testDir=Join-Path $env:TEMP ('SuperPower-S06B-migration-'+(Get-Date -Format 'yyyyMMdd-HHmmss')); New-Item -ItemType Directory -Path $testDir | Out-Null; Copy-Item $db (Join-Path $testDir 'superpower.db'); Copy-Item $salt (Join-Path $testDir 'superpower.salt'); Write-Output "Dossier isolé : $testDir"
```

Avant d'ouvrir la copie avec la branche S06-B, lire son `PRAGMA user_version` au moyen d'un outil compatible SQLCipher et confirmer que la source est en v2. Si la version ne peut pas être relevée, documenter le résultat comme un simple test de démarrage, pas comme une preuve formelle de migration v2→v3. Ne pas utiliser une sauvegarde inconnue pour conclure à une migration précise.

Lancer l'application en pointant vers le dossier temporaire exact affiché par la commande :

```powershell
$env:SUPERPOWER_TEST_DATA_DIR=$testDir; npm run tauri:dev
```

1. Déverrouiller la copie avec sa phrase secrète. Seul le dossier temporaire peut être modifié par ce test.
2. Vérifier que l'application s'ouvre et que la société et les données v2 préexistantes (prévisions et réalisations) sont conservées.
3. Avec un outil compatible SQLCipher, vérifier après migration `PRAGMA user_version = 3`, la table `flux_recurrences` et la colonne nullable `flux_previsionnels.recurrence_id`.
4. Vérifier que chaque flux v2 existant reste ponctuel : `recurrence_id IS NULL`, montant/libellé/date prévus préservés et réalisations intactes.
5. Créer un flux récurrent mensuel de test et vérifier que les échéances futures s'affichent, que les dates de fin de mois ne dérivent pas et que chaque occurrence a son propre historique.
6. Fermer puis rouvrir l'application en gardant le même `SUPERPOWER_TEST_DATA_DIR`. Vérifier la persistance et l'absence de doublons après le rechargement.
7. En cas d'échec, arrêter les essais, garder la copie et les journaux pour diagnostic. Ne jamais remplacer la base habituelle avec cette copie.

Pour revenir au fonctionnement normal dans cette fenêtre PowerShell : `Remove-Item Env:SUPERPOWER_TEST_DATA_DIR`.

### Historique du précédent essai (2026-10-13)

L'utilisateur a précédemment copié `superpower.db` et `superpower.salt` vers un dossier temporaire, confirmé que le coffre s'ouvrait et que les données restaient visibles après réouverture. La version source n'a pas été relevée indépendamment ; cet essai ne constitue pas une preuve de migration v1→v2 ni v2→v3.

Les tests automatisés utilisent des bases en mémoire ; ils ne prouvent pas à eux seuls le déchiffrement et la migration d'un fichier SQLCipher existant sur Windows.

## 6. Conventions de développement
- Montants en centimes entiers côté Rust (`i64`) ; frontend via `parseAmountToCents`.
- IPC en camelCase, enums en snake_case, erreurs affichables en français.
- SQLCipher obligatoire ; aucun repli vers une base non chiffrée.
- Migration transactionnelle, versionnée et sans reconstruction des tables v1 ; refuser les versions inconnues futures.
- Journaliser chaque mutation dans la même transaction que la donnée.
- Ne jamais déduire automatiquement une écriture PCG ou la TVA d'une catégorie, d'un libellé, de la forme juridique ou du régime IR.
- Un contrôle non exécuté reste « non vérifié ».

## 7. Limites fonctionnelles
S06-A et S06-B ne génèrent ni écritures comptables ni TVA. S06-B est livré dans `main`; la projection de trésorerie dédiée au-delà des échéances, les alertes, l'import/rapprochement bancaire, les baux, les règles PCG multi-lignes, la fiscalité et le FEC restent des lots séparés.

## 8. Prochaine reprise — Sprint 03 Import bancaire

1. S02 est intégré dans `main` via la PR #8 ; ne pas redemander les tests natifs déjà effectués.
2. Vérifier la CI de la [PR #10](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/10) pour le parseur de domaine CSV et corriger les échecs éventuels avant d’ajouter la persistance SQLCipher.
3. Ajouter ensuite le modèle de transactions bancaires, le compte/source, une empreinte déterministe, une migration additive transactionnelle et la détection idempotente des doublons. Ensuite, relier le parseur aux commandes Tauri puis à l’écran d’aperçu/confirmation.
4. L’import ne doit jamais créer automatiquement une écriture PCG ou calculer la TVA. La migration S06-B v2→v3 reste un contrôle séparé sur copie SQLCipher isolée ; ne jamais la tester sur le coffre habituel.

Le mode navigateur reste une démo volatile et ne valide ni chiffrement ni migration réelle SQLCipher.
