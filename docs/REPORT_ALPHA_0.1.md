# Rapport de livraison — SuperPower Alpha 0.1

## Base livrée dans main
- Workspace Cargo et crate \`superpower-core\`.
- Domaine société : forme juridique, IR/IS, paramètres TVA séparés, SIRET et clôture.
- Validation de l'équilibre des écritures en centimes entiers.
- Petit socle de comptes (pas le PCG exhaustif).
- Backend Tauri, SQLCipher, Argon2id, schéma SQLite et journal d'audit initial.
- Interface React à cinq thèmes, mode navigateur volatile et CI GitHub Actions.

## État CI main
Le propriétaire a confirmé le 2026-10-13 que les trois checks de main sont passés : contrôles frontend, domaine Rust et build Windows Tauri. La CI utilise le cache Rust et une sortie détaillée. Cette confirmation porte sur main avant S06-A.

## Évolution S06-A — branche de travail du 2026-10-13
Le lot \`feat/guided-cash-flows\` ajoute :
- domaine Rust de flux prévisionnels et réalisations partielles ;
- migration SQLCipher \`user_version 1 → 2\) ;
- commandes IPC de liste, création, modification, annulation et réalisation ;
- interface Trésorerie en langage métier, sans sélection de comptes ;
- documentation : ADR-0001, Sprint 06, architecture, cahier des charges, roadmap, progression et handoff.

**Cette évolution n'est pas encore validée par CI.** Ne pas la fusionner avant réussite des contrôles de branche et vérification manuelle de la migration sur une copie de base native v1.

## Limites
- S06-A ne crée aucune écriture comptable ni TVA ; catégories de trésorerie uniquement.
- Aucun import bancaire, rapprochement, flux récurrent, projection 12/24/36 mois ou alerte.
- Le PCG complet, balance/grand livre, clôture, contre-passation, FEC et déclarations ne sont pas livrés.
- SQLCipher est configuré ; la sauvegarde/restauration et les tests de chiffrement doivent être validés avant données réelles.
- L'immutabilité par triggers SQL et l'audit complet restent à faire.

## Contrôles requis
\`\`\`powershell
python scripts/check_docs.py
cargo fmt --all -- --check
cargo test -p superpower-core
cargo test --workspace
npm test
npm run typecheck
npm run build
\`\`\`
Puis vérifier la CI Windows et le parcours natif sur base de test.
