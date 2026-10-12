# Documentation — SuperPower Alpha 0.1

> Projet cible : [tomzpoker/SuperPower-Alpha-0.1](https://github.com/tomzpoker/SuperPower-Alpha-0.1)

SuperPower est une application desktop locale de pilotage opérationnel, comptable et fiscal pour plusieurs sociétés françaises. Cette alpha pose la base technique et un premier parcours métier ; elle ne constitue pas encore un logiciel comptable ou fiscal complet.

> Ce fichier est l'**index de la documentation** et la référence des commandes. Le [`README.md` racine](../README.md) est la page d'accueil du dépôt (présentation, démarrage, sécurité) ; ne pas dupliquer leur contenu.

## Ordre de lecture

0. [`../AGENTS.md`](../AGENTS.md) puis [`HANDOFF.md`](HANDOFF.md) — reprise par une IA/un développeur : état vérifié, pièges, suite ordonnée.
1. [`PROGRESS.md`](PROGRESS.md) — vérité projet : ce qui est présent, ce qui reste à vérifier.
2. [`REPORT_ALPHA_0.1.md`](REPORT_ALPHA_0.1.md) — périmètre effectivement livré, tests et limites.
3. [`CAHIER_DES_CHARGES.md`](CAHIER_DES_CHARGES.md) — besoins et limites de périmètre.
4. [`ARCHITECTURE.md`](ARCHITECTURE.md) — architecture réelle, sécurité et décisions de conception.
5. [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md) — langage visuel et cinq thèmes.
6. [`REFERENCES_FISCALES.md`](REFERENCES_FISCALES.md) — points fiscaux sensibles et sources officielles.
7. [`ROADMAP_SPRINTS.md`](ROADMAP_SPRINTS.md), puis le sprint à reprendre dans `SPRINTS/`.

## Règles de développement

- **Ne pas déclarer un sprint terminé sans exécuter les tests décrits.** Les sources peuvent avoir été générées sans disposer de Cargo dans l'environnement courant.
- Traiter les montants financiers comme des centimes entiers ou des nombres décimaux exacts, jamais comme des flottants binaires.
- Toute écriture validée est immuable ; une correction devra être une contre-écriture référencée et auditée.
- Distinguer forme juridique, régime d'imposition du résultat, régime de TVA, exigibilité, périodicité et traitement TVA par opération/bail lorsque nécessaire.
- Toute règle fiscale doit indiquer la date de vérification, la source officielle, les hypothèses et le statut (calcul préparatoire vs déclaration officielle).
- Ne jamais envoyer de données vers un service externe sans action explicite.
- Ne jamais remplacer SQLCipher par SQLite non chiffré pour contourner une erreur de compilation.
- Garder les clés, phrases secrètes, données bancaires et fichiers de base hors du dépôt Git.
- Mettre `PROGRESS.md`, le fichier du sprint et `HANDOFF.md` à jour à chaque livraison (protocole : `HANDOFF.md` §7).
- La documentation décrit le code **tel qu'il est** : toute promesse non tenue (immutabilité, audit, dépendances) doit être signalée comme telle.

## Statuts

- `TODO` : pas commencé.
- `IN_PROGRESS` : code en cours ou validation locale incomplète.
- `BLOCKED` : dépendance ou problème bloquant documenté.
- `DONE` : critères de sortie vérifiés, tests réussis et livrable revu.
- `PLANNED` : fonctionnalité future explicitement non livrée.

## Commandes de contrôle

```powershell
python scripts/check_docs.py
npm install
npm test
npm run typecheck
npm run build          # requis avant toute compilation de src-tauri
cargo fmt --all -- --check
cargo test -p superpower-core
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
npm run tauri:dev
```

`npm run dev` ouvre uniquement l'aperçu navigateur à mémoire volatile. Pour toute donnée réelle, utiliser l'application native Tauri et sauvegarder ensemble la base chiffrée, son fichier `.salt` et la phrase secrète conservée par son propriétaire.
