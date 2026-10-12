# Sprint 04 — Documents

## Métadonnées

| Champ | Valeur |
|---|---|
| **Numéro** | 04 |
| **Nom** | Documents (Intelligence Documentaire) |
| **Durée estimée** | 6–8 jours |
| **Dépendances** | Sprints 01, 02 |
| **Branche Git** | `sprint/04-documents` |
| **Statut** | TODO |

## Contexte hérité

> À remplir après sprint 03. Doit contenir :
> - Noyau + compta + tâches + import bancaire opérationnels.
> - Event bus fonctionnel.
> - UI Inbox en place.
> - Schéma SQLCipher initial disponible via `rusqlite`; les migrations versionnées restent à formaliser. Ne pas supposer que SQLx est installé.

## Objectifs du sprint

1. Implémenter le pipeline d'extraction (PDF natif + OCR conditionnel).
2. Implémenter la normalisation de texte.
3. Implémenter le moteur de signaux et de profils.
4. Implémenter le classificateur avec score de confiance.
5. Implémenter l'étiquetage multi-entités.
6. Implémenter le renommage et le classement automatiques.
7. Implémenter la détection de doublons.
8. Implémenter la boucle de feedback (ajustement des poids).
9. Créer l'UI Documents (upload, visualisation, validation).
10. Créer les 10 profils de documents prioritaires.

## User Stories

### US-04-01 : Upload et analyse d'un document

**En tant qu'** utilisateur,
**je veux** déposer un document et qu'il soit analysé automatiquement,
**afin de** ne pas avoir à le classer manuellement.

**Critères d'acceptation :**
- [ ] Drag & drop ou sélection de fichier (PDF, JPG, PNG, HEIC).
- [ ] Le document est hashé (BLAKE3) et stocké.
- [ ] Le pipeline d'analyse se déclenche automatiquement (tâche `ANALYZE_DOCUMENT`).
- [ ] Le type est détecté avec un score de confiance.
- [ ] Les entités (société, lot, locataire, bien) sont extraites.
- [ ] Le document est renommé selon le template du type.
- [ ] Une notification apparaît avec le résultat.

**Tests :**
- [ ] Test unitaire : extraction de texte sur PDF de test.
- [ ] Test unitaire : normalisation (accents, OCR fixes).
- [ ] Test unitaire : score de classification sur corpus étiqueté.
- [ ] Test d'intégration : pipeline complet sur un PDF de quittance.
- [ ] Test e2e : upload + notification.

**Rollback :**
- [ ] Le document reste en statut "non classé" si le pipeline échoue.

### US-04-02 : Validation ou correction du classement

**En tant qu'** utilisateur,
**je veux** valider ou corriger le type proposé,
**afin d'** améliorer le système.

**Critères d'acceptation :**
- [ ] Si score ≥ 0.85 : auto-classé, notification discrète.
- [ ] Si 0.60 ≤ score < 0.85 : suggestion avec bouton "Confirmer".
- [ ] Si 0.40 ≤ score < 0.60 : choix multiple (top 3).
- [ ] Si score < 0.40 : file "À trier".
- [ ] Chaque correction est enregistrée pour le feedback.

**Tests :**
- [ ] Test unitaire : les bons seuils déclenchent les bonnes actions.
- [ ] Test e2e : correction d'un classement.

**Rollback :**
- [ ] Reclasser manuellement.

### US-04-03 : Détection de doublons

**En tant qu'** utilisateur,
**je veux** être alerté des doublons,
**afin de** ne pas avoir de documents en double.

**Critères d'acceptation :**
- [ ] Doublon exact : hash BLAKE3 identique → alerte immédiate.
- [ ] Doublon probable : même type + montant + date + émetteur, similarité texte > 0.90 → alerte.
- [ ] L'utilisateur peut Conserver / Remplacer / Ignorer.

**Tests :**
- [ ] Test unitaire : hash identique détecté.
- [ ] Test unitaire : similarité Jaro-Winkler > 0.90.
- [ ] Test e2e : upload d'un doublon.

**Rollback :**
- [ ] Supprimer le doublon.

### US-04-04 : Navigation multi-axes

**En tant qu'** utilisateur,
**je veux** retrouver un document par n'importe quel axe,
**afin de** ne pas dépendre d'une arborescence figée.

**Critères d'acceptation :**
- [ ] Filtres par : société, type, lot, locataire, bien, année, fournisseur.
- [ ] Recherche plein texte.
- [ ] Vue grille et vue liste.
- [ ] Prévisualisation PDF intégrée.

**Tests :**
- [ ] Test e2e : filtrer par locataire.
- [ ] Test e2e : recherche plein texte.

**Rollback :**
- [ ] Aucun (lecture seule).

## Tâches techniques

### T-04-01 : Extraction

- [ ] `crates/module-docs/src/extractor/pdf.rs` avec `pdf-extract`.
- [ ] `crates/module-docs/src/extractor/ocr.rs` avec `tesseract-rs` (conditionnel).
- [ ] `crates/module-docs/src/extractor/image.rs` avec `image` + `kamadak-exif`.
- [ ] `crates/module-docs/src/extractor/mail.rs` avec `mail-parser`.
- [ ] Hash BLAKE3 avec `blake3`.

### T-04-02 : Normalisation

- [ ] `crates/module-docs/src/normalizer.rs`.
- [ ] Unicode : `unicode-normalization`.
- [ ] OCR fixes : table de remplacement (`0`→`o` contextuel, `rn`→`m`, etc.).
- [ ] Lowercase, collapse espaces, strip non-alphanum.

### T-04-03 : Signaux

- [ ] `crates/module-docs/src/signals/mod.rs`.
- [ ] Types : `Keyword`, `Regex`, `Cooccurrence`, `Not`, `PdfAuthor`, `FilenameContains`, `PageCount`, `Amount`, `DateWithin`, `ValidSiret`, `ValidIban`, `LotNumber`, etc.
- [ ] Évaluateur : `fn evaluate(signal, context) -> bool`.

### T-04-04 : Profils TOML

- [ ] `crates/module-docs/src/profiles.rs`.
- [ ] Parseur TOML avec `toml` crate.
- [ ] Structure :
```toml
[types.quittance_loyer]
label = "Quittance de loyer"
category = "locatif"
renaming_template = "{annee}-{mois}_Quittance_{lot}_{locataire}"
classement = ["SCI/{societe}/Locatif/{annee}"]

[[types.quittance_loyer.indicators]]
kind = "keyword"
value = "quittance de loyer"
weight = 0.40
# ...
```

- [ ] Créer les 10 profils prioritaires :

        quittance_loyer

        bail

        taxe_fonciere

        facture_fournisseur

        bulletin_paie

        releve_bancaire

        avis_imposition

        appel_loyer

        etat_des_lieux

        diagnostic

### T-04-05 : Classificateur

- [ ] crates/module-docs/src/classifier.rs.
- [ ] Calcul du score : score = 1 - Π(1 - poids_i × présent_i).
- [ ] Seuils configurables.
- [ ] Retourne Vec<(TypeDoc, f32)> trié.

### T-04-06 : Étiquetage

- [ ] crates/module-docs/src/tagger.rs.
- [ ] Bases d'entités : locataires, lots, biens, sociétés.
- [ ] Recherche floue : strsim::jaro_winkler.
- [ ] Retourne HashMap<String, Vec<String>>.

### T-04-07 : Renommage et classement

- [ ] crates/module-docs/src/naming.rs.
- [ ] Template engine simple ({var} remplacé).
- [ ] Classement virtuel (tags en base, pas de déplacement fichier).

### T-04-08 : Doublons

- [ ] crates/module-docs/src/duplicates.rs.
- [ ] Détection exacte (hash).
- [ ] Détection floue (similarité).

### T-04-09 : Feedback

- [ ] crates/module-docs/src/feedback.rs.
- [ ] Enregistrement des corrections.
- [ ] Ajustement des poids (perceptron simple).
- [ ] Persistance des poids ajustés.

### T-04-10 : UI Documents

- [ ] Page /documents avec grille/liste.
- [ ] Composant DocumentUploader (drag & drop).
- [ ] Composant DocumentCard (aperçu, type, score, actions).
- [ ] Composant DocumentPreview (PDF viewer).
- [ ] Filtres multi-axes.
- [ ] Page /documents/a-trier.

### T-04-11 : Intégration tâche

- [ ] Enregistrer le handler ANALYZE_DOCUMENT dans le task engine.
- [ ] Émettre DocumentClasse après analyse.

## Ordre d'exécution

1. T-04-01 (extraction)
2. T-04-02 (normalisation)
3. T-04-03 (signaux)
4. T-04-04 (profils)
5. T-04-05 (classificateur)
6. T-04-06 (étiquetage)
7. T-04-07 (renommage)
8. T-04-08 (doublons)
9. T-04-09 (feedback)
10. T-04-10 (UI)
11. T-04-11 (tâche)
12. US-04-01 à US-04-04 (validation)

## Critères de sortie

- [ ] Pipeline complet opérationnel sur un corpus de 50 documents de test.
- [ ] Précision de classification ≥ 85% sur le corpus.
- [ ] Temps de traitement < 100 ms par document (PDF natif).
- [ ] OCR fonctionnel sur scans (temps < 3 s par page).
- [ ] UI Documents fluide et agréable.
- [ ] 10 profils TOML créés et calibrés.
- [ ] Tests passent, couverture ≥ 80%.
- [ ] PROGRESS.md mis à jour.
- [ ] Tag v0.4.0.

## État après sprint (handoff)

À remplir.

## Notes et décisions

- [2026-10-12] Pas de LLM local : règles + regex + score de confiance.

- [2026-10-12] Profils en TOML pour modularité maximale.

- [2026-10-12] Classement virtuel (tags) plutôt qu'arborescence.

- [2026-10-12] Feedback par perceptron simple (pas de réseau de neurones).
