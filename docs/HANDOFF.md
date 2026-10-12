# HANDOFF — reprise de SuperPower Alpha

> Dernière mise à jour : 2026-10-13. Toujours distinguer `main` de la branche S06-A et une compilation CI d'un test métier réel.

## 1. État du dépôt
- Dépôt : https://github.com/tomzpoker/SuperPower-Alpha-0.1
- `main` de référence : `5078de2`.
- Branche de travail : `feat/guided-cash-flows`, pull request [#1 — Sprint 06](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/1).
- Le dernier run complètement vert avant le renforcement du test de migration est [38316144097](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38316144097). Le nouveau test contient désormais une société, une écriture, deux lignes comptables et une ligne d'audit sur le schéma v1 complet. Vérifier la CI de la révision la plus récente avant de déclarer le lot validé.

Le socle de `main` contient le coffre SQLCipher/Argon2id, les sociétés et une saisie comptable technique. S06-A ajoute le suivi des flux de trésorerie prévisionnels et réalisés sans choix de comptes comptables.

## 2. Ce qui doit rester IN_PROGRESS
Ne pas fusionner ou déclarer S06-A terminé avant :
1. CI verte sur la révision actuelle.
2. Test manuel sur une **copie** d'une vraie base chiffrée SQLCipher v1, avec vérification de la conservation des sociétés, écritures, lignes et journal d'audit.
3. Parcours manuel de l'écran Trésorerie de l'application native.
4. Mise à jour du présent handoff, de `PROGRESS.md` et des cases de l'ADR avec les résultats réellement observés.

Ne jamais demander à l'application de migrer la base originale pour « voir ce qui se passe ». L'application de développement reconnaît `SUPERPOWER_TEST_DATA_DIR` pour utiliser un dossier distinct ; cette variable est ignorée dans les builds release. Le test décrit plus bas doit être lancé avec cette variable définie afin que la copie, et non le coffre habituel, soit ouverte.

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
Utiliser une société de test dans l'application native et noter les résultats de chaque étape.

1. Initialiser/déverrouiller le coffre de test.
2. Ouvrir Trésorerie → prévisionnel et réalisé.
3. Créer une entrée attendue (ex. loyer) et une sortie attendue (ex. assurance) avec montants/dates distincts.
4. Modifier une prévision et vérifier que l'identité de la société ne peut pas changer et que le prévu est conservé séparément du réel.
5. Enregistrer deux réalisations partielles sur une prévision ; vérifier le total et l'écart prévu moins réalisé.
6. Enregistrer un réel supérieur au prévu ; vérifier que le dépassement est visible et que le montant prévu n'est pas écrasé.
7. Annuler une réalisation avec motif ; vérifier que l'historique la conserve mais l'exclut du total.
8. Annuler une prévision ; vérifier qu'elle refuse de nouvelles réalisations et qu'elle apparaît avec le filtre des éléments annulés.
9. Fermer puis relancer l'application et vérifier que les données enregistrées dans le coffre natif persistent.

Le mode navigateur est uniquement une démo volatile/non chiffrée ; il ne valide ni le coffre ni la persistance native.

## 5. Test manuel de migration SQLCipher v1 → v2
À effectuer seulement avec une base v1 de test (ou une copie locale d'une base v1, si tu es certain de pouvoir y conserver les données sur la machine). Garder le fichier d'origine intact. La variable de développement `SUPERPOWER_TEST_DATA_DIR` permet de lancer la build debug sur un dossier séparé ; elle n'est pas honorée par les builds release.

**Préparer le dossier isolé dans PowerShell depuis la racine du dépôt** (fermer d'abord SuperPower) :

```powershell
$source = Join-Path $env:APPDATA 'fr.superpower.alpha'
$db = Join-Path $source 'superpower.db'
$salt = Join-Path $source 'superpower.salt'
if (!(Test-Path $db) -or !(Test-Path $salt)) { throw "Base ou fichier de sel introuvable dans $source. Ne poursuis pas avant d'avoir localisé la paire correcte." }
$testDir = Join-Path $env:TEMP ('SuperPower-S06A-migration-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $testDir | Out-Null
Copy-Item $db (Join-Path $testDir 'superpower.db')
Copy-Item $salt (Join-Path $testDir 'superpower.salt')
Write-Output "Dossier isolé : $testDir"
```

Vérifie que la source est bien une base v1 non encore migrée par S06-A. Si l'application a déjà été ouverte avec cette branche, la base source peut être passée en v2 ; dans ce cas, ne prétends pas avoir testé une migration v1 → v2. Utilise une sauvegarde v1 connue. Ne prends pas une base contenant des données que tu n'as pas le droit d'utiliser dans un test.

Lance ensuite l'application en utilisant **le chemin isolé qui vient d'être créé** :

```powershell
$env:SUPERPOWER_TEST_DATA_DIR = $testDir
npm run tauri:dev
```

1. Déverrouiller avec la phrase secrète associée au fichier copié. Le déverrouillage déclenche la migration transactionnelle de ce dossier uniquement.
2. Vérifier que le déverrouillage réussit et que l'application arrive à l'état prêt.
3. Vérifier que les sociétés, écritures, lignes et entrées d'audit préexistantes sont toujours présentes avec les mêmes valeurs.
4. Vérifier que les tables `flux_previsionnels` et `flux_realisations` existent et que `PRAGMA user_version` vaut `2`, avec un outil compatible SQLCipher si disponible.
5. Fermer puis rouvrir l'application en conservant la même variable `SUPERPOWER_TEST_DATA_DIR` ; confirmer que le deuxième déverrouillage fonctionne sans duplication.
6. Créer un flux de test, fermer l'application, puis la relancer pour vérifier que le flux persiste.
7. En cas d'échec, arrêter les essais, conserver la copie et les journaux. Ne jamais remettre le dossier temporaire à la place du dossier original.

Pour revenir au fonctionnement normal dans cette fenêtre PowerShell : `Remove-Item Env:SUPERPOWER_TEST_DATA_DIR`. La variable n'a aucun effet sur un build release.

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
S06-A ne génère ni écritures comptables ni TVA. Les récurrences, projections 12/24/36 mois, alertes, import/rapprochement bancaire, baux, règles PCG multi-lignes, fiscalité et FEC restent à concevoir/livrer dans leurs lots dédiés.

## 8. Suite après validation
Une fois les tests manuels consignés, fusionner la PR si les contrôles sont verts, puis prioriser les tests d'intégration des commandes/audit. Les prochains choix métier à spécifier sont la génération PCG/TVA, les emprunts (principal/intérêts), les apports/CCA, les dépôts/garanties, les paiements partiels, les avoirs et les contre-passations.
