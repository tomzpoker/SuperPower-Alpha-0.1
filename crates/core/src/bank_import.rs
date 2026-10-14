//! Import bancaire CSV : normalisation déterministe avant toute persistance.
//!
//! Cette couche de domaine ne touche ni à SQLCipher ni aux écritures comptables.
//! Le mapping est explicite et ses indices de colonne commencent à zéro.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

const MAX_CSV_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvColumnMapping {
    pub date_column: usize,
    pub label_column: usize,
    pub amount_column: Option<usize>,
    pub debit_column: Option<usize>,
    pub credit_column: Option<usize>,
    pub reference_column: Option<usize>,
}

impl CsvColumnMapping {
    fn validate(&self) -> Result<(), CsvPreviewError> {
        let mode_is_valid = matches!(
            (self.amount_column, self.debit_column, self.credit_column),
            (Some(_), None, None) | (None, Some(_), Some(_))
        );
        if !mode_is_valid {
            return Err(CsvPreviewError::InvalidMapping(
                "Choisir soit une colonne montant, soit les deux colonnes débit et crédit."
                    .to_string(),
            ));
        }

        let mut columns = vec![("date", self.date_column), ("libellé", self.label_column)];
        if let Some(column) = self.amount_column {
            columns.push(("montant", column));
        }
        if let Some(column) = self.debit_column {
            columns.push(("débit", column));
        }
        if let Some(column) = self.credit_column {
            columns.push(("crédit", column));
        }
        if let Some(column) = self.reference_column {
            columns.push(("référence", column));
        }
        for (position, (name, column)) in columns.iter().enumerate() {
            if let Some((other_name, _)) = columns[..position]
                .iter()
                .find(|(_, other_column)| other_column == column)
            {
                return Err(CsvPreviewError::InvalidMapping(format!(
                    "Les colonnes {other_name} et {name} ne peuvent pas être identiques."
                )));
            }
        }
        Ok(())
    }

    fn max_column(&self) -> usize {
        [
            Some(self.date_column),
            Some(self.label_column),
            self.amount_column,
            self.debit_column,
            self.credit_column,
            self.reference_column,
        ]
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankTransactionPreview {
    /// Numéro d'enregistrement CSV logique (l'en-tête compte comme le premier).
    pub source_row: usize,
    /// Date normalisée en AAAA-MM-JJ.
    pub booking_date: String,
    pub label: String,
    /// Montant signé en centimes : crédit positif, débit négatif.
    pub amount_cents: i64,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvImportRowError {
    pub source_row: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CsvImportPreview {
    pub headers: Option<Vec<String>>,
    pub rows: Vec<BankTransactionPreview>,
    pub errors: Vec<CsvImportRowError>,
    /// Nombre de lignes de données lues, y compris celles rejetées.
    pub total_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CsvPreviewError {
    #[error("Séparateur CSV invalide : seuls virgule, point-virgule et tabulation sont acceptés.")]
    InvalidDelimiter,
    #[error("Mapping CSV invalide : {0}")]
    InvalidMapping(String),
    #[error("CSV mal formé au caractère {position} : {message}")]
    MalformedCsv { position: usize, message: String },
    #[error("Le fichier CSV dépasse la limite de {max_bytes} octets.")]
    FileTooLarge { max_bytes: usize },
    #[error("Le fichier CSV est vide.")]
    EmptyCsv,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CsvRecordError {
    MissingColumn(&'static str, usize),
    InvalidDate(String),
    InvalidAmount(String),
    EmptyLabel,
}

impl std::fmt::Display for CsvRecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingColumn(name, column) => write!(
                f,
                "colonne {name} (n° {}) absente de cette ligne",
                column + 1
            ),
            Self::InvalidDate(value) => write!(
                f,
                "date invalide « {value} » (formats acceptés : AAAA-MM-JJ ou JJ/MM/AAAA)"
            ),
            Self::InvalidAmount(value) => write!(f, "montant invalide « {value} »"),
            Self::EmptyLabel => write!(f, "libellé vide"),
        }
    }
}

/// Construit un aperçu CSV. Cette fonction est pure : elle ne persiste aucune ligne.
///
/// Les indices de colonnes du mapping sont basés sur zéro. Les erreurs de données sont
/// retournées ligne par ligne pour qu'un écran puisse les présenter avant confirmation.
pub fn parse_csv_preview(
    input: &str,
    delimiter: char,
    has_headers: bool,
    mapping: &CsvColumnMapping,
) -> Result<CsvImportPreview, CsvPreviewError> {
    if input.len() > MAX_CSV_BYTES {
        return Err(CsvPreviewError::FileTooLarge {
            max_bytes: MAX_CSV_BYTES,
        });
    }
    if !matches!(delimiter, ',' | ';' | '\t') {
        return Err(CsvPreviewError::InvalidDelimiter);
    }
    mapping.validate()?;
    let mut records = parse_records(input, delimiter)?;
    if let Some(first_field) = records.first_mut().and_then(|record| record.first_mut()) {
        if let Some(stripped) = first_field.strip_prefix('\u{feff}') {
            *first_field = stripped.to_string();
        }
    }
    if records.is_empty() {
        return Err(CsvPreviewError::EmptyCsv);
    }

    let headers = if has_headers {
        Some(records.remove(0))
    } else {
        None
    };
    if let Some(headers) = headers.as_ref() {
        if mapping.max_column() >= headers.len() {
            return Err(CsvPreviewError::InvalidMapping(format!(
                "Le mapping demande la colonne {}, mais l'en-tête ne contient que {} colonnes.",
                mapping.max_column() + 1,
                headers.len()
            )));
        }
    }

    let mut preview = CsvImportPreview {
        headers,
        rows: Vec::new(),
        errors: Vec::new(),
        total_rows: records.len(),
    };
    let first_data_record = if has_headers { 2 } else { 1 };
    for (offset, record) in records.iter().enumerate() {
        let source_row = first_data_record + offset;
        match normalize_record(record, source_row, mapping) {
            Ok(row) => preview.rows.push(row),
            Err(error) => preview.errors.push(CsvImportRowError {
                source_row,
                message: error.to_string(),
            }),
        }
    }
    Ok(preview)
}

fn parse_records(input: &str, delimiter: char) -> Result<Vec<Vec<String>>, CsvPreviewError> {
    let characters: Vec<char> = input.chars().collect();
    let mut records = Vec::new();
    let mut record: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut after_quote = false;
    let mut position = 0;

    while position < characters.len() {
        let character = characters[position];
        if in_quotes {
            if character == '"' {
                if characters.get(position + 1) == Some(&'"') {
                    field.push('"');
                    position += 2;
                } else {
                    in_quotes = false;
                    after_quote = true;
                    position += 1;
                }
            } else {
                field.push(character);
                position += 1;
            }
            continue;
        }

        if after_quote {
            match character {
                ch if ch == delimiter => {
                    record.push(std::mem::take(&mut field));
                    after_quote = false;
                    position += 1;
                }
                '\n' | '\r' => {
                    finish_record(&mut records, &mut record, &mut field);
                    after_quote = false;
                    if character == '\r' && characters.get(position + 1) == Some(&'\n') {
                        position += 2;
                    } else {
                        position += 1;
                    }
                }
                ch if ch.is_whitespace() => position += 1,
                _ => {
                    return Err(CsvPreviewError::MalformedCsv {
                        position,
                        message: "caractère inattendu après la fermeture d'un champ cité"
                            .to_string(),
                    });
                }
            }
            continue;
        }

        match character {
            '"' if field.is_empty() => {
                in_quotes = true;
                position += 1;
            }
            '"' => {
                return Err(CsvPreviewError::MalformedCsv {
                    position,
                    message: "guillemet inattendu au milieu d'un champ non cité".to_string(),
                });
            }
            ch if ch == delimiter => {
                record.push(std::mem::take(&mut field));
                position += 1;
            }
            '\n' | '\r' => {
                finish_record(&mut records, &mut record, &mut field);
                if character == '\r' && characters.get(position + 1) == Some(&'\n') {
                    position += 2;
                } else {
                    position += 1;
                }
            }
            _ => {
                field.push(character);
                position += 1;
            }
        }
    }

    if in_quotes {
        return Err(CsvPreviewError::MalformedCsv {
            position: characters.len(),
            message: "champ cité non terminé".to_string(),
        });
    }
    if !field.is_empty() || !record.is_empty() || after_quote {
        finish_record(&mut records, &mut record, &mut field);
    }
    Ok(records)
}

fn finish_record(records: &mut Vec<Vec<String>>, record: &mut Vec<String>, field: &mut String) {
    record.push(std::mem::take(field));
    if record.iter().any(|value| !value.trim().is_empty()) {
        records.push(std::mem::take(record));
    } else {
        record.clear();
    }
}

fn normalize_record(
    record: &[String],
    source_row: usize,
    mapping: &CsvColumnMapping,
) -> Result<BankTransactionPreview, CsvRecordError> {
    let date_value = required_cell(record, mapping.date_column, "date")?;
    let booking_date = normalize_date(date_value)
        .map_err(|_| CsvRecordError::InvalidDate(date_value.to_string()))?;
    let label = required_cell(record, mapping.label_column, "libellé")?
        .trim()
        .to_string();
    if label.is_empty() {
        return Err(CsvRecordError::EmptyLabel);
    }

    let amount_cents = if let Some(column) = mapping.amount_column {
        let value = required_cell(record, column, "montant")?;
        parse_amount_cents(value).map_err(|_| CsvRecordError::InvalidAmount(value.to_string()))?
    } else {
        let debit_column = mapping.debit_column.expect("mapping validated");
        let credit_column = mapping.credit_column.expect("mapping validated");
        let debit_value = required_cell(record, debit_column, "débit")?;
        let credit_value = required_cell(record, credit_column, "crédit")?;
        let debit = if debit_value.trim().is_empty() {
            0
        } else {
            parse_amount_cents(debit_value)
                .map_err(|_| CsvRecordError::InvalidAmount(debit_value.to_string()))?
        };
        let credit = if credit_value.trim().is_empty() {
            0
        } else {
            parse_amount_cents(credit_value)
                .map_err(|_| CsvRecordError::InvalidAmount(credit_value.to_string()))?
        };
        if debit < 0 || credit < 0 {
            return Err(CsvRecordError::InvalidAmount(
                "les colonnes débit/crédit doivent contenir des montants positifs".to_string(),
            ));
        }
        if debit > 0 && credit > 0 {
            return Err(CsvRecordError::InvalidAmount(
                "débit et crédit sont tous deux renseignés".to_string(),
            ));
        }
        if debit > 0 {
            -debit
        } else {
            credit
        }
    };
    if amount_cents == 0 {
        return Err(CsvRecordError::InvalidAmount(
            "le montant doit être différent de zéro".to_string(),
        ));
    }

    let reference = mapping
        .reference_column
        .and_then(|column| optional_cell(record, column))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    Ok(BankTransactionPreview {
        source_row,
        booking_date,
        label,
        amount_cents,
        reference,
    })
}

fn required_cell<'a>(
    record: &'a [String],
    column: usize,
    name: &'static str,
) -> Result<&'a str, CsvRecordError> {
    record
        .get(column)
        .map(String::as_str)
        .map(str::trim)
        .ok_or(CsvRecordError::MissingColumn(name, column))
}

fn optional_cell(record: &[String], column: usize) -> Option<&str> {
    record.get(column).map(String::as_str)
}

fn normalize_date(value: &str) -> Result<String, ()> {
    let value = value.trim();
    for format in ["%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y"] {
        if let Ok(date) = NaiveDate::parse_from_str(value, format) {
            return Ok(date.format("%Y-%m-%d").to_string());
        }
    }
    Err(())
}

/// Lit les montants en centimes, sans passage par un flottant.
/// La virgule française est prise en charge ; si virgule et point sont présents,
/// le dernier séparateur est la décimale et l'autre sert au groupement des milliers.
fn parse_amount_cents(raw: &str) -> Result<i64, ()> {
    let mut value = raw
        .trim()
        .to_uppercase()
        .replace("EUR", "")
        .replace('€', "");
    value = value.trim().to_string();
    if value.is_empty() {
        return Err(());
    }

    let parenthetical_negative = value.starts_with('(') && value.ends_with(')');
    if parenthetical_negative {
        value = value[1..value.len() - 1].trim().to_string();
    } else if value.starts_with('(') || value.ends_with(')') {
        return Err(());
    }
    let mut negative = parenthetical_negative;
    if value.starts_with('-') {
        if negative {
            return Err(());
        }
        negative = true;
        value = value[1..].to_string();
    } else if value.starts_with('+') {
        if negative {
            return Err(());
        }
        value = value[1..].to_string();
    }
    value = value.trim().replace('\'', " ");
    if value.is_empty() {
        return Err(());
    }

    let comma_count = value.matches(',').count();
    let dot_count = value.matches('.').count();
    let (integer_part, fraction_part, grouping_separator) = if comma_count > 0 && dot_count > 0 {
        let comma = value.rfind(',').ok_or(())?;
        let dot = value.rfind('.').ok_or(())?;
        let decimal_separator = if comma > dot { ',' } else { '.' };
        let decimal_position = value.rfind(decimal_separator).ok_or(())?;
        let integer = &value[..decimal_position];
        let fraction = &value[decimal_position + 1..];
        let grouping = if decimal_separator == ',' { '.' } else { ',' };
        (integer.to_string(), fraction.to_string(), Some(grouping))
    } else if comma_count + dot_count > 1 {
        let separator = if comma_count > 0 { ',' } else { '.' };
        let groups: Vec<&str> = value.split(separator).collect();
        let valid_grouping = !groups.is_empty()
            && (1..=3).contains(&groups[0].len())
            && groups
                .iter()
                .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
            && groups.iter().skip(1).all(|part| part.len() == 3);
        if !valid_grouping {
            return Err(());
        }
        (groups.join(""), String::new(), None)
    } else if comma_count == 1 || dot_count == 1 {
        let separator = if comma_count == 1 { ',' } else { '.' };
        let position = value.find(separator).ok_or(())?;
        (
            value[..position].to_string(),
            value[position + 1..].to_string(),
            None,
        )
    } else {
        (value, String::new(), None)
    };

    if fraction_part.len() > 2 || !fraction_part.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(());
    }
    let integer = normalize_integer(&integer_part, grouping_separator)?;
    let whole: i64 = integer.parse().map_err(|_| ())?;
    let mut fraction: i64 = if fraction_part.is_empty() {
        0
    } else {
        fraction_part.parse().map_err(|_| ())?
    };
    if fraction_part.len() == 1 {
        fraction *= 10;
    }
    let cents = whole
        .checked_mul(100)
        .and_then(|n| n.checked_add(fraction))
        .ok_or(())?;
    if negative {
        cents.checked_neg().ok_or(())
    } else {
        Ok(cents)
    }
}

fn normalize_integer(raw: &str, grouping_separator: Option<char>) -> Result<String, ()> {
    let mut groups = String::with_capacity(raw.len());
    for character in raw.chars() {
        if character.is_ascii_digit() {
            groups.push(character);
        } else if character.is_whitespace()
            || character == '\''
            || grouping_separator == Some(character)
        {
            groups.push(' ');
        } else {
            return Err(());
        }
    }
    let parts: Vec<&str> = groups.split_whitespace().collect();
    if parts.is_empty()
        || !parts
            .iter()
            .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
    {
        return Err(());
    }
    if parts.len() > 1
        && (!(1..=3).contains(&parts[0].len()) || parts.iter().skip(1).any(|part| part.len() != 3))
    {
        return Err(());
    }
    Ok(parts.join(""))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount_mapping() -> CsvColumnMapping {
        CsvColumnMapping {
            date_column: 0,
            label_column: 1,
            amount_column: Some(2),
            debit_column: None,
            credit_column: None,
            reference_column: Some(3),
        }
    }

    #[test]
    fn parses_french_decimal_amounts_quoted_fields_and_dates() {
        let csv = "date;libellé;montant;référence\r\n31/01/2026;\"Loyer, local\";\"1 234,56 €\";VIR-01\r\n01/02/2026;\"Achat \"\"cartouche\"\"\";-12,50;CB-02\r\n";
        let preview = parse_csv_preview(csv, ';', true, &amount_mapping()).unwrap();
        assert_eq!(preview.total_rows, 2);
        assert_eq!(preview.errors, Vec::<CsvImportRowError>::new());
        assert_eq!(preview.rows[0].booking_date, "2026-01-31");
        assert_eq!(preview.rows[0].amount_cents, 123_456);
        assert_eq!(preview.rows[0].label, "Loyer, local");
        assert_eq!(preview.rows[1].amount_cents, -1_250);
        assert_eq!(preview.rows[1].label, "Achat \"cartouche\"");
        assert_eq!(preview.rows[0].source_row, 2);
    }

    #[test]
    fn supports_debit_credit_columns_with_zero_in_unused_column() {
        let csv = "date;libellé;débit;crédit\n2026-01-01;Assurance;120,00;0,00\n02/01/2026;Loyer;0,00;3 500,10";
        let mapping = CsvColumnMapping {
            date_column: 0,
            label_column: 1,
            amount_column: None,
            debit_column: Some(2),
            credit_column: Some(3),
            reference_column: None,
        };
        let preview = parse_csv_preview(csv, ';', true, &mapping).unwrap();
        assert_eq!(preview.errors.len(), 0);
        assert_eq!(preview.rows[0].amount_cents, -12_000);
        assert_eq!(preview.rows[1].amount_cents, 350_010);
    }

    #[test]
    fn reports_invalid_rows_without_losing_valid_rows() {
        let csv = "date;libellé;montant;référence\n31/02/2026;Ligne invalide;15,00;X\n2026-03-01;Ligne valide;15,00;Y\n2026-03-02;Montant invalide;1,234;Z";
        let preview = parse_csv_preview(csv, ';', true, &amount_mapping()).unwrap();
        assert_eq!(preview.total_rows, 3);
        assert_eq!(preview.rows.len(), 1);
        assert_eq!(preview.errors.len(), 2);
        assert_eq!(preview.errors[0].source_row, 2);
        assert!(preview.errors[0].message.contains("date invalide"));
        assert_eq!(preview.errors[1].source_row, 4);
        assert!(preview.errors[1].message.contains("montant invalide"));
    }

    #[test]
    fn strips_utf8_bom_from_the_first_header() {
        let csv = "\u{feff}date;libellé;montant;référence\n2026-01-01;Loyer;12,00;R-1";
        let preview = parse_csv_preview(csv, ';', true, &amount_mapping()).unwrap();
        assert_eq!(preview.headers.unwrap()[0], "date");
        assert_eq!(preview.rows[0].amount_cents, 1_200);
    }

    #[test]
    fn rejects_oversized_csv_before_parsing() {
        let csv = "x".repeat(MAX_CSV_BYTES + 1);
        assert_eq!(
            parse_csv_preview(&csv, ';', false, &amount_mapping()).unwrap_err(),
            CsvPreviewError::FileTooLarge {
                max_bytes: MAX_CSV_BYTES,
            }
        );
    }

    #[test]
    fn requires_both_configured_debit_credit_columns_in_each_row() {
        let csv = "date;libellé;débit;crédit\n2026-01-01;Loyer;10,00";
        let mapping = CsvColumnMapping {
            date_column: 0,
            label_column: 1,
            amount_column: None,
            debit_column: Some(2),
            credit_column: Some(3),
            reference_column: None,
        };
        let preview = parse_csv_preview(csv, ';', true, &mapping).unwrap();
        assert_eq!(preview.rows.len(), 0);
        assert_eq!(preview.errors.len(), 1);
        assert!(preview.errors[0].message.contains("colonne crédit"));
    }

    #[test]
    fn supports_multiline_quoted_labels() {
        let csv =
            "date;libellé;montant;référence\n2026-03-01;\"Paiement\nsupplémentaire\";10,00;REF-1";
        let preview = parse_csv_preview(csv, ';', true, &amount_mapping()).unwrap();
        assert_eq!(preview.rows[0].label, "Paiement\nsupplémentaire");
    }

    #[test]
    fn rejects_malformed_csv_and_ambiguous_mapping() {
        let csv_error = parse_csv_preview(
            "date;libellé;montant;référence\n2026-03-01;\"champ non terminé;10,00;REF",
            ';',
            true,
            &amount_mapping(),
        )
        .unwrap_err();
        assert!(matches!(csv_error, CsvPreviewError::MalformedCsv { .. }));

        let invalid_mapping = CsvColumnMapping {
            date_column: 0,
            label_column: 1,
            amount_column: Some(2),
            debit_column: Some(3),
            credit_column: None,
            reference_column: None,
        };
        assert!(matches!(
            parse_csv_preview("a;b;c;d\n1;2;3;4", ';', true, &invalid_mapping),
            Err(CsvPreviewError::InvalidMapping(_))
        ));
    }

    #[test]
    fn supports_us_separator_order_and_grouped_thousands() {
        assert_eq!(parse_amount_cents("1,234.56").unwrap(), 123_456);
        assert_eq!(parse_amount_cents("1.234,56").unwrap(), 123_456);
        assert_eq!(parse_amount_cents("1.234.567").unwrap(), 123_456_700);
        assert_eq!(parse_amount_cents("1'234.50").unwrap(), 123_450);
    }

    #[test]
    fn rejects_duplicate_column_indices_and_invalid_delimiters() {
        let mut mapping = amount_mapping();
        mapping.reference_column = Some(2);
        assert!(matches!(
            parse_csv_preview("date;libellé;montant", ';', true, &mapping),
            Err(CsvPreviewError::InvalidMapping(_))
        ));
        assert!(matches!(
            parse_csv_preview("a,b,c", '|', false, &amount_mapping()),
            Err(CsvPreviewError::InvalidDelimiter)
        ));
    }
}
