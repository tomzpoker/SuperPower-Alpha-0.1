# HANDOFF — reprise du projet par une IA ou un développeur

> Lire ce fichier en premier après `AGENTS.md`. Il décrit l'état **vérifié** au 2026-10-12, les pièges connus et la suite ordonnée. Le tenir à jour à la fin de chaque session (voir §7).

## 1. Où en est le projet

SuperPower Alpha `0.1.0-alpha.1` est une application desktop locale (Tauri 2 + React/TS + Rust + SQLite/SQLCipher) de comptabilité multi-sociétés. **Première brique posée** : déverrouillage par phrase secrète (Argon2id → SQLCipher), registre des sociétés (forme juridique, IR/IS, TVA paramétrées séparément), 26 comptes de départ, saisie d'écritures équilibrées à deux lignes, audit des créations, cockpit, cinq thèmes. Rien d'autre n'est livré : les pages Documents, Locatif, Trésorerie, Fiscalité sont des écrans « à venir ».

Sprints en cours : **00 Bootstrap** et **01 Noyau**, tous deux `IN_PROGRESS`. Aucun n'est `DONE` : les commandes de compilation du workspace et de Tauri ont été tentées sur Windows, mais s'arrêtent pendant la construction d'OpenSSL vendored (`perl` absent), avant validation de l'application.

## 2. Ce qui a été vérifié (retour Windows du 2026-10-12)

Les résultats ci-dessous viennent d'une exécution réelle sur le poste Windows de reprise. Ils décrivent **la version avant le correctif de cette révision** ; après téléchargement du ZIP corrigé, relancer les contrôles pour confirmer les résultats sur le nouvel état du code.

| Vérification | Résultat observé | Portée réelle |
|---|---|---|
| `npm install` | Installation terminée ; 171 paquets annoncés | npm signale 6 vulnérabilités (3 modérées, 1 élevée, 2 critiques) et des scripts d'installation `esbuild` bloqués par la politique `allowScripts`. Le détail de `npm audit` n'a pas été fourni. Ne pas lancer `npm audit fix --force` à l'aveugle. |
| `npm test` | 1 fichier, 3 tests réussis | Tests `money.test.ts`, avec l'ancienne paire Vite 6.4.4 / Vitest 2.1.9. |
| `npm run typecheck` | ÉCHEC (2 diagnostics) | Import TypeScript `DashboardSummary` inutilisé, corrigé ; conflit de types Vite 6 / Vitest 2, corrigé en mettant Vitest à niveau vers la branche 4.1.11, compatible avec Vite 6. Vérifier à nouveau après `npm install`. |
| `npm run build` | ÉCHEC | S'arrête sur le même `typecheck` ; aucun bundle de production n'est confirmé par ce journal. |
| `cargo test -p superpower-core` | **8/8 tests réussis** | Cargo a compilé et testé réellement la crate `superpower-core`. |
| `cargo test --workspace` | ÉCHEC avant la compilation de l'application | `openssl-sys` n'arrive pas à compiler l'OpenSSL vendored : `perl` est introuvable. Ce message ne prouve pas un défaut du code Rust applicatif. |
| `npm run tauri:dev` | ÉCHEC pendant la compilation native | Même cause OpenSSL/Perl ; Vite démarre sur `http://127.0.0.1:1420/`, mais la fenêtre native Tauri ne démarre pas. |
| SQLCipher réel, verrouillage/déverrouillage, mauvaise phrase secrète | **Non validés** | À tester après réussite du build natif ; pas de données réelles avant cela. |

### Correctifs apportés à la révision

- Suppression de l'import TypeScript `DashboardSummary` inutilisé dans `frontend/src/lib/api.ts`.
- Mise à niveau de `vitest` vers `^4.1.11` dans `package.json`. La combinaison rapportée était Vitest 2.1.9 avec Vite 6.4.4 ; Vitest 2 ne prend pas officiellement en charge Vite 6 et génère cette famille d'incompatibilités de types. La compilation de cette correction n'a pas pu être relancée dans l'environnement de préparation, faute d'accès npm.
- Ajout des prérequis OpenSSL Windows au README et consignes de tests dans le bon ordre.

Le journal ne donne pas assez de détail pour identifier les paquets à l'origine des 6 alertes npm. Relancer `npm audit` et conserver son rapport avant toute mise à jour forcée.

## 3. Reprise rapide

```powershell
python scripts/check_docs.py
npm install
npm test
npm run typecheck
npm run build            # doit créer frontend/dist
cargo fmt --all -- --check
cargo test -p superpower-core
cargo test --workspace   # nécessite Perl/OpenSSL vendored + chaîne C sur Windows
npm run tauri:dev
```

Sur Windows, si `cargo test --workspace` ou `npm run tauri:dev` affiche `Command 'perl' not found`, installe Strawberry Perl depuis <https://strawberryperl.com/>, vérifie `where.exe perl` et `perl -v`, puis ferme/réouvre PowerShell. Utilise une console où les outils C++ de Visual Studio sont disponibles. Ne supprime pas `bundled-sqlcipher-vendored-openssl` pour contourner l'erreur : cela compromettrait l'objectif de ne jamais démarrer avec une base non chiffrée.

Parcours manuel à rejouer : initialisation avec phrase de 12+ caractères → fermeture → relance → mauvaise phrase rejetée → bonne phrase acceptée → créer une SCI et une SARL → écriture équilibrée acceptée → écriture déséquilibrée refusée côté Rust → vérifier que `superpower.db` ne commence pas par `SQLite format 3`.

## 4. Conventions à respecter

- Montants : **centimes entiers `i64`** (Rust) / `number` entier (TS). Jamais de flottant. Parse via `parseAmountToCents`.
- IPC : champs JSON `camelCase`, énumérations `snake_case`, erreurs `String` en français. Contrat complet : `ARCHITECTURE.md` §8.
- Toute règle comptable ou fiscale est validée **côté Rust** (le frontend n'est qu'une aide). Le noyau (`crates/core`) ne dépend ni de Tauri ni de SQLite.
- Ne jamais remplacer SQLCipher par SQLite non chiffré, ne jamais committer `*.db`, `*.salt`, phrases secrètes ou données réelles.
- Aucune règle fiscale sans source officielle, date de vérification, hypothèses et statut (voir `REFERENCES_FISCALES.md`). Pas d'inférence « SCI à l'IR ⇒ TVA exonérée ».
- Textes d'interface et messages d'erreur en français.
- Pas de nouvelle dépendance lourde (Tailwind, Zustand, SQLx, Tokio, etc.) sans décision consignée dans la section « Notes et décisions » du sprint concerné.
- Un sprint ne passe `DONE` qu'avec la Définition de Done de `ROADMAP_SPRINTS.md`.

## 5. Pièges connus

1. **`frontend/dist` doit exister à la compilation de `src-tauri`** (`tauri::generate_context!` lit `frontendDist`). Faire `npm run build` avant `cargo build/test --workspace`. *(Comportement Tauri habituel, à confirmer sur ce dépôt.)*
2. **OpenSSL « vendored »** (`bundled-sqlcipher-vendored-openssl`) : le journal Windows confirme que le build s'arrête parce que `perl` est absent. Installer Strawberry Perl, l'ajouter au `PATH`, rouvrir PowerShell et vérifier les prérequis C++/NASM si le build échoue ensuite. Référence : <https://github.com/openssl/openssl/blob/master/NOTES-WINDOWS.md>.
3. **Lockfiles** : aucun `Cargo.lock` ni `package-lock.json` n'était inclus dans le ZIP reçu. Après un `npm install` réussi sur la version corrigée, vérifier puis versionner `package-lock.json` et remplacer `npm install` par `npm ci` dans la CI. Versionner également `Cargo.lock` après une résolution Rust réussie.
4. **Versions** : les manifestes (`package.json`, `Cargo.toml`, `tauri.conf.json`) sont en `0.1.0` ; le tag Git visé est `v0.1.0-alpha.1`. Ne pas mettre de suffixe `-alpha.1` dans `tauri.conf.json` sans vérifier le build MSI Windows (suffixe pré-version numérique exigé).
5. **Immutabilité non appliquée en base** : un `UPDATE`/`DELETE` SQL direct sur `lignes_ecriture` ou `journal_audit` réussit (constaté). Seule l'absence d'endpoint protège aujourd'hui.
6. **Clippy** : la DoD exige `cargo clippy --workspace --all-targets -- -D warnings`, jamais exécuté. Alertes probables : `&PathBuf` en paramètre (`open_connection`), `map_or(true, …)`. *(Non vérifié.)*
7. **Aperçu navigateur ≠ natif** : `api.ts` n'impose ni longueur de phrase secrète ni existence des comptes ; ne pas s'y fier pour valider un comportement.
8. **Un échec de déverrouillage ne peut pas être audité dans la base** (elle est verrouillée) : prévoir un journal séparé si on veut le tracer.
9. Le fichier de sel s'appelle `superpower.salt` (même dossier que `superpower.db`) ; il doit être sauvegardé avec la base.
10. `App.tsx` concentre toute l'UI : le découper avant d'ajouter des pages.
11. La date par défaut d'une écriture utilise `toISOString()` (UTC) : un utilisateur en France peut voir la veille juste après minuit.

## 6. Suite recommandée (ordre conseillé, un lot = une PR)

**Lot A — Prouver la base (finir Sprint 00/01)**
1. Exécuter le §3 sur une machine avec réseau ; corriger ce qui casse ; committer les lockfiles ; mettre `PROGRESS.md` à jour.
2. Extraire de `commands.rs` des fonctions de service prenant une `Connection` (testables sans `tauri::State`), puis écrire des **tests d'intégration Rust** : mauvaise phrase rejetée ; fichier non lisible en clair ; création société ; écriture déséquilibrée/compte inconnu rejetée ; audit écrit dans la même transaction.
3. Ajouter triggers SQL d'interdiction `UPDATE`/`DELETE` sur `ecritures`, `lignes_ecriture`, `journal_audit` + test de non-régression.
4. Mécanisme de migrations versionnées basé sur `PRAGMA user_version` (v1 → v2…), sans SQLx.
5. CI : ajouter `cargo fmt --all -- --check` (déjà présent), `cargo clippy`, `npm ci`.

**Lot B — Compléter le noyau (Sprint 01)**
6. Tables `exercices` et `journaux` ; rattacher les écritures.
7. Écritures **multi-lignes** (formulaire N lignes, équilibre en direct) ; statut brouillon/validé ; **contre-passation** référencée et auditée.
8. PCG complet seedé depuis un fichier versionné, comptes propres à une société, recherche.
9. Balance, grand livre, **export FEC** avec jeu d'essai et test.
10. `update_societe` / suppression logique ; sélecteur de société déjà présent dans l'en-tête.
11. Brancher l'`EventBus` (ou le retirer) ; audit de toutes les mutations sensibles.
12. Découper `App.tsx` (pages, composants, hooks) et ajouter des tests de composants.

**Lot C — Sprint 02 (moteur de tâches)** : prototype et critères d'idempotence/reprise avant choix de dépendance (`flexiq-core` n'est pas dans le dépôt). Voir `SPRINTS/SPRINT_02_MOTEUR_TACHES.md`.

## 7. Protocole de fin de session

1. Lancer `python scripts/check_docs.py` et les commandes du §3 que l'environnement permet ; **consigner ce qui a vraiment été exécuté**, et ce qui ne l'a pas été.
2. Mettre à jour `PROGRESS.md` (état, journal), le fichier du sprint touché (cases cochées seulement si vérifiées, « Notes et décisions »), et ce fichier (§1, §2, §5, §6).
3. Corriger toute phrase de la documentation que le code contredit (ne pas laisser de promesse non tenue : cf. `ARCHITECTURE.md`).
4. Ne rien déclarer `DONE` sans la Définition de Done complète.
