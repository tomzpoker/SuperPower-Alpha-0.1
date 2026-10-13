# ADR-0002 — Moteur de tâches persistant

- **Statut :** prototype en cours ; décision de production non finalisée.
- **Date :** 2026-10-13.
- **Périmètre :** Sprint 02, première tranche technique.

## Contexte

Le cahier des charges impose des tâches persistantes, l'idempotence, la reprise après arrêt et une interface locale. La piste historique `flexiq-core` n'est pas présente dans le dépôt ; l'ajouter directement créerait une dépendance non vérifiée. La donnée opérationnelle ne doit pas être écrite dans une base SQLite annexe en clair : le coffre de l'application utilise SQLCipher.

## Décision provisoire

Prototyper une crate Rust interne `task-engine`, découplée de Tauri et de l'interface, qui reçoit une connexion SQLCipher appartenant à l'application. Ce prototype doit prouver les invariants de stockage avant de brancher un ordonnanceur et des handlers.

La première tranche comprend :
- définitions de tâches ponctuelles avec une clé d'idempotence ;
- file persistante des exécutions, avec une seconde clé d'idempotence par occurrence ;
- réservation atomique d'une exécution avec un bail ;
- historisation de chaque tentative, y compris l'interruption détectée par expiration du bail ;
- succès, échec, limite de tentatives et nouvelle disponibilité après échec ;
- tests sur un fichier temporaire chiffré, fermé puis rouvert avec la même clé.

Cette crate n'installe que ses tables ; elle ne modifie pas `PRAGMA user_version`. L'intégration au coffre sera orchestrée par `src-tauri/src/database.rs` au moyen d'une migration transactionnelle v3→v4. Aucun appel de production ni aucune migration du coffre principal n'est fait par le prototype.

## Points explicitement différés

- validation et calcul des expressions cron ;
- registre des handlers métier et exécution des jobs par le runtime ;
- politique de backoff configurable, arrêt propre et supervision ;
- commandes Tauri, UI liste/calendrier, historique consultable et adaptation du widget historique ;
- intégration des tâches documents, import bancaire et rappels locatifs.

Le statut de prototype ne signifie donc pas que les user stories du Sprint 02 sont livrées.

## Critères pour confirmer ou rejeter cette approche

1. `cargo fmt --all -- --check`, `cargo test -p task-engine`, `cargo test --workspace` et CI Tauri Windows verts.
2. Tests de persistance/idempotence/reprise réussis sur fichiers SQLCipher réouverts.
3. Migration applicative v3→v4 atomique ; schéma S06 conservé ; rejet d'une version future préservé.
4. Les handlers sont exécutés hors transaction de réservation ; une panne ne supprime ni l'exécution ni l'historique.
5. Un parseur cron maintenu, ses fuseaux et changements d'heure sont évalués avant de prendre en charge la planification récurrente.
6. Les commandes et l'interface affichent les erreurs en français ; toutes les mutations persistantes ont un audit adapté.
7. Aucune donnée réelle n'est envoyée à l'extérieur sans consentement explicite.

## État à la création de l'ADR

La première tranche se trouve sur `sprint/02-moteur-taches-prototype` et n'est pas encore fusionnée dans `main`. Elle est à valider par la CI et les tests locaux. Ne pas annoncer le moteur de tâches comme utilisable dans l'application tant que l'intégration, les handlers, cron et l'interface ne sont pas livrés.
