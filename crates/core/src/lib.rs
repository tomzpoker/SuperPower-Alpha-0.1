//! Règles de domaine pures de SuperPower. Aucune dépendance UI ou base de données.

pub mod accounting;
pub mod events;
pub mod flows;
pub mod recurrence;
pub mod society;

pub use accounting::{validate_entry_lines, Account, EntryLine, EntryValidationError};
pub use flows::{
    compute_flux_state, validate_flux_input, validate_realisation_input, CategorieFlux, EtatFlux,
    FluxInput, FluxRealisationView, FluxValidationError, FluxView, RealisationFluxInput, SensFlux,
    StatutFlux,
};
pub use society::{
    CompanyWarnings, ExigibiliteTva, FormeJuridique, PeriodiciteTva, RegimeFiscal, RegimeTva,
    Societe, ValidationSocieteError,
};

pub use recurrence::{generer_echeances, ErreurRecurrence, FrequenceFlux};
