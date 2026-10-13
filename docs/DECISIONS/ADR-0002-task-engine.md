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

La crate fournit l'installation des tables sans modifier elle-même `PRAGMA user_version`. Sur la branche S02, `src-tauri/src/database.rs` appelle cette installation dans une migration transactionnelle v3→v4 et versionne le schéma applicatif. Cette modification est à valider sur le dernier commit par les tests de migration et le build Windows ; elle n'est pas fusionnée dans `main`.

## Seconde tranche — registre et exécuteur piloté

La branche ajoute un registre de handlers indexé par module/type, qui refuse les doublons et peut être partagé entre threads. Le worker one-shot matérialise idempotemment les tâches ponctuelles arrivées à échéance, réserve au plus une exécution, appelle son handler hors transaction SQL et persiste succès ou échec avec historique et délai de nouvelle tentative. Les erreurs de handler et les paniques sont converties en échecs historisés. Cette couche reste appelée explicitement par son intégrateur : elle ne lance pas encore de boucle de fond Tauri et n'inclut pas de handler métier de production. La validation CI de cette tranche est requise avant intégration.

## Points explicitement différés

- validation et calcul des expressions cron ;
- démarrage/arrêt du worker dans Tauri, supervision et politique de backoff configurable ;
- handlers métier de production et accès contrôlé aux services applicatifs ;
- planification cron et calendrier ;
- édition complète des tâches et adaptation du widget historique ;
- intégration des tâches documents, import bancaire et rappels locatifs.

Le statut de prototype ne signifie donc pas que les user stories du Sprint 02 sont livrées.

## Critères pour confirmer ou rejeter cette approche

1. `cargo fmt --all -- --check`, `cargo test -p task-engine`, `cargo test --workspace` et CI Tauri Windows verts.
2. Tests de persistance/idempotence/reprise réussis sur fichiers SQLCipher réouverts.
3. Migration applicative v3→v4 atomique ; schéma S06 conservé ; rejet d'une version future préservé. La migration et un test de conservation sont maintenant codés sur la branche, mais ne sont pas encore confirmés par les contrôles exécutés après ce changement.
4. Les handlers sont exécutés hors transaction de réservation ; une panne ne supprime ni l'exécution ni l'historique.
5. Un parseur cron maintenu, ses fuseaux et changements d'heure sont évalués avant de prendre en charge la planification récurrente.
6. Les commandes et l'interface affichent les erreurs en français ; toutes les mutations persistantes ont un audit adapté.
7. Aucune donnée réelle n'est envoyée à l'extérieur sans consentement explicite.

## État à la création de l'ADR

La première tranche se trouve sur `sprint/02-moteur-taches-prototype` et n'est pas encore fusionnée dans `main`. Elle est à valider par la CI et les tests locaux. La page UI initiale permet désormais de créer/lister les tâches, d'en modifier le statut et de consulter l'historique disponible. Ne pas présenter l'exécution automatique comme livrée : worker, handlers métier, cron, calendrier et validation native restent à faire.
