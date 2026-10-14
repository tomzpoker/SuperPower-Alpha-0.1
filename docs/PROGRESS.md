# PROGRESS — SuperPower Alpha

> Source de vérité : distinguer le code présent du code compilé/testé. Une fonctionnalité ne passe à `DONE` qu'après validation CI, parcours métier et revue.

## État actuel
| Champ | Valeur |
|---|---|
| Version | `0.1.0-alpha.1` |
| Main de référence | `f5affa7` — S02 fusionné le 14 octobre 2026 |
| Lot en cours | Sprint 03 — import bancaire CSV (première tranche parseur) |
| Branche | `sprint/03-import-bancaire` (PR #10 brouillon, cible `main`) |
| Statut | S02 est fusionné dans `main` via PR #8 (`f5affa7`) ; CI #194 verte sur les docs et CI code #192 verte. Les rappels ponctuel/récurrent et le parcours natif lock/unlock ont déjà été confirmés par l’utilisateur. S03 a démarré : `superpower-core::bank_import` fournit `parse_bank_csv_bytes` avec profils Auto/Crédit Agricole/LCL, encodage UTF-8/Windows-1252/UTF-16, détection prudente du séparateur, mapping d’en-têtes variable (Date opération prioritaire sur Date valeur), aperçu et erreurs par ligne. Les récapitulatifs de carte potentiellement doublés sont signalés mais jamais supprimés automatiquement. PR #10 / issue #9 ouvertes ; CI #211 verte sur le correctif CA débit négatif. Le run de 26 min a été un remplissage de cache à froid causé par le changement de clé ; le cache de ~809 Mo est maintenant sauvegardé et le prochain run doit vérifier son réemploi. |
| Prochaine action | Vérifier la nouvelle CI ; le cas CA à débit déjà négatif est couvert par un test fictif, puis obtenir un export LCL réel expurgé avant d’ajouter la persistance SQLCipher et la détection idempotente des doublons. La migration S06-B v2→v3 reste un contrôle séparé sur copie isolée. |
| Reprise | `AGENTS.md` → `docs/HANDOFF.md` |
| Dépôt | https://github.com/tomzpoker/SuperPower-Alpha-0.1 |

## Sprints
| # | Nom | Statut | Fichier | Notes |
|---|---|---|---|---|
| 00 | Bootstrap | IN_PROGRESS | `SPRINTS/SPRINT_00_BOOTSTRAP.md` | Coffre et CI disponibles ; revue sécurité complète restante |
| 01 | Noyau | IN_PROGRESS | `SPRINTS/SPRINT_01_NOYAU.md` | Sociétés et écritures simples ; PCG complet et clôture non livrés |
| 02 | Moteur de tâches | DONE — fusionné | `SPRINTS/SPRINT_02_MOTEUR_TACHES.md` | PR #8 fusionnée dans `main` (`f5affa7`), CI #194 verte et CI code #192 verte ; essais natifs confirmés par l’utilisateur |
| 03 | Import bancaire | IN_PROGRESS | `SPRINTS/SPRINT_03_IMPORT_BANCAIRE.md` | Issue #9 / PR #10 ; parseur CSV de domaine livré sur la branche, CI à confirmer, persistance et interface à venir |
| 04 | Documents | TODO | `SPRINTS/SPRINT_04_DOCUMENTS.md` | Pipeline non livré |
| 05 | Gestion locative | TODO | `SPRINTS/SPRINT_05_GESTION_LOCATIVE.md` | Biens/baux/TVA par opération non livrés |
| 06 | Trésorerie | S06-A et S06-B fusionnés ; validation native restante | `SPRINTS/SPRINT_06_TRESORERIE.md` et `SPRINTS/SPRINT_06B_RECURRENCE.md` | Récurrences intégrées dans `main` ; migration v2→v3 et essai métier natif à confirmer |
| 07 | Fiscalité | TODO | `SPRINTS/SPRINT_07_FISCALITE.md` | Déclarations non livrées |
| 08 | Sync mobile | TODO | `SPRINTS/SPRINT_08_SYNC_MOBILE.md` | Non livré |
| 09 | Stocks | TODO | `SPRINTS/SPRINT_09_STOCKS.md` | Extension facultative |

## Validation automatisée

Le run [#40](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38326327437) a validé S06-A. Pour S06-B, le run de PR [#96](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38332638917) est vert sur `cd764cb`. Le run post-fusion [#97](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38333882817) est également terminé avec succès sur `17966e1`.

| Contrôle | Portée et état |
|---|---|
| CI Sprint 02 — run #191 | **Verte** sur `7683c88` : jobs Windows/Tauri, Rust et frontend ; intégration worker + `tresorerie/rappel_flux` et idempotence : [run #191](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38461882586) |
| `cargo fmt --all -- --check` | S06-B passé sur le run #96 |
| Tests Rust du domaine | S06-B passé sur le run #96 et le run post-fusion #97 |
| Tests workspace / migrations automatisées | CI #153 verte sur `78ace82`, incluant le workspace et les migrations : [run #153](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38423220628) |
| Frontend | Typecheck, tests et bundle S06-B passés sur le run #96 et le run post-fusion #97 |
| Build Tauri Windows | CI #153 verte sur `78ace82` : [run #153](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38423220628) |
| CI fréquence métier v5→v6 | **Verte** sur `2c445cb` : tests Rust du moteur de tâches, build Windows/Tauri, tests workspace et migrations ; [run #162](https://github.com/tomzpoker/SuperPower-Alpha-0.1/actions/runs/38436681181) |
| Vérification Markdown et liens | Passée en CI #38440915992 sur `5c0cee9`, après les vérifications précédentes #153 et #154 |
| Parcours natif Trésorerie (2026-10-13) | **Réussi sur les scénarios observés** : loyer prévu modifié à 1 400 € sans écraser les 400 € réalisés ; assurance prévue à 240 € ; réalisation de 300 € enregistrée puis annulée depuis l'historique ; les lignes annulées restent visibles avec motif et sont exclues des totaux ; prévision assurance annulée et visible via le filtre |
| Réouverture d'une copie isolée du coffre | **Réussie selon le retour utilisateur** ; les données existantes sont restées visibles après réouverture, sans les fusionner dans le coffre habituel |
| Rappel récurrent Sprint 02 | **Testé avec succès selon le retour utilisateur** ; les calculs de récurrence et la reprise du worker sont aussi couverts par des dates simulées en CI, sans attendre plusieurs mois |
| Migration réelle depuis une base dont la version v1 est explicitement vérifiée | **À confirmer** : la copie s'ouvre et se rouvre, mais la valeur initiale de `PRAGMA user_version` n'a pas été relevée indépendamment |
| État `depasse` visible avant annulation | **Confirmé par l'utilisateur** : il a vu l'étiquette « Réalisé au-delà du prévu » lors de la réalisation de 300 € pour une prévision de 240 €, avant d'annuler cette réalisation |
| Refus technique d'une réalisation sur prévision annulée | Test unitaire du garde-fou partagé passé en CI sur la PR #4 ; test d'intégration invoquant la commande avec une prévision annulée en base native encore souhaitable |
| Vérification directe du schéma SQLCipher sur copie isolée | La version source de la copie précédente n'a pas été relevée indépendamment. Pour S06-B, vérifier `user_version = 3`, `flux_recurrences` et `recurrence_id` après migration v2→v3 ; ne jamais utiliser le coffre original comme cible |

Les essais manuels ont utilisé `SUPERPOWER_TEST_DATA_DIR` et des dossiers temporaires. La base source n'a pas été remplacée ni fusionnée avec les données des tests.


## Livré dans main
- Application native Tauri + React/TypeScript + noyau Rust.
- Coffre local SQLCipher/Argon2id.
- Gestion multi-entités des sociétés, socle de comptes et saisie technique d'écritures équilibrées.
- CI Windows, cache Rust et diagnostics.

## Sprint 06-A livré dans main
- Domaine Rust pour flux prévisionnels et réalisations partielles.
- Migration versionnée `user_version 1 → 2`, transactionnelle et sans reconstruction des tables v1.
- Commandes Tauri de liste/création/modification/annulation/réalisation avec audit transactionnel.
- Écran Trésorerie « Prévisionnel et réalisé », sans saisie de comptes, avec écart, états et historique.
- Mode aperçu navigateur cohérent mais volatil.
- Spécification : [Sprint 06](SPRINTS/SPRINT_06_TRESORERIE.md), [ADR-0001](DECISIONS/ADR-0001-flux-previsionnels.md).

## Sprint 06-B livré dans main
- Fréquences ponctuelle, mensuelle, trimestrielle et annuelle.
- Génération idempotente et horizon glissant de 36 mois, sans dérive de calendrier.
- Arrêt d'une série distinct de l'annulation d'une occurrence ; historique conservé.
- Migration SQLCipher v2→v3 intégrée ; validation native sur copie isolée requise.

## Limites / risques connus
- Le Sprint 02 est fusionné dans `main` via PR #8. La CI #194 est verte sur le commit documentaire de clôture ; la CI code #192 valide le moteur et le handler de rappel. Les rappels ponctuel/récurrent et le parcours natif lock/unlock sont confirmés comme effectués par l’utilisateur ; ne pas redemander ces essais. Le Sprint 03 est en cours dans la PR #10 : parseur CSV de domaine Crédit Agricole/LCL. Un extrait CA a fait ajouter la prise en charge des colonnes Débit déjà négatives (test de régression avec données fictives) ; il reste à valider la CI du nouveau head et à obtenir un export LCL réel expurgé. Persistance SQLCipher, déduplication idempotente et interface native restent à développer.
- S06-A ne crée pas encore d'écritures comptables ni de TVA.
- S06-B est livré dans `main` par la PR [#5](https://github.com/tomzpoker/SuperPower-Alpha-0.1/pull/5). La fenêtre glissante de 36 mois part d'aujourd'hui ; une première échéance hors fenêtre est conservée et les suivantes sont générées lorsque l'horizon l'atteint. Cela ne remplace pas une projection financière dédiée.
- Projection financière dédiée, alertes et rapprochement bancaire automatique non livrés.
- Baux, import bancaire, grand livre/balance, FEC validé, clôture et fiscalité automatisée non livrés.
- Plan de comptes limité au socle illustratif.
- Triggers d'immutabilité/append-only, sauvegarde/restauration chiffrée, audit sécurité complet et tests de mauvaise clé à renforcer.
- `App.tsx` reste monolithique ; la couverture supérieure à 80 % n'est pas démontrée.

## Journal des modifications
| Date | Lot | Action | Statut |
|---|---|---|---|
| 2026-10-12 | 00/01 | Première tranche : Tauri, UI, sociétés, écritures, coffre | Sur main |
| 2026-10-12 | CI | Formatage, cache Windows et diagnostics | Sur main |
| 2026-10-13 | 06-A | Prévu/réalisé, migration v2 et documentation métier | Fusionné dans main ; CI #40 verte ; vérification native de migration encore ouverte |
| 2026-10-13 | 06-A | Renforcement du test automatisé de migration v1 → v2 | CI verte sur main, run #40 |
| 2026-10-13 | 06-A | Parcours natif Trésorerie et copie de coffre isolée | Scénarios observés réussis ; version source de la copie migrée non relevée indépendamment |

| 2026-10-13 | 06-B | Fréquences ponctuelle/mensuelle/trimestrielle/annuelle, générateur idempotent et migration v2→v3 | Fusionné dans main (commit `17966e1`) ; CI PR #96 verte ; validation native migration à faire |

| 2026-10-13 | 02 | Prototype du registre de tâches/exécutions avec idempotence et reprise par bail | Branche `sprint/02-moteur-taches-prototype`, validation automatisée à réaliser avant intégration |

| 2026-10-13 | 02 | Migrations v3→v5, cron UTC, worker one-shot et suppression contrôlée | CI #149 entièrement verte sur `5d249e8` |
| 2026-10-13 | 02 | Worker natif stoppable et handler local `general/reminder` | CI #151 entièrement verte sur `8b8c0b5` |

| 2026-10-13 | 02 | Durcissement des erreurs worker/coffre et mise en file des rappels UI | CI #152 entièrement verte sur `8ac4417` |
| 2026-10-13 | 02 | Champ de message dédié pour les rappels et tests de transformation du payload | CI #153 entièrement verte sur `78ace82` |
| 2026-10-13 | 02 | Test automatisé du cycle coffre/worker | CI #38445019655 verte sur `2c13cc8` : lock/unlock AppState SQLCipher, historique conservé et absence de doublons après reprise |

| 2026-10-13 | 02 | Commandes Tauri et page Tâches | Branche non fusionnée ; worker et UI du rappel validés par CI #153 |
| 2026-10-13 | 02 | Tests de logique de commandes avec AppState SQLCipher temporaire | CI #38454692391 verte sur `5e39f35` : création/idempotence/audit, transitions, garde-fous d’historique et verrouillage ; parcours natif lock/unlock confirmé comme effectué par l’utilisateur |
| 2026-10-13 | 02 | Couverture automatisée du déverrouillage | CI #38457756360 verte sur `f3a3ef1` : ouverture correcte, mauvaise phrase secrète, rollback si le worker échoue ; parcours natif UI/IPC lock/unlock confirmé comme effectué par l’utilisateur |
| 2026-10-13 | 02 | Intégration worker + rappel de trésorerie | CI #191 verte sur `7683c88` : exécution du handler enregistré, lecture du solde SQLCipher, historique et seconde passe sans doublon |
| 2026-10-14 | 02 | Test natif d’un rappel ponctuel `general/reminder` (échéance 02:22:00, démarrage 02:22:26, tentative 1/3 réussie, durée 0 s) | Succès confirmé par l’utilisateur ; écart observé de 26 s consigné sans conclure à un défaut de ponctualité | 
| 2026-10-14 | 02 | Confirmation utilisateur : parcours natif de verrouillage/déverrouillage déjà effectué | Contrôle clos dans la documentation ; ne pas redemander le test | 
| 2026-10-14 | 02 | Clôture du moteur de tâches | PR #8 fusionnée dans `main` (`f5affa7`) ; CI #194 verte, sans redemander les tests natifs déjà réalisés | 
| 2026-10-14 | 03 | Première tranche d’import bancaire | Parseur CSV indépendant, mapping explicite, aperçu, validation numérique et erreurs par ligne ; PR #10 en cours, CI à confirmer |
