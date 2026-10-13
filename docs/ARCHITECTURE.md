# Architecture technique — SuperPower Alpha

## 1. Principes
- Application desktop locale : Rust + Tauri 2, React + TypeScript.
- Persistance locale par défaut via SQLite/SQLCipher.
- Domaines métier séparés ; aucune règle fiscale déduite du seul nom de société.
- Frontière IPC explicite entre frontend et commandes Rust.
- Noyau testable sans UI, Tauri ou SQLite.
- Pas de LLM local ni d'appel réseau requis pour les fonctions métier courantes.

## 2. Vue de contexte
\`\`\`mermaid
flowchart TD
 U[Utilisateur] --> UI[React + TypeScript]
 UI -->|IPC Tauri| APP[Backend Rust]
 APP --> CORE[superpower-core]
 APP --> DB[(SQLite + SQLCipher)]
 APP --> AUDIT[Journal d'audit]
 FUT[Banque, locatif, fiscalité] -. réutilisent .-> CORE
\`\`\`

## 3. Structure livrée / en cours
\`\`\`text
crates/core/src/
  accounting.rs  # comptes et validation partie double
  society.rs     # société, régimes, avertissements
  flows.rs       # flux prévisionnels, réalisations, validation, états
  events.rs      # bus mémoire, pas encore branché
  ../task-engine/src/lib.rs # prototype isolé de file durable SQLCipher (branche S02, non intégré)
src-tauri/src/
  database.rs    # AppState, SQLCipher, Argon2id, schéma/migrations
  commands.rs    # IPC Tauri et persistance
  lib.rs         # enregistrement des commandes
frontend/src/
  App.tsx        # UI, encore monolithique
  lib/api.ts     # adaptateur Tauri et démo navigateur volatile
  lib/types.ts   # DTO TypeScript
  lib/money.ts   # conversion euros/centimes
  styles/app.css # design system et thèmes
docs/
  DECISIONS/ADR-0001-flux-previsionnels.md
  SPRINTS/SPRINT_06_TRESORERIE.md
\`\`\`

## 4. Modèle métier
### Société
La fiche distingue forme juridique, IR/IS, régime TVA, exigibilité, périodicité, SIRET et clôture. Ce paramétrage ne constitue pas une décision juridique. Le traitement TVA peut dépendre du bien, du bail, de l'opération et du preneur.

### Comptabilité actuelle
- Montants en centimes entiers.
- Une ligne ne peut pas porter débit et crédit simultanément.
- Une écriture doit comporter au moins deux lignes et un total débit égal au total crédit.
- Le backend vérifie société et comptes avant l'insertion.
- Le plan de comptes est un socle illustratif, pas un PCG exhaustif.
- L'immutabilité SQL des écritures et de l'audit reste à renforcer par triggers.

### Flux de trésorerie S06-A
Le flux prévu, chaque réalisation et une écriture comptable sont trois objets distincts. Le flux possède une société, un sens (encaissement/décaissement), une catégorie de pilotage, un libellé, une date, un montant prévu en centimes et des champs facultatifs tiers/référence. Les réalisations sont enregistrées séparément, ce qui permet les paiements partiels et la conservation du prévu.

Les états sont \`a_prevoir\`, \`en_retard\`, \`partiellement_realise\`, \`realise\`, \`depasse\), \`annule\`. Le cumul réalisé exclut les réalisations annulées. L'écart est \`prévu - réalisé\`. Une annulation conserve date/motif ; mutation et audit sont transactionnels.

Les catégories (loyer, charge courante, travaux, impôt/taxe, emprunt, apport d'associé, autre) sont uniquement des catégories de pilotage. Elles ne déterminent ni compte PCG ni TVA.

## 5. Persistance et chiffrement
1. La phrase secrète d'au moins 12 caractères sert à dériver une clé Argon2id.
2. Un sel aléatoire est conservé à côté de la base.
3. La clé est fournie à SQLCipher ; la phrase secrète n'est pas stockée.
4. La base reste verrouillée jusqu'à ressaisie.
5. Sauvegarder ensemble base et fichier \`.salt\).

Aucun fallback vers SQLite non chiffré n'est autorisé.

## 6. Migration de schéma
- v1 : sociétés, comptes, écritures, lignes, audit.
- v2 : ajoute \`flux_previsionnels\` et \`flux_realisations\`, index chronologiques et contraintes de montant.
- Migration v1→v2 transactionnelle à l'initialisation et au déverrouillage ; elle ne supprime aucune table v1.
- Une version future inconnue est refusée, jamais rétrogradée automatiquement.
- Test automatisé d'idempotence ajouté ; la validation sur copie d'une vraie base SQLCipher v1 reste nécessaire avant merge.

## 7. Contrat IPC actuel
| Commande | Arguments | Retour |
|---|---|---|
| \`database_status\` | — | \`needs_setup\` / \`locked\` / \`ready\` |
| \`initialize_database\` / \`unlock_database\` | \`passphrase\` | résultat de sécurité |
| \`list_societes\` / \`create_societe\` / \`update_societe\` / \`delete_societe\` | DTO société / id | société(s) |
| \`list_comptes\` | — | \`Account[]\` |
| \`create_ecriture\` / \`list_ecritures\` | DTO écriture / filtre société | écriture(s) |
| \`dashboard_summary\` | — | compteurs |
| \`list_flux\` | \`societeId?\` | \`FluxView[]\`, limite 500 |
| \`create_flux\` | \`input: FluxInput\` | \`FluxView\` |
| \`update_flux\` | \`id\`, \`input: FluxInput\` | vue, société inchangée |
| \`cancel_flux\` | \`id\` | vue annulée |
| \`register_flux_realisation\` | \`fluxId\`, \`input: RealisationFluxInput\` | vue recalculée |
| \`cancel_flux_realisation\` | \`id\`, \`motifAnnulation\` | vue recalculée |

Les champs JSON sont en \`camelCase\), enums en \`snake_case\). Chaque nouvelle commande doit être déclarée dans \`commands.rs\), \`lib.rs\), \`types.ts\` et dans \`api.ts\` pour la démo.

## 8. Séparation comptabilité/fiscalité
S06-A ne crée pas d'écriture comptable ni de TVA. Une opération réelle peut nécessiter tiers, ventilation multi-lignes, séparation principal/intérêts, traitement des dépôts/garanties et exigibilité TVA. Une couche future doit spécifier et tester ces règles avant de produire une écriture. Lire [ADR-0001](DECISIONS/ADR-0001-flux-previsionnels.md).

## 9. Dette sécurité/technique
- Triggers SQL pour empêcher la modification/suppression des écritures et de l'audit.
- Effacement mémoire de la clé dérivée.
- Tests natifs de mauvaise clé, chiffrement réel, sauvegarde/restauration.
- Exercices, journaux, clôture, contre-passation, FEC.
- Audit exhaustif des mutations.
- \`App.tsx\` à découper.
- Couverture >80 % non démontrée.

## 10. Évolutions
S06-A → import bancaire (03) → gestion locative (05) → projection/alertes (06) → règles comptables et fiscalité préparatoire (07). Le Sprint 02 démarre par un prototype interne task-engine utilisant la connexion SQLCipher de l'application, avec persistance, idempotence et reprise par bail. Il reste isolé tant que ses tests/CI ne sont pas validés ; flexiq-core est absent du dépôt et n'est pas ajouté sans évaluation.
