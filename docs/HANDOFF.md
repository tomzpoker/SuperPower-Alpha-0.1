# HANDOFF — reprise de SuperPower Alpha

> Dernière mise à jour : 2026-10-14. S06-B est fusionné dans `main`; le Sprint 02 est toujours en cours sur une PR distincte. Distinguer code présent, CI verte et essai natif.

## 1. État du dépôt
- Dépôt : https://github.com/tomzpoker/SuperPower-Alpha-0.1
- `main` de référence : `535baa6431fe87dfaa165ad1e9b7476f753d7c49` (S06-B fusionné ; head vérifié le 14 octobre 2026).
- S06-A et S06-B sont fusionnés dans `main`; la PR [#5 — S06-B, flux prévisionnels récurrents](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/5) a été fusionnée le 13 octobre 2026.
- Le Sprint 02 se poursuit sur `sprint/02-moteur-taches-prototype` (PR [#8](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/8), encore en brouillon et non fusionnée). Le prototype comprend le moteur persistant SQLCipher, les migrations applicatives v3→v6 sur cette branche, l’IPC Tauri, la page Tâches, le worker supervisé, les rappels locaux, les récurrences métier ancrées, le calendrier interactif, les filtres et l’historique des exécutions/tentatives. La CI la plus récente, [run #191](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38461882586), est entièrement verte sur `7683c88` (jobs Windows/Tauri, Rust et frontend), notamment le test d’intégration worker + `tresorerie/rappel_flux` avec historique et absence de doublon. Les essais ponctuels et récurrent de rappel ont été confirmés réussis par l’utilisateur. Le contrôle manuel restant sur S02 est le parcours des commandes Tauri de verrouillage/déverrouillage dans l’interface native. Ne jamais utiliser le coffre habituel comme cible de migration.
- Le run de PR [#96](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38332638917) est vert sur `cd764cb`, y compris build Tauri Windows et tests workspace. Le run post-fusion [#97](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38333882817) est terminé avec succès sur `17966e1`.
- S06-B conserve les prévisions ponctuelles au-delà de trois ans ; pour les récurrences, l'horizon glissant de 36 mois part d'aujourd'hui.

Le socle de `main` contient le coffre SQLCipher/Argon2id, les sociétés et une saisie comptable technique. S06-A ajoute le suivi des flux de trésorerie prévisionnels et réalisés sans choix de comptes comptables.

## 2. État de validation et contrôles encore ouverts

État de validation au 2026-10-14 : la CI #191, [run complet](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38461882586), est verte sur le head de la PR #8 (`7683c88`) pour les jobs Windows/Tauri, Rust et frontend. Elle passe après l’ajout du test d’intégration du handler `tresorerie/rappel_flux` via `TaskWorker`, avec solde calculé depuis SQLCipher, historique de succès et absence de doublon à la passe suivante. Les tests automatisés vérifient également l’idempotence, l’historique, les dates de récurrence simulées, le cycle AppState lock/unlock, la phrase secrète incorrecte et le reverrouillage si le worker échoue. Ces tests n’établissent pas à eux seuls que le clic réel sur les commandes Tauri de verrouillage/déverrouillage fonctionne dans l’application native.

Résultats manuels signalés par l’utilisateur jusqu’au 2026-10-14 :
- Test natif Sprint 02 sur un coffre de test : la tâche « RAPPEL TEST » (`general/reminder`) a déclenché une notification visible. L’historique a affiché l’exécution terminée et la tentative réussie. La notification, le worker et l’historisation ont été confirmés par l’utilisateur.
- Test du rappel récurrent : l’utilisateur a confirmé que le scénario est passé. Les dates métier ancrées sont également couvertes par des tests déterministes en CI ; il n’est pas nécessaire d’attendre plusieurs mois pour valider cette logique.
- Le cycle SQLCipher AppState lock/unlock, l’idempotence du worker et le refus de démarrage avec une phrase secrète incorrecte sont automatisés. Le parcours graphique réel des commandes Tauri verrouillage/déverrouillage reste à vérifier manuellement.
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

## 8. Prochaine reprise — Sprint 02 puis Sprint 03

1. **Contrôle manuel restant de S02 :** sur un coffre temporaire neuf, verrouiller puis déverrouiller depuis l’interface native et vérifier que le worker s’arrête/reprend, que l’historique reste visible et qu’aucune occurrence n’est exécutée deux fois. La logique SQLCipher/AppState, les erreurs de phrase secrète et le redémarrage du worker sont déjà couverts en CI ; ne pas refaire les tests de rappel déjà validés.
2. Après ce contrôle, relire le diff et les résultats CI actuels de la [PR #8](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/8). Elle est encore en brouillon : ne pas la considérer fusionnée. Faire la revue finale puis décider de la passer en « Ready for review » et de la fusionner.
3. Après clôture de S02, préparer **S03 — Import bancaire** : périmètre CSV documenté, aperçu avant import, validation des dates/montants, détection des doublons et aucune création automatique d’écriture PCG/TVA sans règle validée.
4. **Contrôle séparé de S06-B toujours ouvert :** sur une copie d’une base SQLCipher dont `PRAGMA user_version = 2` a été relevé avant l’essai, confirmer après ouverture `user_version = 3`, `flux_recurrences`, `flux_previsionnels.recurrence_id` et la conservation des flux/réalisations. Ne jamais utiliser le coffre habituel comme cible de migration.

Le mode navigateur reste une démo volatile et ne valide ni chiffrement ni migration réelle SQLCipher.