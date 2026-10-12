# SuperPower Alpha 0.1

Application desktop locale de pilotage métier, comptable et fiscal pour plusieurs entités françaises (notamment SCI et SARL).

> **État : alpha en développement, pas encore un logiciel comptable/fiscal complet.** \`main\` contient le coffre local SQLCipher, le registre des sociétés, le socle comptable et la CI Windows validée. La branche \`feat/guided-cash-flows\` ajoute le premier parcours de trésorerie guidé : flux prévu, réalisations partielles, écart et historique, sans choix de comptes. Ce lot ne génère pas encore d'écritures comptables ni de calcul TVA. Import bancaire, gestion locative et fiscalité déclarative restent à développer.

## Stack
- Rust workspace : \`crates/core\` pour le domaine ; \`src-tauri\` pour Tauri/persistance.
- Tauri 2 ; React 18 + TypeScript + Vite.
- SQLite chiffré via SQLCipher ; clé dérivée Argon2id.
- Montants comptables en centimes entiers (\`i64\`).

## Démarrage Windows
Pré-requis : Node.js 22+, npm, Rust stable, Visual Studio Build Tools (C++ Desktop), WebView2 et Strawberry Perl dans le PATH pour OpenSSL vendored. Voir [prérequis Tauri](https://v2.tauri.app/start/prerequisites/) et [notes OpenSSL Windows](https://github.com/openssl/openssl/blob/master/NOTES-WINDOWS.md).

Depuis la racine du dépôt :
\`\`\`powershell
npm install
npm test
npm run typecheck
npm run build
cargo fmt --all -- --check
cargo test -p superpower-core
cargo test --workspace
npm run tauri:dev
\`\`\`
Pour un aperçu sans fenêtre native : \`npm run dev\`. Il est volatile, non chiffré et ne doit jamais contenir de données réelles.

## Sécurité
- Ne pas retirer SQLCipher pour contourner une erreur de compilation.
- Sauvegarder ensemble la base chiffrée et son fichier \`.salt\`, et conserver la phrase secrète dans un gestionnaire sécurisé.
- L'alpha n'est pas encore un logiciel certifié ni une validation de conformité légale.
- Les écritures validées sont verrouillées dans l'interface ; les triggers SQL d'immutabilité, la contre-passation, l'audit complet, le FEC validé et les déclarations fiscales restent à construire.
- Les flux S06-A sont des données de pilotage ; aucune écriture PCG ou TVA n'est inférée automatiquement.

## Documentation
Commencer par [\`AGENTS.md\`](AGENTS.md), [\`docs/HANDOFF.md\`](docs/HANDOFF.md), [\`docs/PROGRESS.md\`](docs/PROGRESS.md) et [\`docs/README.md\`](docs/README.md).
Dépôt : https://github.com/tomzpoker/SuperPower-Alpha-0.1
