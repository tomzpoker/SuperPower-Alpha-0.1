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

La crate fournit l'installation des tables sans modifier elle-même `PRAGMA user_version`. Sur la branche S02, `src-tauri/src/database.rs` orchestre les migrations transactionnelles v3→v5 et versionne le schéma applicatif. La CI #146 a validé les migrations et le build Windows sur le commit `881fcd5` ; la branche n'est pas fusionnée dans `main`.

## Seconde tranche — registre et exécuteur piloté

La branche ajoute un registre de handlers indexé par module/type, qui refuse les doublons et peut être partagé entre threads. Le worker one-shot persiste les exécutions et tentatives ; le mode sûr ne consomme que les tâches dont le handler est enregistré. Le lot courant ajoute un contrôleur natif démarré après déverrouillage et arrêté au verrouillage/à la fermeture, plus un handler `general/reminder` qui place le rappel dans une file locale. La notification n'est publiée qu'après succès SQLCipher, afin d'éviter de signaler une exécution non validée. La CI complète doit encore confirmer cette intégration.

## Points explicitement différés

- politiques de backoff configurables et supervision avancée du worker ;
- handlers métier adaptés aux documents, à l'import bancaire et à la gestion locative ;
- planification cron et calendrier ;
- édition complète des tâches et adaptation du widget historique ;
- intégration des tâches documents, import bancaire et rappels locatifs.

Le statut de prototype ne signifie donc pas que les user stories du Sprint 02 sont livrées.

## Critères pour confirmer ou rejeter cette approche

1. `cargo fmt --all -- --check`, `cargo test -p task-engine`, `cargo test --workspace` et CI Tauri Windows verts.
2. Tests de persistance/idempotence/reprise réussis sur fichiers SQLCipher réouverts.
3. Migration applicative v3→v4 atomique ; schéma S06 conservé ; rejet d'une version future préservé. La migration et un test de conservation sont maintenant codés sur la branche, mais ne sont pas encore confirmés par les contrôles exécutés après ce changement.
4. Les handlers sont exécutés hors transaction de réservation ; une panne ne supprime ni l'exécution ni l'historique.
5. Le parseur cron numérique à cinq champs et le calcul UTC sont présents avec tests et aperçu natif ; avant d'offrir un fuseau local, évaluer les changements d'heure et expliciter les règles de saut ou de répétition d'horaires.
6. Les commandes et l'interface affichent les erreurs en français ; toutes les mutations persistantes ont un audit adapté.
7. Aucune donnée réelle n'est envoyée à l'extérieur sans consentement explicite.

## État à la création de l'ADR

La tranche se trouve toujours sur `sprint/02-moteur-taches-prototype` et n'est pas fusionnée dans `main`. La CI #149 est entièrement verte sur `5d249e8` pour le moteur, le frontend, le build Windows/Tauri et les migrations. Le lot courant ajoute le worker natif supervisé et le premier rappel local ; son build et ses tests doivent être validés par la nouvelle CI. Les rappels se déclenchent uniquement avec l'application ouverte et le coffre déverrouillé. Les autres handlers métier, le fuseau configurable et l'adaptation du widget restent à faire.
