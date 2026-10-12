# HANDOFF — reprise de SuperPower Alpha

> Lire après \`AGENTS.md\`. Dernière mise à jour : 2026-10-13. Distinguer l'état de \`main\` de la branche S06-A.

## 1. État du dépôt
- Dépôt : [tomzpoker/SuperPower-Alpha-0.1](https://github.com/tomzpoker/SuperPower-Alpha-0.1).
- \`main\` : \`5078de2\`, CI 3/3 verte selon le propriétaire le 2026-10-13.
- Branche en cours : \`feat/guided-cash-flows\`, premier lot métier de trésorerie S06-A ; la CI doit réussir avant fusion.
- Lire [ADR-0001](DECISIONS/ADR-0001-flux-previsionnels.md) et [Sprint 06](SPRINTS/SPRINT_06_TRESORERIE.md) avant de modifier le modèle.

Main contient le coffre local, les sociétés, un petit socle comptable et une saisie technique à deux lignes. S06-A rend Trésorerie utilisable pour prévoir et enregistrer le réel, mais ne transforme pas encore les flux en écritures ou TVA.

## 2. Tests connus
| Contrôle | Résultat | Portée |
|---|---|---|
| \`cargo fmt --all -- --check\` | OK sur main précédent | Refaire sur S06-A |
| \`cargo test -p superpower-core\` | 8/8 sur l'ancien main | Nouveaux tests à exécuter |
| \`cargo check --workspace\` | OK sur main précédent | Pas une validation de S06-A |
| \`npm run typecheck\` | OK sur main précédent | À rejouer avec l'écran |
| GitHub Actions main | 3/3 verts, confirmation propriétaire | S06-A à vérifier |
| Migration v1→v2 native | À tester sur copie | Ne pas risquer la base originale |

Ne pas déclarer S06-A terminé avant la CI de la branche et un test de migration sur copie d'une base native existante.

## 3. Contrôles locaux Windows
\`\`\`powershell
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
\`\`\`
Le CI Windows compile Tauri et exécute les tests workspace. Éviter les compilations Cargo parallèles. Pour SQLCipher/OpenSSL Windows, \`where.exe perl\` doit trouver Strawberry Perl ; ne pas retirer SQLCipher pour contourner une erreur.

## 4. Parcours manuel S06-A
Utiliser une société de test et ne pas toucher aux flux de production.
1. Déverrouiller le coffre natif.
2. Ouvrir Trésorerie → Prévisionnel et réalisé.
3. Créer un flux : société, sens, catégorie, libellé, date, montant, tiers/référence facultatifs.
4. Modifier la prévision et vérifier que les réalisations restent liées.
5. Ajouter une réalisation partielle puis une deuxième ; vérifier total et écart.
6. Annuler une réalisation avec motif ; vérifier historique et baisse du cumul.
7. Annuler une prévision ; vérifier qu'elle refuse une nouvelle réalisation et reste visible via le filtre.
8. Sur copie d'une base v1, vérifier \`PRAGMA user_version = 2\) et l'intégrité des sociétés/écritures existantes.

**Limite :** aucun journal PCG ni calcul TVA n'est créé. Un flux réalisé ne suffit pas à déterminer la qualification comptable/fiscale.

## 5. Conventions
- Centimes entiers Rust \`i64\`, frontend via \`parseAmountToCents\`.
- IPC \`camelCase\`, enums \`snake_case\`, erreurs françaises.
- Commande Tauri ajoutée à \`commands.rs\` et \`src-tauri/src/lib.rs\`, DTO dans \`types.ts\`, mode navigateur dans \`api.ts\`.
- SQLCipher obligatoire ; aucun fallback non chiffré.
- Migration versionnée, transactionnelle, préservant les données ; refuser les versions futures.
- Audit de mutation dans la même transaction.
- Ne pas inférer PCG/TVA depuis catégorie, libellé, forme juridique ou régime IR.
- Mode navigateur volatil/non chiffré ; aucun usage réel.
- Un test non exécuté reste non vérifié.

## 6. Suite recommandée
1. Valider PR S06-A (CI/tests/build natif) et mettre ce handoff à jour.
2. Tester migration sur copie SQLCipher v1.
3. Ajouter tests d'intégration des commandes et de l'audit.
4. Ajouter flux récurrents et projection 12/24/36 mois.
5. Ajouter import CSV/rapprochement (03), puis baux (05).
6. Spécifier règles multi-lignes/TVA : principal-intérêts, apports/CCA, dépôts/garanties, paiements partiels, avoirs et contre-passations.
7. Renforcer triggers d'immutabilité, sauvegardes/restaurations, périodes et FEC.

## 7. Fin de session
- Exécuter \`python scripts/check_docs.py\` et les contrôles possibles, consigner les résultats exacts.
- Mettre à jour \`PROGRESS.md\`, le sprint, l'ADR si besoin et ce handoff.
- Ne pas cocher DONE sans preuve de CI et tests manuels.
