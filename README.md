# SuperPower Alpha 0.1

Application desktop locale de pilotage métier, comptable et fiscal pour plusieurs entités françaises (notamment SCI et SARL). Le dépôt est initialisé à partir du cahier des charges fourni ; il ne reprend aucun code d'un ancien projet.

> **Deux README, deux rôles** : ce fichier est l'entrée du dépôt (présentation, démarrage, sécurité). L'index de la documentation, les règles de développement et les commandes de contrôle sont dans [`docs/README.md`](docs/README.md). Une IA ou un développeur qui reprend le code commence par [`AGENTS.md`](AGENTS.md) puis [`docs/HANDOFF.md`](docs/HANDOFF.md).

> **État : première brique testable, pas encore un logiciel comptable/fiscal complet.** Cette version met en place une base SQLite chiffrée SQLCipher, le déverrouillage par phrase secrète, le registre des sociétés, un socle de comptes et l'enregistrement d'écritures équilibrées. Les modules TVA déclarative, 2072/2044, liasse 2065, import bancaire, documents, locatif et calendrier fiscal restent à développer.

## Stack

- Rust workspace : `crates/core` pour le domaine et la validation ; `src-tauri` pour l'application native et la persistance.
- Tauri 2 ; React 18 + TypeScript + Vite.
- SQLite chiffré via SQLCipher, clé dérivée avec Argon2id. La phrase secrète est demandée au lancement et n'est pas stockée par l'application.
- Montants comptables stockés en centimes entiers (`i64`) : aucun flottant binaire pour les montants.

## Démarrage sur Windows

Pré-requis : Node.js 22+, npm, Rust stable (rustup), Visual Studio Build Tools avec le composant C++ Desktop, WebView2, et **Strawberry Perl** dans le `PATH` pour compiler l’OpenSSL utilisé par SQLCipher. NASM peut aussi être requis selon la configuration de compilation. Voir les prérequis Tauri : <https://v2.tauri.app/start/prerequisites/> et les notes OpenSSL Windows : <https://github.com/openssl/openssl/blob/master/NOTES-WINDOWS.md>. Après toute installation d’un outil, fermer puis rouvrir PowerShell.

Le premier build doit produire les ressources web avant la compilation native. Si npm indique que les scripts `esbuild` sont bloqués et que Vite échoue ensuite avec une erreur explicite sur son binaire, examine `npm install-scripts ls` puis approuve uniquement le paquet attendu avec `npm install-scripts approve esbuild` avant de relancer `npm install`. Ne lance pas d’approbation globale de scripts non examinés :

Dans PowerShell, depuis la racine du dépôt :

```powershell
npm install
npm test
npm run typecheck
npm run build
cargo test -p superpower-core
cargo test --workspace
npm run tauri:dev
```

Pour vérifier le frontend sans fenêtre native :

```powershell
npm run dev
```

Le mode navigateur est une **démo volatile** : il n'est pas chiffré et ne conserve pas les données après rechargement. Il ne doit jamais contenir de données réelles.

## Tests

```powershell
npm test
npm run typecheck
npm run build
cargo test -p superpower-core
cargo fmt --all
```

Avant `cargo test --workspace`, `cargo build` ou `npm run tauri:dev`, exécuter `npm run build` : Tauri a besoin de `frontend/dist`. Le mode natif ne compilera pas tant que Perl et la chaîne C Windows nécessaires à OpenSSL ne sont pas accessibles. L’état exact des dernières commandes est consigné dans [`docs/HANDOFF.md`](docs/HANDOFF.md) §2. Liste complète des commandes : [`docs/README.md`](docs/README.md).

## Sécurité des données

- La base native est destinée à être chiffrée avec SQLCipher ; si la compilation échoue sur la dépendance SQLCipher/OpenSSL, ne remplacez pas silencieusement cette dépendance par SQLite non chiffré.
- Le fichier de sel `.salt` n'est pas secret ; la clé dérivée et la phrase secrète ne sont pas écrites en clair dans les fichiers de configuration.
- Faites des sauvegardes de la base ET de son fichier `.salt`. Sans la phrase secrète et le sel d'origine, les données sont irrécupérables.
- La base de cette alpha n'est pas encore un système de comptabilité certifié ni une validation de conformité légale. Les écritures validées sont verrouillées fonctionnellement ; la contre-passation, l'audit complet, le FEC validé et l'archivage probant restent à construire.

## Dépôt

Dépôt cible : <https://github.com/tomzpoker/SuperPower-Alpha-0.1>

Les instructions détaillées et les décisions de conception se trouvent dans `docs/`. Commencer par `docs/README.md`, puis `docs/HANDOFF.md` et `docs/PROGRESS.md`.

## Première publication Git

*(À supprimer une fois le dépôt publié.)* Après les essais locaux et la revue des modifications, depuis la racine du dossier décompressé (le dépôt GitHub cible est vide) :

```powershell
git init
git add .
git commit -m "chore: bootstrap SuperPower Alpha 0.1"
git branch -M main
git remote add origin https://github.com/tomzpoker/SuperPower-Alpha-0.1.git
git push -u origin main
```

Ne versionnez jamais le coffre local, le fichier `.salt`, une phrase secrète ou des données réelles.
