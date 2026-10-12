# AGENTS.md — consignes pour reprendre SuperPower

1. Lire \`docs/HANDOFF.md\`, \`docs/PROGRESS.md\`, puis le sprint courant. Index : \`docs/README.md\`.
2. Ne rien déclarer terminé sans exécuter les tests ; consigner les résultats réels.
3. Règles non négociables : centimes entiers ; validation métier côté Rust ; SQLCipher obligatoire ; aucun secret, \`*.db\` ou \`*.salt\) dans Git ; aucune règle fiscale sans source officielle datée ; aucun envoi de données sans consentement explicite.
4. Interface et erreurs en français ; IPC \`camelCase\) pour les champs, \`snake_case\) pour les enums. Toute commande doit être ajoutée dans \`commands.rs\`, \`src-tauri/src/lib.rs\`, \`frontend/src/lib/types.ts\` et, si utile, \`api.ts\`.
5. Contrôles : \`python scripts/check_docs.py\`, \`cargo fmt --all -- --check\`, \`cargo test -p superpower-core\`, \`npm test\`, \`npm run typecheck\`, \`npm run build\`. Voir \`docs/HANDOFF.md\`.
6. Mettre à jour la documentation dans le même changement que le code.
7. Les catégories de flux sont des catégories de pilotage, jamais une affectation PCG/TVA implicite. Lire \`docs/DECISIONS/ADR-0001-flux-previsionnels.md\` avant de relier flux et écritures.
8. Toute évolution du schéma incrémente \`PRAGMA user_version\`, migre en transaction, préserve les données et teste migration/idempotence/rejet des versions futures.
