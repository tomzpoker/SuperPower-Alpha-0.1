# Documentation — SuperPower Alpha 0.1

> Projet : [tomzpoker/SuperPower-Alpha-0.1](https://github.com/tomzpoker/SuperPower-Alpha-0.1)

SuperPower est une application desktop locale de pilotage opérationnel, comptable et fiscal pour plusieurs sociétés françaises. Cette alpha ne constitue pas encore un logiciel comptable/fiscal complet.

## Ordre de lecture
0. [\`../AGENTS.md\`](../AGENTS.md) puis [\`HANDOFF.md\`](HANDOFF.md) — reprise, état vérifié, pièges.
1. [\`PROGRESS.md\`](PROGRESS.md) — état du code et contrôles réels.
2. [\`REPORT_ALPHA_0.1.md\`](REPORT_ALPHA_0.1.md) — périmètre livré et limites.
3. [\`CAHIER_DES_CHARGES.md\`](CAHIER_DES_CHARGES.md) — besoins métier.
4. [\`ARCHITECTURE.md\`](ARCHITECTURE.md) — architecture réelle, sécurité, IPC, schéma.
5. [\`DESIGN_SYSTEM.md\`](DESIGN_SYSTEM.md) — design et thèmes.
6. [\`REFERENCES_FISCALES.md\`](REFERENCES_FISCALES.md) — sources officielles et limites.
7. [\`ROADMAP_SPRINTS.md\`](ROADMAP_SPRINTS.md), puis le sprint concerné.
8. [\`DECISIONS/ADR-0001-flux-previsionnels.md\`](DECISIONS/ADR-0001-flux-previsionnels.md) — séparation flux métier/réalisation/écriture comptable.
9. [\`DECISIONS/ADR-0002-task-engine.md\`](DECISIONS/ADR-0002-task-engine.md) — décision de prototype du moteur de tâches persistant.

## Règles de développement
- Ne pas déclarer un sprint terminé sans exécuter ses contrôles.
- Montants financiers en centimes entiers ou décimaux exacts, jamais flottants binaires.
- Toute règle comptable/fiscale validée côté Rust, documentée avec source officielle, date, hypothèses et statut.
- Forme juridique, régime IR/IS, TVA, exigibilité et périodicité sont distincts.
- Ne jamais remplacer SQLCipher par SQLite non chiffré.
- Aucun secret, phrase secrète, fichier \`*.db\` ou \`*.salt\` dans Git.
- Aucune donnée envoyée à l'extérieur sans action explicite.
- Un flux prévu, une réalisation et une écriture comptable sont trois objets séparés. Les catégories de trésorerie n'affectent pas automatiquement le PCG/TVA.
- Mettre à jour \`PROGRESS.md\`, le sprint, l'ADR concerné et \`HANDOFF.md\` dans chaque lot.

## Statuts
- \`TODO\` : non commencé.
- \`IN_PROGRESS\` : code en cours ou validation incomplète.
- \`BLOCKED\` : dépendance/problème bloquant documenté.
- \`DONE\` : critères de sortie et tests vérifiés.
- \`PLANNED\` : explicitement non livré.

## Contrôles
\`\`\`powershell
python scripts/check_docs.py
npm install
npm test
npm run typecheck
npm run build
cargo fmt --all -- --check
cargo test -p superpower-core
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run tauri:dev
\`\`\`
\`npm run dev\` est un aperçu volatile, non chiffré, sans persistance. Les données réelles appartiennent à l'application native Tauri.
