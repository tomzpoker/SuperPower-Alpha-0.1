# AGENTS.md — consignes pour une IA ou un développeur qui reprend SuperPower

1. Lire dans l'ordre : `docs/HANDOFF.md` (état vérifié, pièges, suite), `docs/PROGRESS.md`, puis le sprint à reprendre dans `docs/SPRINTS/`. L'index complet est `docs/README.md`.
2. Ne rien déclarer terminé sans l'avoir exécuté. Consigner dans `docs/PROGRESS.md` ce qui a réellement été lancé (et ce qui ne l'a pas été).
3. Règles non négociables : montants en centimes entiers (jamais de flottant) ; validation métier côté Rust ; SQLCipher obligatoire (jamais de SQLite en clair) ; aucun secret, `*.db` ou `*.salt` dans Git ; aucune règle fiscale sans source officielle datée ; aucune donnée envoyée à l'extérieur sans action explicite.
4. Interface et messages d'erreur en français ; IPC en `camelCase` (champs) / `snake_case` (valeurs d'énumération) ; toute nouvelle commande est déclarée dans `commands.rs`, `lib.rs`, `frontend/src/lib/types.ts` et, si utile, `api.ts`.
5. Contrôles avant de rendre la main : `python scripts/check_docs.py`, `cargo fmt --all -- --check`, `cargo test -p superpower-core`, `npm test`, `npm run typecheck`, `npm run build` (voir `docs/HANDOFF.md` §3).
6. Mettre à jour la documentation dans le même changement que le code (protocole : `docs/HANDOFF.md` §7).
