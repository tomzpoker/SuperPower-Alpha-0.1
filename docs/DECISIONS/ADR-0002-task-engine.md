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

La branche ajoute un registre de handlers indexé par module/type, un worker one-shot persistant et un mode sûr qui ignore les tâches sans handler. La CI #153 est entièrement verte sur `78ace82`, y compris le champ de message dédié, les contrôles Rust/frontend, le build Windows/Tauri et les tests workspace/migrations ; la CI #154 valide la mise à jour documentaire. L'essai natif sur coffre temporaire confirme que le rappel `general/reminder` affiche une notification et que l'exécution est historisée comme `Terminée`, avec une tentative `Réussie` (retour utilisateur du 13 octobre 2026). La vérification du verrouillage/déverrouillage et de la reprise sans doublon reste ouverte.

## Points explicitement différés

- politiques de backoff configurables et supervision avancée du worker ;
- handlers métier adaptés aux documents, à l'import bancaire et à la gestion locative ;
- édition complète des tâches et adaptation du widget historique ;
- intégration des tâches documents, import bancaire et rappels locatifs.

Les fréquences métier sont distinctes du cron avancé : `once`, `monthly`, `quarterly`, `yearly`, ou `custom_cron`. Les occurrences métier repartent de l'ancre d'origine pour éviter une dérive de fin de mois. La CI de cette tranche reste à confirmer.

Le statut de prototype ne signifie donc pas que les user stories du Sprint 02 sont livrées.

## Critères pour confirmer ou rejeter cette approche

1. `cargo fmt --all -- --check`, `cargo test -p task-engine`, `cargo test --workspace` et CI Tauri Windows verts.
2. Tests de persistance/idempotence/reprise réussis sur fichiers SQLCipher réouverts.
3. Migration applicative v3→v6 atomique ; schéma S06 conservé ; rejet d'une version future préservé. Les migrations et leurs tests de workspace sont passés dans la CI #153. Les essais manuels du coffre de test vérifient maintenant le worker de rappel ; le cycle verrouillage/déverrouillage reste ouvert.
4. Les handlers sont exécutés hors transaction de réservation ; une panne ne supprime ni l'exécution ni l'historique.
5. Le parseur cron numérique à cinq champs et le calcul UTC sont présents avec tests et aperçu natif ; avant d'offrir un fuseau local, évaluer les changements d'heure et expliciter les règles de saut ou de répétition d'horaires.
6. Les commandes et l'interface affichent les erreurs en français ; toutes les mutations persistantes ont un audit adapté.
7. Aucune donnée réelle n'est envoyée à l'extérieur sans consentement explicite.

## État à la création de l'ADR

La tranche se trouve toujours sur `sprint/02-moteur-taches-prototype` et n'est pas fusionnée dans `main`. La CI #153 est entièrement verte sur `78ace82`, y compris les migrations, le build Windows/Tauri et les tests. Les rappels se déclenchent uniquement avec l'application ouverte et le coffre déverrouillé. Un essai natif sur coffre temporaire confirme désormais la notification du rappel et son historique réussi. Le contrôle encore ouvert est le verrouillage/déverrouillage et la reprise sans doublon ; les autres handlers métier, le fuseau configurable et l'adaptation du widget historique restent à faire.
