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
    /// Avertissements de prudence qui doivent être montrés avant confirmation.
    pub warnings: Vec<String>,
    /// Nombre de lignes de données lues, y compris celles rejetées.
    pub total_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BankCsvProfile {
    /// Reconnaissance à partir des en-têtes, avec repli prudent sur la forme LCL sans en-tête.
    Auto,
    /// Export CSV Crédit Agricole : colonnes à en-têtes variables selon la Caisse régionale.
    CreditAgricole,
    /// Export opérations LCL sans en-tête (métadonnées solde/compte en début et fin de fichier).
    Lcl,
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
    #[error("L'encodage du fichier CSV n'est pas reconnu (UTF-8, Windows-1252/Latin-1 ou UTF-16 attendu).")]
    InvalidEncoding,
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

/// Construit un aperçu générique CSV avec un mapping explicite. Aucun effet de bord.
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
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let records = parse_records(input, delimiter)?;
    if records.is_empty() {
        return Err(CsvPreviewError::EmptyCsv);
    }

    let headers = if has_headers {
        Some(records[0].clone())
    } else {
        None
    };
    let start_index = usize::from(has_headers);
    let data = records
        .into_iter()
        .enumerate()
        .skip(start_index)
        .map(|(index, record)| (index + 1, record))
        .collect();
    build_preview(headers, data, mapping, Vec::new())
}

/// Parse les octets bruts téléchargés depuis un espace bancaire, sans conversion préalable
/// par Excel. Le format de banque peut être fixé dans l'interface ou laissé en détection.
pub fn parse_bank_csv_bytes(
    input: &[u8],
    profile: BankCsvProfile,
) -> Result<CsvImportPreview, CsvPreviewError> {
    if input.len() > MAX_CSV_BYTES {
        return Err(CsvPreviewError::FileTooLarge {
            max_bytes: MAX_CSV_BYTES,
        });
    }
    let text = decode_bank_csv(input)?;
    let delimiter = detect_delimiter(&text, profile)?;
    let records = parse_records(&text, delimiter)?;
    if records.is_empty() {
        return Err(CsvPreviewError::EmptyCsv);
    }

    if let Some(header_index) = records
        .iter()
        .position(|record| mapping_from_headers(record).is_some())
    {
        let headers = records[header_index].clone();
        let mapping = mapping_from_headers(&headers).ok_or_else(|| {
            CsvPreviewError::InvalidMapping(
                "Les colonnes date, libellé et montant (ou débit/crédit) n'ont pas été reconnues."
                    .to_string(),
            )
        })?;
        mapping.validate()?;

        let data: Vec<(usize, Vec<String>)> = records
            .into_iter()
            .enumerate()
            .skip(header_index + 1)
            .filter_map(|(index, record)| {
                if is_non_transaction_record(&record) || mapping_from_headers(&record).is_some() {
                    None
                } else {
                    Some((index + 1, record))
                }
            })
            .collect();
        let mut preview = build_preview(Some(headers), data, &mapping, Vec::new())?;
        add_card_summary_warning(&mut preview, profile);
        return Ok(preview);
    }

    if profile == BankCsvProfile::CreditAgricole {
        return Err(CsvPreviewError::InvalidMapping(
            "En-tête Crédit Agricole non reconnu. Les exports varient selon la Caisse régionale ; vérifiez que le CSV contient les colonnes Date, Libellé et Montant ou Débit/Crédit."
                .to_string(),
        ));
    }

    parse_lcl_headerless(records)
}

fn build_preview(
    headers: Option<Vec<String>>,
    data: Vec<(usize, Vec<String>)>,
    mapping: &CsvColumnMapping,
    warnings: Vec<String>,
) -> Result<CsvImportPreview, CsvPreviewError> {
    mapping.validate()?;
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
        warnings,
        total_rows: data.len(),
    };
    for (source_row, record) in data {
        match normalize_record(&record, source_row, mapping) {
            Ok(row) => preview.rows.push(row),
            Err(error) => preview.errors.push(CsvImportRowError {
                source_row,
                message: error.to_string(),
            }),
        }
    }
    Ok(preview)
}

fn decode_bank_csv(input: &[u8]) -> Result<String, CsvPreviewError> {
    if input.starts_with(&[0xFF, 0xFE]) || input.starts_with(&[0xFE, 0xFF]) {
        let little_endian = input.starts_with(&[0xFF, 0xFE]);
        let bytes = &input[2..];
        if bytes.len() % 2 != 0 {
            return Err(CsvPreviewError::InvalidEncoding);
        }
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|pair| {
                if little_endian {
                    u16::from_le_bytes([pair[0], pair[1]])
                } else {
                    u16::from_be_bytes([pair[0], pair[1]])
                }
            })
            .collect();
        return String::from_utf16(&units).map_err(|_| CsvPreviewError::InvalidEncoding);
    }

    let input = input.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(input);
    match std::str::from_utf8(input) {
        Ok(text) => Ok(text.to_string()),
        Err(_) => Ok(decode_windows_1252(input)),
    }
}

fn decode_windows_1252(input: &[u8]) -> String {
    input
        .iter()
        .map(|byte| match *byte {
            0x80 => '€',
            0x82 => '‚',
            0x83 => 'ƒ',
            0x84 => '„',
            0x85 => '…',
            0x86 => '†',
            0x87 => '‡',
            0x88 => 'ˆ',
            0x89 => '‰',
            0x8A => 'Š',
            0x8B => '‹',
            0x8C => 'Œ',
            0x8E => 'Ž',
            0x91 => '‘',
            0x92 => '’',
            0x93 => '“',
            0x94 => '”',
            0x95 => '•',
            0x96 => '–',
            0x97 => '—',
            0x98 => '˜',
            0x99 => '™',
            0x9A => 'š',
            0x9B => '›',
            0x9C => 'œ',
            0x9E => 'ž',
            0x9F => 'Ÿ',
            other => char::from(other),
        })
        .collect()
}

fn detect_delimiter(text: &str, profile: BankCsvProfile) -> Result<char, CsvPreviewError> {
    // Les exports français du Crédit Agricole sont généralement séparés par ';'
    // avec des virgules décimales. Prioriser ce motif évite de confondre les
    // décimales des montants avec le délimiteur de colonnes.
    if let Ok(records) = parse_records(text, ';') {
        let has_recognized_header = records
            .iter()
            .any(|record| mapping_from_headers(record).is_some());
        let has_lcl_transaction = records.iter().any(|record| {
            record.len() >= 4
                && record
                    .first()
                    .and_then(|date| normalize_date(date).ok())
                    .is_some()
                && record
                    .get(1)
                    .and_then(|amount| parse_amount_cents(amount).ok())
                    .is_some()
        });
        if has_recognized_header
            || (profile != BankCsvProfile::CreditAgricole && has_lcl_transaction)
        {
            return Ok(';');
        }
    }

    let mut best: Option<(isize, char)> = None;
    for delimiter in [';', '\t', ','] {
        let Ok(records) = parse_records(text, delimiter) else {
            continue;
        };
        if records.is_empty() {
            continue;
        }

        let header_index = records
            .iter()
            .position(|record| mapping_from_headers(record).is_some());
        let score = if let Some(header_index) = header_index {
            let width = records[header_index].len();
            let mut matching_rows = 0_isize;
            let mut mismatched_rows = 0_isize;
            for record in records.iter().skip(header_index + 1) {
                if is_non_transaction_record(record) {
                    continue;
                }
                if record.len() == width {
                    matching_rows += 1;
                } else {
                    mismatched_rows += 1;
                }
            }
            10_000 + matching_rows * 100 - mismatched_rows * 100 + width as isize
        } else {
            let mut width_counts: Vec<(usize, usize)> = Vec::new();
            for record in &records {
                if let Some(entry) = width_counts
                    .iter_mut()
                    .find(|entry| entry.0 == record.len())
                {
                    entry.1 += 1;
                } else {
                    width_counts.push((record.len(), 1));
                }
            }
            let (width, count) = width_counts
                .into_iter()
                .max_by_key(|(width, count)| width.saturating_mul(*count))
                .unwrap_or((1, 0));
            if width > 1 {
                (width * count * 10 + width) as isize
            } else {
                0
            }
        };

        if best
            .map(|(best_score, _)| score > best_score)
            .unwrap_or(score > 0)
        {
            best = Some((score, delimiter));
        }
    }
    best.map(|(_, delimiter)| delimiter)
        .ok_or(CsvPreviewError::InvalidDelimiter)
}

fn normalize_header(value: &str) -> String {
    let mut result = String::new();
    for character in value.trim().to_lowercase().chars() {
        match character {
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => result.push('a'),
            'ç' => result.push('c'),
            'è' | 'é' | 'ê' | 'ë' => result.push('e'),
            'ì' | 'í' | 'î' | 'ï' => result.push('i'),
            'ñ' => result.push('n'),
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => result.push('o'),
            'ù' | 'ú' | 'û' | 'ü' => result.push('u'),
            'ý' | 'ÿ' => result.push('y'),
            'œ' => result.push_str("oe"),
            ch if ch.is_ascii_alphanumeric() => result.push(ch),
            _ => {}
        }
    }
    result
}

fn mapping_from_headers(headers: &[String]) -> Option<CsvColumnMapping> {
    let names: Vec<String> = headers.iter().map(|name| normalize_header(name)).collect();
    let pick = |predicate: &dyn Fn(&str) -> Option<usize>| {
        names
            .iter()
            .enumerate()
            .filter_map(|(index, name)| predicate(name).map(|priority| (priority, index)))
            .min_by_key(|(priority, _)| *priority)
            .map(|(_, index)| index)
    };

    let date_column = pick(&|name| {
        if name.contains("datevaleur") || name == "valeur" {
            Some(30)
        } else if name.contains("dateoperation")
            || name.contains("dateoper")
            || name.contains("dateop")
        {
            Some(0)
        } else if name == "date" {
            Some(1)
        } else if name.contains("datecomptable") || name.contains("dateecriture") {
            Some(2)
        } else if name.contains("date") {
            Some(10)
        } else {
            None
        }
    })?;

    let label_column = pick(&|name| {
        if name.contains("libelle") {
            Some(if name.contains("operation") { 0 } else { 1 })
        } else if name.contains("description") {
            Some(2)
        } else if name.contains("motif") {
            Some(3)
        } else if name == "nature" || name == "natureoperation" {
            Some(4)
        } else {
            None
        }
    })?;

    let amount_column = pick(&|name| {
        if name.contains("solde") || name.contains("debit") || name.contains("credit") {
            None
        } else if name == "montant" || name == "amount" || name.contains("montantsigne") {
            Some(0)
        } else if name.contains("montantoperation") || name.contains("amountoperation") {
            Some(1)
        } else if name.contains("montant") || name == "amount" {
            Some(2)
        } else {
            None
        }
    });
    let debit_column = pick(&|name| {
        if name.contains("debit") && !name.contains("debitdiffere") {
            Some(0)
        } else {
            None
        }
    });
    let credit_column = pick(&|name| {
        if name.contains("credit") {
            Some(0)
        } else {
            None
        }
    });

    let (amount_column, debit_column, credit_column) = match amount_column {
        Some(amount) => (Some(amount), None, None),
        None => (None, Some(debit_column?), Some(credit_column?)),
    };

    let reference_column = pick(&|name| {
        if name.contains("reference") {
            Some(0)
        } else if name == "numerooperation" || name == "numoperation" || name == "nooperation" {
            Some(1)
        } else if name.contains("numcheque") || name == "cheque" {
            Some(2)
        } else {
            None
        }
    })
    .filter(|index| {
        *index != date_column
            && *index != label_column
            && Some(*index) != amount_column
            && Some(*index) != debit_column
            && Some(*index) != credit_column
    });

    let mapping = CsvColumnMapping {
        date_column,
        label_column,
        amount_column,
        debit_column,
        credit_column,
        reference_column,
    };
    mapping.validate().ok()?;
    Some(mapping)
}

fn is_non_transaction_record(record: &[String]) -> bool {
    let joined = record
        .iter()
        .map(|cell| normalize_header(cell))
        .collect::<Vec<_>>()
        .join(" ");
    [
        "anciensolde",
        "nouveausolde",
        "soldecrediteur",
        "totaldesoperations",
        "encoursdebite",
    ]
    .iter()
    .any(|marker| joined.contains(marker))
}

fn parse_lcl_headerless(records: Vec<Vec<String>>) -> Result<CsvImportPreview, CsvPreviewError> {
    let first_transaction = records.iter().position(|record| {
        record.len() >= 4
            && record
                .first()
                .and_then(|date| normalize_date(date).ok())
                .is_some()
            && record
                .get(1)
                .and_then(|amount| parse_amount_cents(amount).ok())
                .is_some()
    });
    let Some(first_transaction) = first_transaction else {
        return Err(CsvPreviewError::InvalidMapping(
            "Format LCL sans en-tête non reconnu : une ligne d'opération doit contenir date, montant, type et libellé."
                .to_string(),
        ));
    };

    let max_width = records[first_transaction..]
        .iter()
        .filter(|record| record.len() >= 4)
        .map(Vec::len)
        .max()
        .unwrap_or(4);
    let mut data = Vec::new();
    for (index, record) in records.into_iter().enumerate().skip(first_transaction) {
        if is_non_transaction_record(&record) || record.len() < 4 {
            continue;
        }
        data.push((index + 1, record));
    }
    if data.is_empty() {
        return Err(CsvPreviewError::EmptyCsv);
    }

    let mut preview = CsvImportPreview {
        headers: None,
        rows: Vec::new(),
        errors: Vec::new(),
        warnings: vec![
            "Format LCL sans en-tête détecté : vérifiez le compte, le solde et les premières/dernières lignes dans l'aperçu avant confirmation. Les lignes de métadonnées courtes sont ignorées."
                .to_string(),
        ],
        total_rows: data.len(),
    };
    for (source_row, record) in data {
        let label_column = record.len().saturating_sub(1);
        let mapping = CsvColumnMapping {
            date_column: 0,
            label_column,
            amount_column: Some(1),
            debit_column: None,
            credit_column: None,
            reference_column: if max_width >= 5 && record.len() >= 5 {
                Some(3)
            } else {
                None
            },
        };
        match normalize_record(&record, source_row, &mapping) {
            Ok(row) => preview.rows.push(row),
            Err(error) => preview.errors.push(CsvImportRowError {
                source_row,
                message: error.to_string(),
            }),
        }
    }
    Ok(preview)
}

fn add_card_summary_warning(preview: &mut CsvImportPreview, profile: BankCsvProfile) {
    if profile == BankCsvProfile::Lcl {
        return;
    }
    let summary_rows: Vec<usize> = preview
        .rows
        .iter()
        .filter(|row| {
            let label = normalize_header(&row.label);
            label.contains("depensescarte")
                || label.contains("prelevementcarte")
                || label.contains("totalcarte")
        })
        .map(|row| row.source_row)
        .collect();
    if !summary_rows.is_empty() {
        preview.warnings.push(format!(
            "Récapitulatif de carte potentiellement détecté à la/aux ligne(s) {}. Certains exports Crédit Agricole peuvent juxtaposer un prélèvement carte total et le détail des achats : vérifiez l'aperçu pour éviter de compter les mêmes dépenses deux fois. Aucune ligne n'est supprimée automatiquement.",
            summary_rows.iter().map(usize::to_string).collect::<Vec<_>>().join(", ")
        ));
    }
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
    for format in ["%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y", "%d.%m.%Y"] {
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
    let mut value = raw.trim().to_uppercase();
    if let Some(stripped) = value.strip_prefix("EUR") {
        value = stripped.trim().to_string();
    } else if let Some(stripped) = value.strip_suffix("EUR") {
        value = stripped.trim().to_string();
    }
    if let Some(stripped) = value.strip_prefix('€') {
        value = stripped.trim().to_string();
    } else if let Some(stripped) = value.strip_suffix('€') {
        value = stripped.trim().to_string();
    }
    if value.contains("EUR") || value.contains('€') {
        return Err(());
    }
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
        let csv = "\u{feff}\"date\";libellé;montant;référence\n2026-01-01;Loyer;12,00;R-1";
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
    fn rejects_ambiguous_overprecise_and_overflowing_amounts() {
        assert!(parse_amount_cents("1,234").is_err());
        assert!(parse_amount_cents("12,34.56").is_err());
        assert!(parse_amount_cents("1.234,567").is_err());
        assert!(parse_amount_cents("9999999999999999999999999").is_err());
        assert!(parse_amount_cents("1EUR2,00").is_err());
        assert!(parse_amount_cents("1€2,00").is_err());
        assert_eq!(parse_amount_cents("€1 234,50").unwrap(), 123_450);
        assert_eq!(parse_amount_cents("1 234,50 EUR").unwrap(), 123_450);
        assert_eq!(parse_amount_cents("EUR -1 234,50").unwrap(), -123_450);
    }

    #[test]
    fn parses_credit_agricole_cp1252_headers_and_prefers_operation_date() {
        let csv = b"Date op\xe9ration;Date valeur;Libell\xe9 op\xe9ration;D\xe9bit;Cr\xe9dit;Solde\r\n01/03/2026;02/03/2026;\"Pr\xe9l\xe8vement SEPA\nLOYER\";12,50;;987,50\r\n02/03/2026;02/03/2026;VIREMENT RECU;;1 500,00;2 487,50\r\n";
        let preview = parse_bank_csv_bytes(csv, BankCsvProfile::CreditAgricole).unwrap();
        assert_eq!(preview.rows.len(), 2);
        assert_eq!(preview.errors.len(), 0);
        assert_eq!(preview.rows[0].booking_date, "2026-03-01");
        assert_eq!(preview.rows[0].amount_cents, -1_250);
        assert_eq!(preview.rows[0].label, "Prélèvement SEPA\nLOYER");
        assert_eq!(preview.rows[1].amount_cents, 150_000);
    }

    #[test]
    fn credit_agricole_card_summary_is_warned_but_not_removed() {
        let csv = b"Date;Libell\xe9;Montant\n01/03/2026;Pr\xe9l\xe8vement carte DEPENSES CARTE MARS;-150,00\n15/02/2026;SUPERMARCHE;-80,00\n";
        let preview = parse_bank_csv_bytes(csv, BankCsvProfile::CreditAgricole).unwrap();
        assert_eq!(preview.rows.len(), 2);
        assert_eq!(preview.warnings.len(), 1);
        assert!(preview.warnings[0].contains("Aucune ligne n'est supprimée"));
    }

    #[test]
    fn parses_lcl_headerless_csv_and_ignores_balance_metadata() {
        let csv = "01/03/2026;1 000,00;123456789\n02/03/2026;-12,50;CARTE;;SUPERMARCHE\n03/03/2026;125,00;VIREMENT;;LOYER RECU\n01/04/2026;1 112,50;123456789";
        let preview = parse_bank_csv_bytes(csv.as_bytes(), BankCsvProfile::Lcl).unwrap();
        assert_eq!(preview.rows.len(), 2);
        assert_eq!(preview.errors.len(), 0);
        assert_eq!(preview.rows[0].booking_date, "2026-03-02");
        assert_eq!(preview.rows[0].amount_cents, -1_250);
        assert_eq!(preview.rows[0].label, "SUPERMARCHE");
        assert_eq!(preview.rows[1].amount_cents, 12_500);
        assert_eq!(preview.rows[1].label, "LOYER RECU");
        assert_eq!(preview.warnings.len(), 1);
    }

    #[test]
    fn auto_profile_detects_comma_csv_with_dot_decimals() {
        let csv = b"Date,Libelle,Montant\n2026-03-01,LOYER,1250.00\n2026-03-02,ASSURANCE,-80.25";
        let preview = parse_bank_csv_bytes(csv, BankCsvProfile::Auto).unwrap();
        assert_eq!(preview.rows.len(), 2);
        assert_eq!(preview.rows[0].amount_cents, 125_000);
        assert_eq!(preview.rows[1].amount_cents, -8_025);
    }

    #[test]
    fn auto_profile_detects_semicolon_despite_french_decimal_commas() {
        let csv = b"Date;Libell\xe9 simplifi\xe9;Montant\n01/03/2026;LOYER;1250,00\n02/03/2026;ASSURANCE;-80,25";
        let preview = parse_bank_csv_bytes(csv, BankCsvProfile::Auto).unwrap();
        assert_eq!(preview.rows.len(), 2);
        assert_eq!(preview.rows[0].amount_cents, 125_000);
        assert_eq!(preview.rows[1].amount_cents, -8_025);
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
