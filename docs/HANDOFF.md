# HANDOFF — reprise de SuperPower Alpha

> Dernière mise à jour : 2026-10-13. S06-A est fusionné ; S06-B est en cours dans la PR #5. Distinguer compilation CI, test unitaire et essai natif SQLCipher.

## 1. État du dépôt
- Dépôt : https://github.com/tomzpoker/SuperPower-Alpha-0.1
- `main` de référence : `7f87caf9d878db0ae2e254fcbd4f01b7acc48a2c`.
- S06-A est fusionné ; les PR #3 (spécification S06-B) et #4 (test du garde-fou des flux annulés) sont également fusionnées.
- PR de travail : [#5 — S06-B, flux prévisionnels récurrents](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/5), branche `feat/s06b-recurrence-domain`.
- Le run [#88](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38331494863) a passé frontend et Rust sur `14e755b`; build Tauri Windows et tests workspace étaient encore en cours au dernier relevé. Ne pas fusionner avant résultat vert sur le dernier SHA de la PR.

Le socle de `main` contient le coffre SQLCipher/Argon2id, les sociétés et une saisie comptable technique. S06-A ajoute le suivi des flux de trésorerie prévisionnels et réalisés sans choix de comptes comptables.

## 2. État de validation et contrôles encore ouverts

Résultats manuels signalés le 2026-10-13 :
- Parcours natif du 2026-10-13 confirmé par l'utilisateur : prévision de loyer modifiée à 1 400 € avec 400 € réalisés conservés ; prévision d'assurance de 240 € ; réalisation de 300 € enregistrée puis annulée via l'historique ; historique conservé avec motif d'annulation ; prévision assurance annulée et affichée via le filtre ; totaux observés : 1 400 € prévus à encaisser, 0 € prévus à payer, 400 € réalisés actifs.
- Une autre réalisation de 300 € liée au loyer est également conservée dans l'historique avec motif d'annulation ; le cumul actif du loyer reste 400 €.
- L'utilisateur a fermé puis relancé l'application avec le même dossier temporaire et confirme que la société et les prévisions/réalisations sont revenues.
- Copie de `superpower.db` et `superpower.salt` dans un dossier temporaire : l'application s'est déverrouillée, les données existantes ont été signalées présentes et la réouverture de cette même copie a fonctionné.
- Aucun test n'a utilisé le coffre habituel comme cible de migration ; il a servi uniquement de source pour copier les fichiers.

S06-A est fusionné et ses scénarios manuels de base sont documentés. Deux contrôles natifs restent à distinguer :
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

## 5. Test manuel de migration SQLCipher v1 → v2
À effectuer seulement avec une base v1 de test (ou une copie locale d'une base v1, si tu es certain de pouvoir y conserver les données sur la machine). Garder le fichier d'origine intact. La variable de développement `SUPERPOWER_TEST_DATA_DIR` permet de lancer la build debug sur un dossier séparé ; elle n'est pas honorée par les builds release.

**Préparer le dossier isolé dans PowerShell depuis la racine du dépôt** (fermer d'abord SuperPower). Cette commande s'arrête si la paire de fichiers n'est pas trouvée :

```powershell
$source=Join-Path $env:APPDATA 'fr.superpower.alpha'; $db=Join-Path $source 'superpower.db'; $salt=Join-Path $source 'superpower.salt'; if (!(Test-Path $db) -or !(Test-Path $salt)) { throw "Base ou fichier de sel introuvable dans $source. Stoppe ici et localise la paire correcte." }; $testDir=Join-Path $env:TEMP ('SuperPower-S06A-migration-'+(Get-Date -Format 'yyyyMMdd-HHmmss')); New-Item -ItemType Directory -Path $testDir | Out-Null; Copy-Item $db (Join-Path $testDir 'superpower.db'); Copy-Item $salt (Join-Path $testDir 'superpower.salt'); Write-Output "Dossier isolé : $testDir"
```

Vérifie que la source est bien une base v1 non encore migrée par S06-A. Si l'application a déjà été ouverte avec cette branche, la base source peut être passée en v2 ; dans ce cas, ne prétends pas avoir testé une migration v1 → v2. Utilise une sauvegarde v1 connue. Ne prends pas une base contenant des données que tu n'as pas le droit d'utiliser dans un test.

Lance ensuite l'application en utilisant **le chemin isolé qui vient d'être créé** :

```powershell
$env:SUPERPOWER_TEST_DATA_DIR=$testDir; npm run tauri:dev
```

1. Déverrouiller avec la phrase secrète associée au fichier copié. Le déverrouillage déclenche la migration transactionnelle de ce dossier uniquement.
2. Vérifier que le déverrouillage réussit et que l'application arrive à l'état prêt.
3. Vérifier que les sociétés, écritures, lignes et entrées d'audit préexistantes sont toujours présentes avec les mêmes valeurs.
4. Vérifier que les tables `flux_previsionnels` et `flux_realisations` existent et que `PRAGMA user_version` vaut `2`, avec un outil compatible SQLCipher si disponible.
5. Fermer puis rouvrir l'application en conservant la même variable `SUPERPOWER_TEST_DATA_DIR` ; confirmer que le deuxième déverrouillage fonctionne sans duplication.
6. Créer un flux de test, fermer l'application, puis la relancer pour vérifier que le flux persiste.
7. En cas d'échec, arrêter les essais, conserver la copie et les journaux. Ne jamais remettre le dossier temporaire à la place du dossier original.

Pour revenir au fonctionnement normal dans cette fenêtre PowerShell : `Remove-Item Env:SUPERPOWER_TEST_DATA_DIR`. La variable n'a aucun effet sur un build release.

### Résultat de l'essai sur copie native (2026-10-13)

L'utilisateur a copié `superpower.db` et `superpower.salt` dans un dossier temporaire, lancé la branche avec `SUPERPOWER_TEST_DATA_DIR`, puis confirmé que l'application s'ouvrait, que les données existantes étaient visibles et que le deuxième lancement avec la même copie fonctionnait. Le dossier de l'application habituelle n'a pas été utilisé comme cible de migration.

**Réserve importante :** la version initiale du fichier source n'a pas été relevée indépendamment. Ne pas présenter cet essai comme preuve définitive d'une migration v1 → v2 tant que `PRAGMA user_version` de la source et de la copie migrée n'a pas été confirmé, ainsi que la présence des tables S06-A.

Le test automatisé utilise SQLite en mémoire et le schéma v1 complet représentatif ; il augmente la confiance en la conservation des données, mais ne prouve pas l'ouverture, le déchiffrement et la migration d'un fichier SQLCipher existant sur Windows.

## 6. Conventions de développement
- Montants en centimes entiers côté Rust (`i64`) ; frontend via `parseAmountToCents`.
- IPC en camelCase, enums en snake_case, erreurs affichables en français.
- SQLCipher obligatoire ; aucun repli vers une base non chiffrée.
- Migration transactionnelle, versionnée et sans reconstruction des tables v1 ; refuser les versions inconnues futures.
- Journaliser chaque mutation dans la même transaction que la donnée.
- Ne jamais déduire automatiquement une écriture PCG ou la TVA d'une catégorie, d'un libellé, de la forme juridique ou du régime IR.
- Un contrôle non exécuté reste « non vérifié ».

## 7. Limites fonctionnelles
S06-A et S06-B ne génèrent ni écritures comptables ni TVA. Les occurrences récurrentes sont en cours d'implémentation sous PR #5 ; la projection de trésorerie dédiée au-delà des échéances, les alertes, l'import/rapprochement bancaire, les baux, les règles PCG multi-lignes, la fiscalité et le FEC restent des lots séparés.

## 8. Suite après validation

1. Obtenir le résultat vert de la CI sur le dernier SHA de la [PR #5](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/5) avant de la fusionner.
2. Après fusion, exécuter le test manuel de la migration v2→v3 sur une copie isolée SQLCipher et vérifier `user_version = 3`, les tables/colonnes attendues et la conservation de flux/réalisations.
3. Tester dans l'interface native les quatre choix (ponctuel, mensuel, trimestriel, annuel), l'ancrage des fins de mois, l'arrêt d'une série et l'indépendance de chaque occurrence. Le navigateur reste un mode démo volatil, pas une preuve du coffre chiffré.
4. Maintenir la comptabilisation PCG/TVA dans un lot séparé ; S06-B ne génère aucune écriture comptable.
