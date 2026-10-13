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

La crate fournit l'installation des tables sans modifier elle-même `PRAGMA user_version`. Sur la branche S02, `src-tauri/src/database.rs` orchestre les migrations transactionnelles v3→v6 et versionne le schéma applicatif. La CI #146 a validé les migrations et le build Windows sur le commit `881fcd5` ; la branche n'est pas fusionnée dans `main`.

## Seconde tranche — registre et exécuteur piloté

La branche ajoute un registre de handlers indexé par module/type, un worker one-shot persistant et un mode sûr qui ignore les tâches sans handler. La CI #38445019655 est verte sur `2c13cc8`, avec formatage/tests Rust, frontend et build Windows/Tauri. Le test du worker mensuel simule le 31 janvier, le 28 février et le 31 mars sans dérive ni doublon ; un test SQLCipher distinct vérifie le verrouillage/déverrouillage AppState, l’historique conservé et la reprise du worker. La CI #38438886275 valide le contrôleur start/stop/restart. L’essai natif du rappel confirme notification et historique réussi. Les commandes Tauri de verrouillage/déverrouillage dans l’interface native restent à confirmer manuellement. [CI #38445019655](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38445019655).

## Points explicitement différés

- politiques de backoff configurables et supervision avancée du worker ;
- handlers métier adaptés aux documents, à l'import bancaire et à la gestion locative ;
- édition complète des tâches et adaptation du widget historique ;
- intégration des tâches documents, import bancaire et rappels locatifs.

Les fréquences métier sont distinctes du cron avancé : `once`, `monthly`, `quarterly`, `yearly`, ou `custom_cron`. Les occurrences métier repartent de l'ancre d'origine pour éviter une dérive de fin de mois. Les tests de calcul et l’intégration worker mensuelle passent en CI #38442503451 ; les dates sont simulées, sans attente réelle d’échéance.

Le statut de prototype ne signifie donc pas que les user stories du Sprint 02 sont livrées.

## Critères pour confirmer ou rejeter cette approche

1. `cargo fmt --all -- --check`, `cargo test -p task-engine`, `cargo test --workspace` et CI Tauri Windows verts.
2. Tests de persistance/idempotence/reprise réussis sur fichiers SQLCipher réouverts.
3. Migration applicative v3→v6 atomique ; schéma S06 conservé ; rejet d'une version future préservé. Les migrations et le test AppState SQLCipher lock/unlock avec reprise idempotente passent en CI #38445019655. Le parcours natif des commandes Tauri reste à confirmer.
4. Les handlers sont exécutés hors transaction de réservation ; une panne ne supprime ni l'exécution ni l'historique.
5. Le parseur cron numérique à cinq champs et le calcul UTC sont présents avec tests et aperçu natif ; avant d'offrir un fuseau local, évaluer les changements d'heure et expliciter les règles de saut ou de répétition d'horaires.
6. Les commandes et l'interface affichent les erreurs en français ; toutes les mutations persistantes ont un audit adapté.
7. Aucune donnée réelle n'est envoyée à l'extérieur sans consentement explicite.

## État à la création de l'ADR

La tranche se trouve toujours sur `sprint/02-moteur-taches-prototype` et n'est pas fusionnée dans `main`. La CI #38445019655 est verte sur `2c13cc8`, incluant les tests de récurrence, les migrations, le build Windows/Tauri et le cycle AppState SQLCipher lock/unlock sans perte d’historique ni double exécution. Les rappels s’affichent lorsque l’application est ouverte et le coffre déverrouillé. Un essai natif sur coffre temporaire confirme la notification et son historique réussi. Le seul contrôle de cycle ouvert est l’appel réel aux commandes Tauri dans l’interface native. Les autres handlers métier, le fuseau configurable et l’adaptation du widget historique restent à faire.
