//! Domaine des flux prévisionnels et des réalisations de trésorerie.
//! Ce module ne déduit ni compte PCG ni traitement de TVA.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensFlux {
    Encaissement,
    Decaissement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CategorieFlux {
    Loyer,
    ChargeCourante,
    Travaux,
    ImpotTaxe,
    Emprunt,
    ApportAssocie,
    Autre,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatutFlux {
    Actif,
    Annule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatFlux {
    #[serde(rename = "a_prevoir")]
    APrevoir,
    #[serde(rename = "en_retard")]
    EnRetard,
    #[serde(rename = "partiellement_realise")]
    PartiellementRealise,
    #[serde(rename = "realise")]
    Realise,
    #[serde(rename = "depasse")]
    Depasse,
    #[serde(rename = "annule")]
    Annule,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FluxInput {
    pub societe_id: String,
    pub libelle: String,
    pub sens: SensFlux,
    pub categorie: CategorieFlux,
    pub date_prevue: String,
    pub montant_prevu_cents: i64,
    pub tiers: Option<String>,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealisationFluxInput {
    pub date_realisation: String,
    pub montant_cents: i64,
    pub reference: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FluxRealisationView {
    pub id: String,
    pub flux_id: String,
    pub date_realisation: String,
    pub montant_cents: i64,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
    pub annulee_at: Option<String>,
    pub motif_annulation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FluxView {
    pub id: String,
    pub societe_id: String,
    pub libelle: String,
    pub sens: SensFlux,
    pub categorie: CategorieFlux,
    pub date_prevue: String,
    pub montant_prevu_cents: i64,
    pub tiers: Option<String>,
    pub reference: Option<String>,
    pub statut: StatutFlux,
    pub etat: EtatFlux,
    pub montant_realise_cents: i64,
    /// Prévu moins réalisé ; cette valeur est négative en cas de dépassement.
    pub reste_cents: i64,
    pub realisations: Vec<FluxRealisationView>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum FluxValidationError {
    #[error("La société du flux est obligatoire.")]
    SocieteVide,
    #[error("Le libellé du flux est obligatoire.")]
    LibelleVide,
    #[error("Le libellé du flux ne peut pas dépasser 160 caractères.")]
    LibelleTropLong,
    #[error("La date prévue doit être au format AAAA-MM-JJ.")]
    DatePrevueInvalide,
    #[error("Le montant prévu doit être strictement positif.")]
    MontantPrevuInvalide,
    #[error("Le tiers ne peut pas dépasser 160 caractères.")]
    TiersTropLong,
    #[error("La référence ne peut pas dépasser 100 caractères.")]
    ReferenceTropLong,
    #[error("La date de réalisation doit être au format AAAA-MM-JJ.")]
    DateRealisationInvalide,
    #[error("Le montant réalisé doit être strictement positif.")]
    MontantRealiseInvalide,
    #[error("La note ne peut pas dépasser 500 caractères.")]
    NoteTropLongue,
}

pub fn validate_flux_input(input: &FluxInput) -> Result<(), FluxValidationError> {
    if input.societe_id.trim().is_empty() {
        return Err(FluxValidationError::SocieteVide);
    }
    if input.libelle.trim().is_empty() {
        return Err(FluxValidationError::LibelleVide);
    }
    if input.libelle.trim().chars().count() > 160 {
        return Err(FluxValidationError::LibelleTropLong);
    }
    if NaiveDate::parse_from_str(&input.date_prevue, "%Y-%m-%d").is_err() {
        return Err(FluxValidationError::DatePrevueInvalide);
    }
    if input.montant_prevu_cents <= 0 {
        return Err(FluxValidationError::MontantPrevuInvalide);
    }
    if input
        .tiers
        .as_deref()
        .unwrap_or_default()
        .trim()
        .chars()
        .count()
        > 160
    {
        return Err(FluxValidationError::TiersTropLong);
    }
    if input
        .reference
        .as_deref()
        .unwrap_or_default()
        .trim()
        .chars()
        .count()
        > 100
    {
        return Err(FluxValidationError::ReferenceTropLong);
    }
    Ok(())
}

pub fn validate_realisation_input(input: &RealisationFluxInput) -> Result<(), FluxValidationError> {
    if NaiveDate::parse_from_str(&input.date_realisation, "%Y-%m-%d").is_err() {
        return Err(FluxValidationError::DateRealisationInvalide);
    }
    if input.montant_cents <= 0 {
        return Err(FluxValidationError::MontantRealiseInvalide);
    }
    if input
        .reference
        .as_deref()
        .unwrap_or_default()
        .trim()
        .chars()
        .count()
        > 100
    {
        return Err(FluxValidationError::ReferenceTropLong);
    }
    if input
        .note
        .as_deref()
        .unwrap_or_default()
        .trim()
        .chars()
        .count()
        > 500
    {
        return Err(FluxValidationError::NoteTropLongue);
    }
    Ok(())
}

/// Le dépassement est autorisé mais explicite : le réalisé n'écrase jamais le prévu.
pub fn compute_flux_state(
    statut: StatutFlux,
    date_prevue: NaiveDate,
    montant_prevu_cents: i64,
    montant_realise_cents: i64,
    today: NaiveDate,
) -> EtatFlux {
    if statut == StatutFlux::Annule {
        return EtatFlux::Annule;
    }
    if montant_realise_cents > montant_prevu_cents {
        return EtatFlux::Depasse;
    }
    if montant_realise_cents == montant_prevu_cents {
        return EtatFlux::Realise;
    }
    if montant_realise_cents > 0 {
        return EtatFlux::PartiellementRealise;
    }
    if date_prevue < today {
        EtatFlux::EnRetard
    } else {
        EtatFlux::APrevoir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn flow_state_distinguishes_forecast_overdue_partial_realized_and_overrun() {
        let today = date("2026-10-13");
        assert_eq!(
            compute_flux_state(StatutFlux::Actif, date("2026-10-14"), 10_000, 0, today),
            EtatFlux::APrevoir
        );
        assert_eq!(
            compute_flux_state(StatutFlux::Actif, date("2026-10-12"), 10_000, 0, today),
            EtatFlux::EnRetard
        );
        assert_eq!(
            compute_flux_state(StatutFlux::Actif, date("2026-10-12"), 10_000, 3_000, today),
            EtatFlux::PartiellementRealise
        );
        assert_eq!(
            compute_flux_state(StatutFlux::Actif, date("2026-10-12"), 10_000, 10_000, today),
            EtatFlux::Realise
        );
        assert_eq!(
            compute_flux_state(StatutFlux::Actif, date("2026-10-12"), 10_000, 12_000, today),
            EtatFlux::Depasse
        );
        assert_eq!(
            compute_flux_state(StatutFlux::Annule, date("2026-10-12"), 10_000, 0, today),
            EtatFlux::Annule
        );
    }

    #[test]
    fn validation_rejects_zero_amount_and_blank_label() {
        let mut input = FluxInput {
            societe_id: "soc-1".into(),
            libelle: "Loyer".into(),
            sens: SensFlux::Encaissement,
            categorie: CategorieFlux::Loyer,
            date_prevue: "2026-10-31".into(),
            montant_prevu_cents: 0,
            tiers: None,
            reference: None,
        };
        assert_eq!(
            validate_flux_input(&input),
            Err(FluxValidationError::MontantPrevuInvalide)
        );
        input.montant_prevu_cents = 1;
        input.libelle = "   ".into();
        assert_eq!(
            validate_flux_input(&input),
            Err(FluxValidationError::LibelleVide)
        );
    }

    #[test]
    fn validation_accepts_valid_partial_actualization() {
        let input = RealisationFluxInput {
            date_realisation: "2026-10-13".into(),
            montant_cents: 1,
            reference: None,
            note: Some("Versement partiel".into()),
        };
        assert_eq!(validate_realisation_input(&input), Ok(()));
        let invalid = RealisationFluxInput {
            montant_cents: -1,
            ..input
        };
        assert_eq!(
            validate_realisation_input(&invalid),
            Err(FluxValidationError::MontantRealiseInvalide)
        );
    }
}
