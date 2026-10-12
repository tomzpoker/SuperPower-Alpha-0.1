# Rapport de livraison — SuperPower Alpha 0.1

## Ce que contient cette livraison

- Workspace Cargo avec crate `superpower-core`.
- Domaine « société » configurable : forme juridique, IR/IS, paramètres TVA indépendants, SIRET facultatif, clôture.
- Avertissements non bloquants sur les régimes incomplets ou atypiques ; aucune déduction automatique irréversible.
- Validation d'écriture en partie double avec centimes entiers, contrôle des lignes, des montants et de l'égalité débit/crédit.
- Socle de comptes initial limité (pas le PCG exhaustif).
- Bus d'événements en mémoire avec test unitaire.
- Backend Tauri avec SQLCipher, dérivation Argon2id, schéma SQLite, sociétés, écritures, lignes comptables et journal d'audit de premier niveau.
- Interface React à cinq thèmes, accueil, saisie des sociétés et écritures, états de modules pas encore livrés.
- Aperçu navigateur à mémoire volatile ; aucune prétention de sécurité pour ce mode.
- CI GitHub Actions et tests frontend/noyau.
- Documentation nettoyée, références fiscales ajoutées et sprint 05 créé (le fichier référencé était absent du ZIP source).

## Validation et dernier retour Windows (2026-10-12)

Le développeur a exécuté les commandes sur Windows après avoir installé les dépendances. Les résultats avant le correctif sont : `npm install` réussi avec 6 vulnérabilités signalées ; `npm test` 3/3 ; `cargo test -p superpower-core` 8/8. Le typecheck échouait sur un import inutilisé et un conflit de types Vite 6/Vitest 2. Cette révision corrige l'import et met Vitest à niveau vers `^4.1.11`, mais cette mise à jour ne pouvait pas être réinstallée/recompilée dans l'environnement de préparation faute d'accès au registre npm.

Le build natif (`cargo test --workspace` et `npm run tauri:dev`) échouait avant compilation de l'application parce que la commande `perl` est introuvable pendant la compilation d'OpenSSL vendored. Installe Strawberry Perl et vérifie les outils C++ Windows avant de reprendre. Le correctif TypeScript, le build frontend et le build natif doivent encore être revalidés sur le poste Windows.

Ne pas utiliser de données réelles tant que le build natif, SQLCipher réel, le déverrouillage et le rejet d'une mauvaise phrase secrète n'ont pas été testés.

## Revue de cohérence du 2026-10-12

- Les résultats concrets et leur portée sont consignés dans `PROGRESS.md` et `HANDOFF.md` §2. Les 8 tests Rust du noyau et les 3 tests frontend passent dans la version précédant le correctif ; le schéma SQL a été vérifié séparément sur SQLite classique. Ces résultats ne valident pas SQLCipher.
- Écarts constatés : immutabilité et audit append-only non appliqués au niveau SQL ; bus d'événements non branché ; spécifications des sprints 00/01 divergeant du code (Tailwind/shadcn, `rust_decimal`, `tokio`, crates vides). Documentation alignée sur le code ; correctifs de code listés dans `HANDOFF.md` §6.
- Le code Rust a été reformaté (`cargo fmt`) sans changement de comportement.

## Tests à exécuter en priorité

```powershell
npm install
npm test
npm run typecheck
npm run build
cargo fmt --all
cargo test -p superpower-core
cargo test --workspace
npm run tauri:dev
```

Dans l'application native, tester : première initialisation avec une phrase secrète de 12 caractères ou plus, fermeture, redémarrage, déverrouillage correct, rejet d'une phrase incorrecte, création d'une SCI et d'une SARL, puis création d'écritures équilibrées et rejet d'une écriture déséquilibrée par le backend.

## Limites assumées

- Pas encore d'import bancaire, de baux/loyers, OCR, automatisation durable, projection de trésorerie, calendrier fiscal, déclarations 2072/2044/2065, CA3, export FEC validé ou synchronisation mobile.
- Le plan de comptes livré est un petit socle de démarrage ; il ne suffit pas à tenir une comptabilité de production.
- La gestion des comptes et écritures est minimale ; le grand livre, balance, clôture d'exercice, lettrage, contre-passation et contrôles d'audit sont à implémenter.
- Le journal d'audit de cette alpha n'enregistre pas encore toutes les actions sensibles.
- Le chiffrement SQLCipher a été configuré dans le code mais doit être confirmé sur Windows par un build et des essais négatifs/positifs.
- Aucun fichier de base ou secret utilisateur n'est inclus dans le ZIP.

## Proposition de version

`0.1.0-alpha.1` — première brique source à valider localement avant tout push ou usage avec des données réelles.
