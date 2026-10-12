use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub code: String,
    pub label: String,
    pub account_class: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryLine {
    pub account_code: String,
    pub account_label: String,
    pub debit_cents: i64,
    pub credit_cents: i64,
    pub line_label: Option<String>,
}

impl EntryLine {
    pub fn debit(account_code: &str, account_label: &str, cents: i64) -> Self {
        Self {
            account_code: account_code.to_string(),
            account_label: account_label.to_string(),
            debit_cents: cents,
            credit_cents: 0,
            line_label: None,
        }
    }
    pub fn credit(account_code: &str, account_label: &str, cents: i64) -> Self {
        Self {
            account_code: account_code.to_string(),
            account_label: account_label.to_string(),
            debit_cents: 0,
            credit_cents: cents,
            line_label: None,
        }
    }
}

pub fn validate_entry_lines(lines: &[EntryLine]) -> Result<i64, EntryValidationError> {
    if lines.len() < 2 {
        return Err(EntryValidationError::TooFewLines);
    }
    let mut total_debit: i64 = 0;
    let mut total_credit: i64 = 0;
    for (index, line) in lines.iter().enumerate() {
        if line.account_code.trim().is_empty() {
            return Err(EntryValidationError::MissingAccount { line: index + 1 });
        }
        if line.debit_cents < 0 || line.credit_cents < 0 {
            return Err(EntryValidationError::NegativeAmount { line: index + 1 });
        }
        if line.debit_cents > 0 && line.credit_cents > 0 {
            return Err(EntryValidationError::BothSidesFilled { line: index + 1 });
        }
        if line.debit_cents == 0 && line.credit_cents == 0 {
            return Err(EntryValidationError::EmptyLine { line: index + 1 });
        }
        total_debit = total_debit
            .checked_add(line.debit_cents)
            .ok_or(EntryValidationError::AmountOverflow)?;
        total_credit = total_credit
            .checked_add(line.credit_cents)
            .ok_or(EntryValidationError::AmountOverflow)?;
    }
    if total_debit == 0 || total_credit == 0 {
        return Err(EntryValidationError::ZeroEntry);
    }
    if total_debit != total_credit {
        return Err(EntryValidationError::Unbalanced {
            debit_cents: total_debit,
            credit_cents: total_credit,
        });
    }
    Ok(total_debit)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntryValidationError {
    #[error("Une écriture doit contenir au moins deux lignes.")]
    TooFewLines,
    #[error("Le compte de la ligne {line} est obligatoire.")]
    MissingAccount { line: usize },
    #[error("La ligne {line} contient un montant négatif.")]
    NegativeAmount { line: usize },
    #[error("La ligne {line} ne peut pas avoir à la fois un débit et un crédit.")]
    BothSidesFilled { line: usize },
    #[error("La ligne {line} ne comporte aucun montant.")]
    EmptyLine { line: usize },
    #[error("Le total de l'écriture est nul.")]
    ZeroEntry,
    #[error("Les montants dépassent la capacité autorisée.")]
    AmountOverflow,
    #[error("L'écriture n'est pas équilibrée : débit {debit_cents} centimes, crédit {credit_cents} centimes.")]
    Unbalanced { debit_cents: i64, credit_cents: i64 },
}

/// Socle de comptes illustratif pour l'alpha. Ce n'est pas le PCG exhaustif validé.
pub fn starter_accounts() -> Vec<Account> {
    [
        ("101", "Capital"),
        ("106", "Réserves"),
        ("110", "Report à nouveau créditeur"),
        ("120", "Résultat de l'exercice - bénéfice"),
        ("129", "Résultat de l'exercice - perte"),
        ("164", "Emprunts auprès des établissements de crédit"),
        ("213", "Constructions"),
        ("218", "Autres immobilisations corporelles"),
        ("2813", "Amortissements des constructions"),
        ("401", "Fournisseurs"),
        ("411", "Clients"),
        ("44566", "TVA déductible sur autres biens et services"),
        ("44571", "TVA collectée"),
        ("467", "Autres comptes débiteurs ou créditeurs"),
        ("471", "Compte d'attente"),
        ("512", "Banque"),
        ("530", "Caisse"),
        ("606", "Achats non stockés de matières et fournitures"),
        ("613", "Locations"),
        ("616", "Primes d'assurances"),
        ("622", "Rémunérations d'intermédiaires et honoraires"),
        ("63512", "Taxes foncières"),
        ("661", "Charges d'intérêts"),
        ("706", "Prestations de services"),
        ("708", "Produits des activités annexes"),
        ("758", "Produits divers de gestion courante"),
    ]
    .into_iter()
    .map(|(code, label)| Account {
        code: code.to_string(),
        label: label.to_string(),
        account_class: code
            .chars()
            .next()
            .and_then(|c| c.to_digit(10))
            .unwrap_or(0) as u8,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn balanced() -> Vec<EntryLine> {
        vec![
            EntryLine::debit("512", "Banque", 12_345),
            EntryLine::credit("706", "Produits", 12_345),
        ]
    }

    #[test]
    fn accepts_a_balanced_entry_in_integer_cents() {
        assert_eq!(validate_entry_lines(&balanced()), Ok(12_345));
    }

    #[test]
    fn rejects_unbalanced_entry() {
        let lines = vec![
            EntryLine::debit("512", "Banque", 10_000),
            EntryLine::credit("706", "Produits", 9_999),
        ];
        assert_eq!(
            validate_entry_lines(&lines),
            Err(EntryValidationError::Unbalanced {
                debit_cents: 10_000,
                credit_cents: 9_999
            })
        );
    }

    #[test]
    fn rejects_line_with_both_debit_and_credit() {
        let lines = vec![
            EntryLine {
                account_code: "512".to_string(),
                account_label: "Banque".to_string(),
                debit_cents: 1,
                credit_cents: 1,
                line_label: None,
            },
            EntryLine::credit("706", "Produits", 1),
        ];
        assert_eq!(
            validate_entry_lines(&lines),
            Err(EntryValidationError::BothSidesFilled { line: 1 })
        );
    }

    #[test]
    fn rejects_negative_and_zero_lines() {
        assert_eq!(
            validate_entry_lines(&[
                EntryLine::debit("512", "Banque", -1),
                EntryLine::credit("706", "Produits", 1)
            ]),
            Err(EntryValidationError::NegativeAmount { line: 1 })
        );
        assert_eq!(
            validate_entry_lines(&[
                EntryLine::debit("512", "Banque", 0),
                EntryLine::credit("706", "Produits", 1)
            ]),
            Err(EntryValidationError::EmptyLine { line: 1 })
        );
    }
}
