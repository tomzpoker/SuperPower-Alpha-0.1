//! Règles de domaine pures de SuperPower. Aucune dépendance UI ou base de données.

pub mod accounting;
pub mod events;
pub mod society;

pub use accounting::{validate_entry_lines, Account, EntryLine, EntryValidationError};
pub use society::{
    CompanyWarnings, ExigibiliteTva, FormeJuridique, PeriodiciteTva, RegimeFiscal, RegimeTva,
    Societe, ValidationSocieteError,
};
