use chrono::{Local, NaiveDate};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use superpower_core::{
    accounting::{validate_entry_lines, EntryLine},
    flows::{
        compute_flux_state, validate_flux_input, validate_realisation_input, CategorieFlux,
        FluxInput, FluxRealisationView, FluxView, RealisationFluxInput, SensFlux, StatutFlux,
    },
    society::{ExigibiliteTva, FormeJuridique, PeriodiciteTva, RegimeFiscal, RegimeTva, Societe},
};
use tauri::State;
use uuid::Uuid;

use crate::database::{AppState, DatabaseStatus};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSocieteInput {
    nom: String,
    forme_juridique: FormeJuridique,
    regime_fiscal: RegimeFiscal,
    regime_tva: RegimeTva,
    exigibilite_tva: ExigibiliteTva,
    periodicite_tva: PeriodiciteTva,
    siret: Option<String>,
    fin_exercice: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocieteView {
    #[serde(flatten)]
    pub societe: Societe,
    pub warnings: Vec<String>,
}

impl From<Societe> for SocieteView {
    fn from(societe: Societe) -> Self {
        let warnings = societe.warnings().items;
        Self { societe, warnings }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntryInput {
    societe_id: String,
    date_ecriture: String,
    libelle: String,
    reference: Option<String>,
    lines: Vec<EntryLine>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryView {
    pub id: String,
    pub societe_id: String,
    pub date_ecriture: String,
    pub libelle: String,
    pub reference: Option<String>,
    pub statut: String,
    pub total_cents: i64,
    pub lines: Vec<EntryLine>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub societes: i64,
    pub ecritures_validees: i64,
    pub lignes_comptables: i64,
    pub volume_debit_cents: i64,
}

struct FluxRecord {
    id: String,
    societe_id: String,
    libelle: String,
    sens: String,
    categorie: String,
    date_prevue: String,
    montant_prevu_cents: i64,
    tiers: Option<String>,
    reference: Option<String>,
    statut: String,
    created_at: String,
    updated_at: String,
}

#[tauri::command]
pub fn database_status(state: State<'_, AppState>) -> Result<DatabaseStatus, String> {
    Ok(state.status())
}

#[tauri::command]
pub fn initialize_database(passphrase: String, state: State<'_, AppState>) -> Result<(), String> {
    state.initialize(&passphrase)
}

#[tauri::command]
pub fn unlock_database(passphrase: String, state: State<'_, AppState>) -> Result<(), String> {
    state.unlock(&passphrase)
}

#[tauri::command]
pub fn lock_database(state: State<'_, AppState>) -> Result<(), String> {
    state.lock();
    Ok(())
}

#[tauri::command]
pub fn list_societes(state: State<'_, AppState>) -> Result<Vec<SocieteView>, String> {
    let connection = state.connection()?;
    let mut statement = connection.prepare(
        "SELECT id, nom, forme_juridique, regime_fiscal, regime_tva, exigibilite_tva, periodicite_tva, siret, fin_exercice, created_at FROM societes ORDER BY nom COLLATE NOCASE"
    ).map_err(db_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
            ))
        })
        .map_err(db_error)?;
    let mut companies = Vec::new();
    for row in rows {
        let raw = row.map_err(db_error)?;
        let company = Societe {
            id: raw.0,
            nom: raw.1,
            forme_juridique: from_enum(&raw.2)?,
            regime_fiscal: from_enum(&raw.3)?,
            regime_tva: from_enum(&raw.4)?,
            exigibilite_tva: from_enum(&raw.5)?,
            periodicite_tva: from_enum(&raw.6)?,
            siret: raw.7,
            fin_exercice: raw.8,
            created_at: raw.9,
        };
        companies.push(company.into());
    }
    Ok(companies)
}

#[tauri::command]
pub fn create_societe(
    input: CreateSocieteInput,
    state: State<'_, AppState>,
) -> Result<SocieteView, String> {
    let company = Societe::new(
        input.nom,
        input.forme_juridique,
        input.regime_fiscal,
        input.regime_tva,
        input.exigibilite_tva,
        input.periodicite_tva,
        input.siret,
        input.fin_exercice,
    )
    .map_err(|error| error.to_string())?;
    let mut connection = state.connection()?;
    let transaction = connection.transaction().map_err(db_error)?;
    transaction.execute(
        "INSERT INTO societes(id, nom, forme_juridique, regime_fiscal, regime_tva, exigibilite_tva, periodicite_tva, siret, fin_exercice, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![company.id, company.nom, enum_name(company.forme_juridique)?, enum_name(company.regime_fiscal)?,
            enum_name(company.regime_tva)?, enum_name(company.exigibilite_tva)?, enum_name(company.periodicite_tva)?,
            company.siret, company.fin_exercice, company.created_at],
    ).map_err(db_error)?;
    audit_tx(
        &transaction,
        "create",
        "societe",
        &company.id,
        serde_json::json!({"nom": company.nom}),
    )
    .map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    Ok(company.into())
}

#[tauri::command]
pub fn update_societe(
    id: String,
    input: CreateSocieteInput,
    state: State<'_, AppState>,
) -> Result<SocieteView, String> {
    let mut company = Societe::new(
        input.nom,
        input.forme_juridique,
        input.regime_fiscal,
        input.regime_tva,
        input.exigibilite_tva,
        input.periodicite_tva,
        input.siret,
        input.fin_exercice,
    )
    .map_err(|error| error.to_string())?;

    let mut connection = state.connection()?;
    let transaction = connection.transaction().map_err(db_error)?;
    let created_at: Option<String> = transaction
        .query_row(
            "SELECT created_at FROM societes WHERE id = ?1",
            [&id],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)?;
    let Some(created_at) = created_at else {
        return Err("La société à modifier n'existe plus.".to_string());
    };

    company.id = id.clone();
    company.created_at = created_at;
    transaction.execute(
        "UPDATE societes SET nom = ?1, forme_juridique = ?2, regime_fiscal = ?3, regime_tva = ?4, exigibilite_tva = ?5, periodicite_tva = ?6, siret = ?7, fin_exercice = ?8 WHERE id = ?9",
        params![
            company.nom,
            enum_name(company.forme_juridique)?,
            enum_name(company.regime_fiscal)?,
            enum_name(company.regime_tva)?,
            enum_name(company.exigibilite_tva)?,
            enum_name(company.periodicite_tva)?,
            company.siret,
            company.fin_exercice,
            company.id,
        ],
    )
    .map_err(db_error)?;
    audit_tx(
        &transaction,
        "update",
        "societe",
        &id,
        serde_json::json!({"nom": company.nom}),
    )
    .map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    Ok(company.into())
}

#[tauri::command]
pub fn delete_societe(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut connection = state.connection()?;
    let transaction = connection.transaction().map_err(db_error)?;
    let company_name: Option<String> = transaction
        .query_row("SELECT nom FROM societes WHERE id = ?1", [&id], |row| {
            row.get(0)
        })
        .optional()
        .map_err(db_error)?;
    let Some(company_name) = company_name else {
        return Err("La société à supprimer n'existe plus.".to_string());
    };
    let entry_count: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM ecritures WHERE societe_id = ?1",
            [&id],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    if entry_count > 0 {
        return Err(format!(
            "Suppression refusée : {company_name} possède {entry_count} écriture(s) comptable(s). L'historique comptable doit être conservé."
        ));
    }
    let flux_count: i64 = transaction
        .query_row("SELECT COUNT(*) FROM flux_previsionnels WHERE societe_id = ?1", [&id], |row| row.get(0))
        .map_err(db_error)?;
    if flux_count > 0 {
        return Err(format!(
            "Suppression refusée : {company_name} possède {flux_count} flux prévisionnel(s), même annulé(s). Conservez l'historique."
        ));
    }
    audit_tx(
        &transaction,
        "delete",
        "societe",
        &id,
        serde_json::json!({"nom": company_name}),
    )
    .map_err(db_error)?;
    transaction
        .execute("DELETE FROM societes WHERE id = ?1", [&id])
        .map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    Ok(())
}

#[tauri::command]
pub fn list_flux(
    societe_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<FluxView>, String> {
    let connection = state.connection()?;
    let records = {
        let mut statement = connection.prepare(
            "SELECT id, societe_id, libelle, sens, categorie, date_prevue, montant_prevu_cents, tiers, reference, statut, created_at, updated_at
             FROM flux_previsionnels WHERE (?1 IS NULL OR societe_id = ?1)
             ORDER BY date_prevue ASC, created_at DESC LIMIT 500"
        ).map_err(db_error)?;
        let rows = statement.query_map([societe_id], |row| {
            Ok(FluxRecord {
                id: row.get(0)?, societe_id: row.get(1)?, libelle: row.get(2)?,
                sens: row.get(3)?, categorie: row.get(4)?, date_prevue: row.get(5)?,
                montant_prevu_cents: row.get(6)?, tiers: row.get(7)?, reference: row.get(8)?,
                statut: row.get(9)?, created_at: row.get(10)?, updated_at: row.get(11)?,
            })
        }).map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)?
    };
    records.into_iter().map(|record| build_flux_view(&connection, record)).collect()
}

#[tauri::command]
pub fn create_flux(input: FluxInput, state: State<'_, AppState>) -> Result<FluxView, String> {
    validate_flux_input(&input).map_err(|error| error.to_string())?;
    let libelle = input.libelle.trim().to_string();
    let tiers = clean_optional(input.tiers);
    let reference = clean_optional(input.reference);
    let date_prevue = NaiveDate::parse_from_str(&input.date_prevue, "%Y-%m-%d")
        .map_err(|_| "La date prévue doit être au format AAAA-MM-JJ.".to_string())?;
    let sens = enum_name(input.sens)?;
    let categorie = enum_name(input.categorie)?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let mut connection = state.connection()?;
    let transaction = connection.transaction().map_err(db_error)?;
    let company_exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM societes WHERE id = ?1)",
        [&input.societe_id], |row| row.get(0)
    ).map_err(db_error)?;
    if !company_exists {
        return Err("La société sélectionnée n'existe pas pour ce flux.".to_string());
    }
    transaction.execute(
        "INSERT INTO flux_previsionnels(id, societe_id, libelle, sens, categorie, date_prevue, montant_prevu_cents, tiers, reference, statut, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'actif', ?10, ?10)",
        params![id, input.societe_id, libelle, sens, categorie, date_prevue.to_string(), input.montant_prevu_cents, tiers, reference, now],
    ).map_err(db_error)?;
    audit_tx(&transaction, "create", "flux_previsionnel", &id, serde_json::json!({
        "societeId": input.societe_id, "libelle": input.libelle.trim(), "sens": sens,
        "categorie": categorie, "datePrevue": date_prevue.to_string(),
        "montantPrevuCents": input.montant_prevu_cents,
    })).map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    build_flux_view(&connection, read_flux_record(&connection, &id)?)
}

#[tauri::command]
pub fn update_flux(id: String, input: FluxInput, state: State<'_, AppState>) -> Result<FluxView, String> {
    validate_flux_input(&input).map_err(|error| error.to_string())?;
    let libelle = input.libelle.trim().to_string();
    let tiers = clean_optional(input.tiers);
    let reference = clean_optional(input.reference);
    let date_prevue = NaiveDate::parse_from_str(&input.date_prevue, "%Y-%m-%d")
        .map_err(|_| "La date prévue doit être au format AAAA-MM-JJ.".to_string())?;
    let sens = enum_name(input.sens)?;
    let categorie = enum_name(input.categorie)?;
    let now = chrono::Utc::now().to_rfc3339();
    let mut connection = state.connection()?;
    let transaction = connection.transaction().map_err(db_error)?;
    let existing: Option<(String, String)> = transaction.query_row(
        "SELECT societe_id, statut FROM flux_previsionnels WHERE id = ?1", [&id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).optional().map_err(db_error)?;
    let Some((societe_id, statut)) = existing else {
        return Err("Ce flux n'existe plus.".to_string());
    };
    if statut != "actif" {
        return Err("Un flux annulé ne peut pas être modifié. Créez une nouvelle prévision si nécessaire.".to_string());
    }
    if societe_id != input.societe_id {
        return Err("La société d'un flux existant ne peut pas être changée. Annulez-le puis créez le flux dans la bonne société.".to_string());
    }
    transaction.execute(
        "UPDATE flux_previsionnels SET libelle = ?1, sens = ?2, categorie = ?3, date_prevue = ?4, montant_prevu_cents = ?5, tiers = ?6, reference = ?7, updated_at = ?8 WHERE id = ?9 AND statut = 'actif'",
        params![libelle, sens, categorie, date_prevue.to_string(), input.montant_prevu_cents, tiers, reference, now, id],
    ).map_err(db_error)?;
    audit_tx(&transaction, "update", "flux_previsionnel", &id, serde_json::json!({
        "libelle": input.libelle.trim(), "datePrevue": date_prevue.to_string(),
        "montantPrevuCents": input.montant_prevu_cents, "sens": sens, "categorie": categorie,
    })).map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    build_flux_view(&connection, read_flux_record(&connection, &id)?)
}

#[tauri::command]
pub fn cancel_flux(id: String, state: State<'_, AppState>) -> Result<FluxView, String> {
    let mut connection = state.connection()?;
    let transaction = connection.transaction().map_err(db_error)?;
    let existing: Option<(String, String)> = transaction.query_row(
        "SELECT libelle, statut FROM flux_previsionnels WHERE id = ?1", [&id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).optional().map_err(db_error)?;
    let Some((libelle, statut)) = existing else {
        return Err("Ce flux n'existe plus.".to_string());
    };
    if statut == "annule" {
        return Err("Cette prévision est déjà annulée.".to_string());
    }
    let now = chrono::Utc::now().to_rfc3339();
    transaction.execute(
        "UPDATE flux_previsionnels SET statut = 'annule', updated_at = ?1 WHERE id = ?2",
        params![now, id]
    ).map_err(db_error)?;
    audit_tx(&transaction, "cancel", "flux_previsionnel", &id, serde_json::json!({"libelle": libelle}))
        .map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    build_flux_view(&connection, read_flux_record(&connection, &id)?)
}

#[tauri::command]
pub fn register_flux_realisation(
    flux_id: String,
    input: RealisationFluxInput,
    state: State<'_, AppState>,
) -> Result<FluxView, String> {
    validate_realisation_input(&input).map_err(|error| error.to_string())?;
    let date = NaiveDate::parse_from_str(&input.date_realisation, "%Y-%m-%d")
        .map_err(|_| "La date de réalisation doit être au format AAAA-MM-JJ.".to_string())?;
    let reference = clean_optional(input.reference);
    let note = clean_optional(input.note);
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let mut connection = state.connection()?;
    let transaction = connection.transaction().map_err(db_error)?;
    let status: Option<String> = transaction.query_row(
        "SELECT statut FROM flux_previsionnels WHERE id = ?1", [&flux_id], |row| row.get(0)
    ).optional().map_err(db_error)?;
    let Some(status) = status else { return Err("Ce flux n'existe plus.".to_string()); };
    if status != "actif" {
        return Err("Impossible d'enregistrer un encaissement ou paiement sur une prévision annulée.".to_string());
    }
    transaction.execute(
        "INSERT INTO flux_realisations(id, flux_id, date_realisation, montant_cents, reference, note, created_at, annulee_at, motif_annulation)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, NULL)",
        params![id, flux_id, date.to_string(), input.montant_cents, reference, note, now],
    ).map_err(db_error)?;
    audit_tx(&transaction, "create", "flux_realisation", &id, serde_json::json!({
        "fluxId": flux_id, "dateRealisation": date.to_string(), "montantCents": input.montant_cents,
    })).map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    build_flux_view(&connection, read_flux_record(&connection, &flux_id)?)
}

#[tauri::command]
pub fn cancel_flux_realisation(
    id: String,
    motif_annulation: String,
    state: State<'_, AppState>,
) -> Result<FluxView, String> {
    let motif = motif_annulation.trim();
    if motif.is_empty() || motif.chars().count() > 300 {
        return Err("Indiquez un motif d'annulation de 1 à 300 caractères.".to_string());
    }
    let mut connection = state.connection()?;
    let transaction = connection.transaction().map_err(db_error)?;
    let existing: Option<(String, Option<String>, i64)> = transaction.query_row(
        "SELECT flux_id, annulee_at, montant_cents FROM flux_realisations WHERE id = ?1", [&id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).optional().map_err(db_error)?;
    let Some((flux_id, annulee_at, montant_cents)) = existing else {
        return Err("Cette réalisation n'existe plus.".to_string());
    };
    if annulee_at.is_some() {
        return Err("Cette réalisation est déjà annulée.".to_string());
    }
    let now = chrono::Utc::now().to_rfc3339();
    transaction.execute(
        "UPDATE flux_realisations SET annulee_at = ?1, motif_annulation = ?2 WHERE id = ?3 AND annulee_at IS NULL",
        params![now, motif, id],
    ).map_err(db_error)?;
    audit_tx(&transaction, "cancel", "flux_realisation", &id, serde_json::json!({
        "fluxId": flux_id, "montantCents": montant_cents, "motif": motif,
    })).map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    build_flux_view(&connection, read_flux_record(&connection, &flux_id)?)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.map(|text| text.trim().to_string()).filter(|text| !text.is_empty())
}

fn read_flux_record(connection: &rusqlite::Connection, id: &str) -> Result<FluxRecord, String> {
    connection.query_row(
        "SELECT id, societe_id, libelle, sens, categorie, date_prevue, montant_prevu_cents, tiers, reference, statut, created_at, updated_at
         FROM flux_previsionnels WHERE id = ?1",
        [id],
        |row| Ok(FluxRecord {
            id: row.get(0)?, societe_id: row.get(1)?, libelle: row.get(2)?, sens: row.get(3)?,
            categorie: row.get(4)?, date_prevue: row.get(5)?, montant_prevu_cents: row.get(6)?,
            tiers: row.get(7)?, reference: row.get(8)?, statut: row.get(9)?, created_at: row.get(10)?, updated_at: row.get(11)?,
        })
    ).optional().map_err(db_error)?
     .ok_or_else(|| "Ce flux n'existe plus.".to_string())
}

fn build_flux_view(connection: &rusqlite::Connection, record: FluxRecord) -> Result<FluxView, String> {
    let realisations = {
        let mut statement = connection.prepare(
            "SELECT id, flux_id, date_realisation, montant_cents, reference, note, created_at, annulee_at, motif_annulation
             FROM flux_realisations WHERE flux_id = ?1 ORDER BY date_realisation DESC, created_at DESC"
        ).map_err(db_error)?;
        let rows = statement.query_map([&record.id], |row| Ok(FluxRealisationView {
            id: row.get(0)?, flux_id: row.get(1)?, date_realisation: row.get(2)?, montant_cents: row.get(3)?,
            reference: row.get(4)?, note: row.get(5)?, created_at: row.get(6)?, annulee_at: row.get(7)?, motif_annulation: row.get(8)?,
        })).map_err(db_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(db_error)?
    };
    let mut montant_realise_cents = 0_i64;
    for realisation in &realisations {
        if realisation.annulee_at.is_none() {
            montant_realise_cents = montant_realise_cents.checked_add(realisation.montant_cents)
                .ok_or_else(|| "Le cumul des réalisations dépasse la plage autorisée.".to_string())?;
        }
    }
    let statut: StatutFlux = from_enum(&record.statut)?;
    let sens: SensFlux = from_enum(&record.sens)?;
    let categorie: CategorieFlux = from_enum(&record.categorie)?;
    let date = NaiveDate::parse_from_str(&record.date_prevue, "%Y-%m-%d")
        .map_err(|_| "Une date prévue enregistrée est invalide ; vérifiez la base et sa sauvegarde.".to_string())?;
    let etat = compute_flux_state(statut, date, record.montant_prevu_cents, montant_realise_cents, Local::now().date_naive());
    Ok(FluxView {
        id: record.id, societe_id: record.societe_id, libelle: record.libelle, sens, categorie,
        date_prevue: record.date_prevue, montant_prevu_cents: record.montant_prevu_cents,
        tiers: record.tiers, reference: record.reference, statut, etat, montant_realise_cents,
        reste_cents: record.montant_prevu_cents - montant_realise_cents, realisations,
        created_at: record.created_at, updated_at: record.updated_at,
    })
}

#[tauri::command]
pub fn list_comptes(state: State<'_, AppState>) -> Result<Vec<superpower_core::Account>, String> {
    let connection = state.connection()?;
    let mut statement = connection
        .prepare("SELECT code, label, account_class FROM accounts ORDER BY code")
        .map_err(db_error)?;
    let rows = statement
        .query_map([], |row| {
            Ok(superpower_core::Account {
                code: row.get(0)?,
                label: row.get(1)?,
                account_class: row.get::<_, i64>(2)?.clamp(0, 9) as u8,
            })
        })
        .map_err(db_error)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(db_error)
}

#[tauri::command]
pub fn create_ecriture(
    input: CreateEntryInput,
    state: State<'_, AppState>,
) -> Result<EntryView, String> {
    if input.libelle.trim().is_empty() {
        return Err("Le libellé de l'écriture est obligatoire.".to_string());
    }
    let entry_date = NaiveDate::parse_from_str(&input.date_ecriture, "%Y-%m-%d")
        .map_err(|_| "La date de l'écriture doit être au format AAAA-MM-JJ.".to_string())?;
    let mut connection = state.connection()?;
    let society_exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM societes WHERE id = ?1)",
            [&input.societe_id],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    if !society_exists {
        return Err("La société sélectionnée n'existe pas.".to_string());
    }
    if input.lines.iter().any(|line| {
        line.line_label
            .as_ref()
            .is_some_and(|label| label.len() > 300)
    }) {
        return Err("Le libellé d'une ligne ne peut pas dépasser 300 caractères.".to_string());
    }
    let mut lines = input.lines;
    for line in &mut lines {
        let label: Option<String> = connection
            .query_row(
                "SELECT label FROM accounts WHERE code = ?1",
                [&line.account_code],
                |row| row.get(0),
            )
            .optional()
            .map_err(db_error)?;
        match label {
            Some(authoritative_label) => line.account_label = authoritative_label,
            None => {
                return Err(format!(
                    "Le compte {} n'existe pas dans le plan de comptes chargé.",
                    line.account_code
                ))
            }
        }
    }
    let total_cents = validate_entry_lines(&lines).map_err(|error| error.to_string())?;
    let id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    let reference = input
        .reference
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let transaction = connection.transaction().map_err(db_error)?;
    transaction.execute(
        "INSERT INTO ecritures(id, societe_id, date_ecriture, libelle, reference, statut, total_cents, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 'validated', ?6, ?7)",
        params![id, input.societe_id, entry_date.to_string(), input.libelle.trim(), reference, total_cents, created_at],
    ).map_err(db_error)?;
    for (index, line) in lines.iter().enumerate() {
        transaction.execute(
            "INSERT INTO lignes_ecriture(id, ecriture_id, ordre, compte_code, compte_libelle, libelle, debit_cents, credit_cents) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![Uuid::new_v4().to_string(), id, index as i64, line.account_code, line.account_label,
                line.line_label, line.debit_cents, line.credit_cents],
        ).map_err(db_error)?;
    }
    audit_tx(
        &transaction,
        "validate",
        "ecriture",
        &id,
        serde_json::json!({"societeId": input.societe_id, "totalCents": total_cents}),
    )
    .map_err(db_error)?;
    transaction.commit().map_err(db_error)?;
    Ok(EntryView {
        id,
        societe_id: input.societe_id,
        date_ecriture: entry_date.to_string(),
        libelle: input.libelle.trim().to_string(),
        reference,
        statut: "validated".to_string(),
        total_cents,
        lines,
    })
}

#[tauri::command]
pub fn list_ecritures(
    societe_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<EntryView>, String> {
    let connection = state.connection()?;
    let mut entries_statement = connection.prepare(
        "SELECT id, societe_id, date_ecriture, libelle, reference, statut, total_cents FROM ecritures WHERE (?1 IS NULL OR societe_id = ?1) ORDER BY date_ecriture DESC, created_at DESC LIMIT 500"
    ).map_err(db_error)?;
    let rows = entries_statement
        .query_map([societe_id], |row| {
            Ok(EntryView {
                id: row.get(0)?,
                societe_id: row.get(1)?,
                date_ecriture: row.get(2)?,
                libelle: row.get(3)?,
                reference: row.get(4)?,
                statut: row.get(5)?,
                total_cents: row.get(6)?,
                lines: Vec::new(),
            })
        })
        .map_err(db_error)?;
    let mut entries = Vec::new();
    for row in rows {
        let mut entry = row.map_err(db_error)?;
        let mut lines_statement = connection.prepare(
            "SELECT compte_code, compte_libelle, debit_cents, credit_cents, libelle FROM lignes_ecriture WHERE ecriture_id = ?1 ORDER BY ordre"
        ).map_err(db_error)?;
        let lines = lines_statement
            .query_map([&entry.id], |row| {
                Ok(EntryLine {
                    account_code: row.get(0)?,
                    account_label: row.get(1)?,
                    debit_cents: row.get(2)?,
                    credit_cents: row.get(3)?,
                    line_label: row.get(4)?,
                })
            })
            .map_err(db_error)?;
        entry.lines = lines.collect::<Result<Vec<_>, _>>().map_err(db_error)?;
        entries.push(entry);
    }
    Ok(entries)
}

#[tauri::command]
pub fn dashboard_summary(state: State<'_, AppState>) -> Result<DashboardSummary, String> {
    let connection = state.connection()?;
    let societes = connection
        .query_row("SELECT COUNT(*) FROM societes", [], |row| row.get(0))
        .map_err(db_error)?;
    let ecritures_validees = connection
        .query_row(
            "SELECT COUNT(*) FROM ecritures WHERE statut = 'validated'",
            [],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    let lignes_comptables = connection
        .query_row("SELECT COUNT(*) FROM lignes_ecriture", [], |row| row.get(0))
        .map_err(db_error)?;
    let volume_debit_cents = connection
        .query_row(
            "SELECT COALESCE(SUM(total_cents), 0) FROM ecritures WHERE statut = 'validated'",
            [],
            |row| row.get(0),
        )
        .map_err(db_error)?;
    Ok(DashboardSummary {
        societes,
        ecritures_validees,
        lignes_comptables,
        volume_debit_cents,
    })
}

fn audit_tx(
    transaction: &rusqlite::Transaction<'_>,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    details: serde_json::Value,
) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO journal_audit(id, action, type_entite, entite_id, horodatage, details_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![Uuid::new_v4().to_string(), action, entity_type, entity_id, chrono::Utc::now().to_rfc3339(), details.to_string()],
    )?;
    Ok(())
}

fn enum_name<T: Serialize>(value: T) -> Result<String, String> {
    serde_json::to_value(value)
        .map_err(|error| error.to_string())?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "Valeur de configuration invalide.".to_string())
}

fn from_enum<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, String> {
    serde_json::from_value(serde_json::Value::String(value.to_string()))
        .map_err(|error| format!("Valeur stockée invalide : {error}"))
}

fn db_error(error: rusqlite::Error) -> String {
    format!("Erreur de base de données : {error}")
}
