use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormeJuridique {
    Sci,
    Sarl,
    Sas,
    Autre,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegimeFiscal {
    Ir,
    Is,
    NonRenseigne,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegimeTva {
    NonRenseigne,
    HorsChamp,
    FranchiseBase,
    ReelSimplifie,
    ReelNormal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExigibiliteTva {
    NonDefinie,
    NonApplicable,
    Encaissements,
    Debits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodiciteTva {
    NonDefinie,
    NonApplicable,
    Mensuelle,
    Trimestrielle,
    Annuelle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Societe {
    pub id: String,
    pub nom: String,
    pub forme_juridique: FormeJuridique,
    pub regime_fiscal: RegimeFiscal,
    pub regime_tva: RegimeTva,
    pub exigibilite_tva: ExigibiliteTva,
    pub periodicite_tva: PeriodiciteTva,
    pub siret: Option<String>,
    pub fin_exercice: String,
    pub created_at: String,
}

impl Societe {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        nom: String,
        forme_juridique: FormeJuridique,
        regime_fiscal: RegimeFiscal,
        regime_tva: RegimeTva,
        exigibilite_tva: ExigibiliteTva,
        periodicite_tva: PeriodiciteTva,
        siret: Option<String>,
        fin_exercice: String,
    ) -> Result<Self, ValidationSocieteError> {
        let societe = Self {
            id: Uuid::new_v4().to_string(),
            nom: nom.trim().to_owned(),
            forme_juridique,
            regime_fiscal,
            regime_tva,
            exigibilite_tva,
            periodicite_tva,
            siret: siret
                .map(|value| value.trim().to_owned())
                .filter(|v| !v.is_empty()),
            fin_exercice,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        societe.validate()?;
        Ok(societe)
    }

    pub fn validate(&self) -> Result<(), ValidationSocieteError> {
        if self.nom.trim().is_empty() {
            return Err(ValidationSocieteError::NomVide);
        }
        if let Some(siret) = &self.siret {
            let digits_only = siret.chars().all(|c| c.is_ascii_digit());
            if siret.len() != 14 || !digits_only {
                return Err(ValidationSocieteError::SiretInvalide);
            }
        }
        chrono::NaiveDate::parse_from_str(&self.fin_exercice, "%Y-%m-%d")
            .map_err(|_| ValidationSocieteError::DateClotureInvalide)?;
        Ok(())
    }

    /// Fournit des points d'attention, sans interdire les options légales particulières.
    /// Ce sont des avertissements d'interface, jamais un avis fiscal définitif.
    pub fn warnings(&self) -> CompanyWarnings {
        let mut warnings = Vec::new();
        match (self.forme_juridique, self.regime_fiscal) {
            (FormeJuridique::Sci, RegimeFiscal::Is) => warnings.push(
                "Une SCI peut relever de l'IS dans certaines situations : vérifier l'option et ses conséquences avec un professionnel.".to_string(),
            ),
            (FormeJuridique::Sarl, RegimeFiscal::Ir) => warnings.push(
                "Une SARL à l'IR n'est possible que dans certains cas : vérifier l'éligibilité et la durée de l'option.".to_string(),
            ),
            (_, RegimeFiscal::NonRenseigne) => warnings.push(
                "Le régime d'imposition du résultat n'est pas encore renseigné.".to_string(),
            ),
            _ => {}
        }
        if self.regime_tva == RegimeTva::NonRenseigne {
            warnings.push("Le régime de TVA reste à confirmer pour cette société.".to_string());
        }
        if matches!(
            self.regime_tva,
            RegimeTva::ReelNormal | RegimeTva::ReelSimplifie
        ) && self.exigibilite_tva == ExigibiliteTva::NonDefinie
        {
            warnings.push("Le mode d'exigibilité de la TVA reste à renseigner (encaissements ou débits selon l'activité et l'option applicable).".to_string());
        }
        if self.regime_tva == RegimeTva::ReelNormal
            && self.periodicite_tva == PeriodiciteTva::Annuelle
        {
            warnings.push("Une périodicité annuelle avec un régime réel normal mérite une vérification : les obligations dépendent du dossier fiscal.".to_string());
        }
        CompanyWarnings { items: warnings }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompanyWarnings {
    pub items: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationSocieteError {
    #[error("Le nom de la société est obligatoire.")]
    NomVide,
    #[error("Le SIRET doit contenir exactement 14 chiffres.")]
    SiretInvalide,
    #[error("La date de clôture doit être une date valide au format AAAA-MM-JJ.")]
    DateClotureInvalide,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_company(siret: Option<&str>) -> Result<Societe, ValidationSocieteError> {
        Societe::new(
            " SCI Démo ".to_string(),
            FormeJuridique::Sci,
            RegimeFiscal::Ir,
            RegimeTva::ReelNormal,
            ExigibiliteTva::Encaissements,
            PeriodiciteTva::Mensuelle,
            siret.map(str::to_owned),
            "2026-12-31".to_string(),
        )
    }

    #[test]
    fn trims_company_name_and_accepts_valid_siret() {
        let company = new_company(Some("12345678901234")).expect("valid company");
        assert_eq!(company.nom, "SCI Démo");
    }

    #[test]
    fn rejects_invalid_siret() {
        assert_eq!(
            new_company(Some("1234")),
            Err(ValidationSocieteError::SiretInvalide)
        );
    }

    #[test]
    fn warns_when_vat_regime_is_unknown() {
        let company = Societe::new(
            "SARL Démo".to_string(),
            FormeJuridique::Sarl,
            RegimeFiscal::Is,
            RegimeTva::NonRenseigne,
            ExigibiliteTva::NonDefinie,
            PeriodiciteTva::NonDefinie,
            None,
            "2026-12-31".to_string(),
        )
        .expect("valid company");
        assert!(company
            .warnings()
            .items
            .iter()
            .any(|warning| warning.contains("TVA")));
    }
}
