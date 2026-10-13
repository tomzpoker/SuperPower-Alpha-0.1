use std::{fs, path::PathBuf, sync::Mutex};

use argon2::{Algorithm, Argon2, Params, Version};
use rand::{rngs::OsRng, RngCore};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use superpower_core::accounting::starter_accounts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseStatus {
    NeedsSetup,
    Locked,
    Ready,
}

pub struct AppState {
    pub database_path: PathBuf,
    salt_path: PathBuf,
    key_hex: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(database_path: PathBuf) -> Self {
        let salt_path = database_path.with_extension("salt");
        Self {
            database_path,
            salt_path,
            key_hex: Mutex::new(None),
        }
    }

    pub fn status(&self) -> DatabaseStatus {
        if !self.database_path.exists() {
            return DatabaseStatus::NeedsSetup;
        }
        let key = self.key_hex.lock().ok().and_then(|guard| guard.clone());
        match key {
            None => DatabaseStatus::Locked,
            Some(_) => match self.connection() {
                Ok(_) => DatabaseStatus::Ready,
                Err(_) => {
                    if let Ok(mut guard) = self.key_hex.lock() {
                        *guard = None;
                    }
                    DatabaseStatus::Locked
                }
            },
        }
    }

    pub fn initialize(&self, passphrase: &str) -> Result<(), String> {
        validate_passphrase(passphrase)?;
        if self.database_path.exists() {
            return Err("Une base existe déjà : utilisez le déverrouillage.".to_string());
        }
        let parent = self
            .database_path
            .parent()
            .ok_or("Chemin de base invalide.")?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Création du dossier impossible : {error}"))?;
        let mut salt = [0_u8; 16];
        OsRng.fill_bytes(&mut salt);
        fs::write(&self.salt_path, salt)
            .map_err(|error| format!("Écriture du sel impossible : {error}"))?;
        let key = derive_key(passphrase, &salt)?;
        self.set_key(Some(key));

        let result = (|| {
            let mut connection = self.connection()?;
            create_schema(&mut connection)
                .map_err(|error| format!("Initialisation du schéma impossible : {error}"))?;
            migrate_schema(&mut connection)?;
            Ok(())
        })();
        if result.is_err() {
            self.set_key(None);
            let _ = fs::remove_file(&self.database_path);
            let _ = fs::remove_file(self.database_path.with_extension("db-wal"));
            let _ = fs::remove_file(self.database_path.with_extension("db-shm"));
            let _ = fs::remove_file(&self.salt_path);
        }
        result
    }

    pub fn unlock(&self, passphrase: &str) -> Result<(), String> {
        if !self.database_path.exists() {
            return Err(
                "Aucune base à déverrouiller : initialisez d'abord SuperPower.".to_string(),
            );
        }
        let salt = fs::read(&self.salt_path).map_err(|error| format!("Fichier de sel manquant ou illisible : {error}. Restaurez le fichier .salt de la sauvegarde.") )?;
        if salt.len() != 16 {
            return Err(
                "Le fichier de sel est invalide. Restaurez la sauvegarde complète.".to_string(),
            );
        }
        let key = derive_key(passphrase, &salt)?;
        let mut connection = open_connection(&self.database_path, &key)?;
        let table_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table'",
                [],
                |row| row.get(0),
            )
            .map_err(|_| "Phrase secrète incorrecte ou base chiffrée endommagée.".to_string())?;
        if table_count == 0 {
            return Err("Base chiffrée valide mais sans schéma applicatif.".to_string());
        }
        migrate_schema(&mut connection)?;
        self.set_key(Some(key));
        Ok(())
    }

    pub fn lock(&self) {
        self.set_key(None);
    }

    pub fn connection(&self) -> Result<Connection, String> {
        let key = self
            .key_hex
            .lock()
            .map_err(|_| "État de sécurité indisponible.".to_string())?
            .clone()
            .ok_or_else(|| "La base est verrouillée.".to_string())?;
        open_connection(&self.database_path, &key)
    }

    fn set_key(&self, key: Option<String>) {
        if let Ok(mut guard) = self.key_hex.lock() {
            *guard = key;
        }
    }
}

/// Migrations du schéma SQLite/SQLCipher. Une version inconnue n'est jamais rétrogradée.
fn migrate_schema(connection: &mut Connection) -> Result<(), String> {
    let mut version = connection
        .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
        .map_err(|error| format!("Lecture de version de base impossible : {error}"))?;
    if version > 3 {
        return Err(format!(
            "Cette base est en version {version}, plus récente que celle prise en charge."
        ));
    }
    if version < 1 {
        return Err(format!("Version de schéma {version} inconnue ; aucune migration automatique n'a été appliquée."));
    }
    if version == 1 {
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Démarrage de migration v1 vers v2 impossible : {error}"))?;
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS flux_previsionnels (
               id TEXT PRIMARY KEY NOT NULL,
               societe_id TEXT NOT NULL REFERENCES societes(id) ON DELETE RESTRICT,
               libelle TEXT NOT NULL CHECK (length(trim(libelle)) > 0 AND length(libelle) <= 160),
               sens TEXT NOT NULL CHECK (sens IN ('encaissement', 'decaissement')),
               categorie TEXT NOT NULL CHECK (categorie IN ('loyer', 'charge_courante', 'travaux', 'impot_taxe', 'emprunt', 'apport_associe', 'autre')),
               date_prevue TEXT NOT NULL,
               montant_prevu_cents INTEGER NOT NULL CHECK (montant_prevu_cents > 0),
               tiers TEXT NULL,
               reference TEXT NULL,
               statut TEXT NOT NULL DEFAULT 'actif' CHECK (statut IN ('actif', 'annule')),
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_flux_societe_date ON flux_previsionnels(societe_id, date_prevue);
             CREATE TABLE IF NOT EXISTS flux_realisations (
               id TEXT PRIMARY KEY NOT NULL,
               flux_id TEXT NOT NULL REFERENCES flux_previsionnels(id) ON DELETE RESTRICT,
               date_realisation TEXT NOT NULL,
               montant_cents INTEGER NOT NULL CHECK (montant_cents > 0),
               reference TEXT NULL,
               note TEXT NULL,
               created_at TEXT NOT NULL,
               annulee_at TEXT NULL,
               motif_annulation TEXT NULL,
               CHECK ((annulee_at IS NULL AND motif_annulation IS NULL) OR (annulee_at IS NOT NULL AND motif_annulation IS NOT NULL))
             );
             CREATE INDEX IF NOT EXISTS idx_flux_realisations_flux_date ON flux_realisations(flux_id, date_realisation);
             PRAGMA user_version = 2;"
        ).map_err(|error| format!("Migration v1 vers v2 impossible : {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Validation de migration v1 vers v2 impossible : {error}"))?;
        version = 2;
    }
    if version == 2 {
        let transaction = connection
            .transaction()
            .map_err(|error| format!("Démarrage de migration v2 vers v3 impossible : {error}"))?;
        transaction.execute_batch(
            "CREATE TABLE IF NOT EXISTS flux_recurrences (
               id TEXT PRIMARY KEY NOT NULL,
               societe_id TEXT NOT NULL REFERENCES societes(id) ON DELETE RESTRICT,
               frequence TEXT NOT NULL CHECK (frequence IN ('mensuel', 'trimestriel', 'annuel')),
               date_debut TEXT NOT NULL,
               date_fin TEXT NULL,
               actif INTEGER NOT NULL DEFAULT 1 CHECK (actif IN (0, 1)),
               libelle TEXT NOT NULL CHECK (length(trim(libelle)) > 0 AND length(libelle) <= 160),
               sens TEXT NOT NULL CHECK (sens IN ('encaissement', 'decaissement')),
               categorie TEXT NOT NULL CHECK (categorie IN ('loyer', 'charge_courante', 'travaux', 'impot_taxe', 'emprunt', 'apport_associe', 'autre')),
               montant_prevu_cents INTEGER NOT NULL CHECK (montant_prevu_cents > 0),
               tiers TEXT NULL,
               reference TEXT NULL,
               created_at TEXT NOT NULL,
               updated_at TEXT NOT NULL,
               CHECK (date_fin IS NULL OR date_fin >= date_debut)
             );
             ALTER TABLE flux_previsionnels
               ADD COLUMN recurrence_id TEXT NULL REFERENCES flux_recurrences(id) ON DELETE RESTRICT;
             CREATE INDEX IF NOT EXISTS idx_flux_recurrence_id ON flux_previsionnels(recurrence_id);
             CREATE UNIQUE INDEX IF NOT EXISTS idx_flux_recurrence_date_unique
               ON flux_previsionnels(recurrence_id, date_prevue)
               WHERE recurrence_id IS NOT NULL;
             PRAGMA user_version = 3;"
        )
        .map_err(|error| format!("Migration v2 vers v3 impossible : {error}"))?;
        transaction
            .commit()
            .map_err(|error| format!("Validation de migration v2 vers v3 impossible : {error}"))?;
        version = 3;
    }
    if version != 3 {
        return Err(format!(
            "La version de schéma {version} n'est pas prise en charge."
        ));
    }
    Ok(())
}

fn validate_passphrase(passphrase: &str) -> Result<(), String> {
    if passphrase.chars().count() < 12 {
        return Err("Choisissez une phrase secrète d'au moins 12 caractères.".to_string());
    }
    if passphrase.trim().is_empty() {
        return Err("La phrase secrète est obligatoire.".to_string());
    }
    Ok(())
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<String, String> {
    let params = Params::new(19_456, 2, 1, Some(32))
        .map_err(|error| format!("Paramètres Argon2 invalides : {error}"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0_u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|error| format!("Dérivation de clé impossible : {error}"))?;
    Ok(hex::encode(key))
}

fn open_connection(path: &PathBuf, key_hex: &str) -> Result<Connection, String> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| format!("Ouverture de la base impossible : {error}"))?;
    // La valeur est composée uniquement d'hexadécimal issu de la clé Argon2id.
    connection
        .execute_batch(&format!(
            "PRAGMA key = \"x'{key_hex}'\"; PRAGMA cipher_memory_security = ON;"
        ))
        .map_err(|error| format!("Activation du chiffrement SQLCipher impossible : {error}"))?;
    let cipher_version = connection
        .query_row("PRAGMA cipher_version", [], |row| row.get::<_, String>(0))
        .optional()
        .map_err(|error| format!("Vérification SQLCipher impossible : {error}"))?;
    if cipher_version.as_deref().map_or(true, str::is_empty) {
        return Err("SQLCipher n'est pas disponible dans ce binaire ; aucune base non chiffrée ne sera créée.".to_string());
    }
    // Cette lecture échoue si la clé ne permet pas d'ouvrir la base chiffrée.
    connection
        .query_row("SELECT count(*) FROM sqlite_master", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|_| "Phrase secrète incorrecte ou base chiffrée endommagée.".to_string())?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")
        .map_err(|error| format!("Configuration SQLite impossible : {error}"))?;
    Ok(connection)
}

fn create_schema(connection: &mut Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        "PRAGMA journal_mode = WAL;
         CREATE TABLE IF NOT EXISTS societes (
           id TEXT PRIMARY KEY NOT NULL,
           nom TEXT NOT NULL,
           forme_juridique TEXT NOT NULL,
           regime_fiscal TEXT NOT NULL,
           regime_tva TEXT NOT NULL,
           exigibilite_tva TEXT NOT NULL,
           periodicite_tva TEXT NOT NULL,
           siret TEXT NULL,
           fin_exercice TEXT NOT NULL,
           created_at TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS accounts (
           code TEXT PRIMARY KEY NOT NULL,
           label TEXT NOT NULL,
           account_class INTEGER NOT NULL,
           is_seed INTEGER NOT NULL DEFAULT 1
         );
         CREATE TABLE IF NOT EXISTS ecritures (
           id TEXT PRIMARY KEY NOT NULL,
           societe_id TEXT NOT NULL REFERENCES societes(id),
           date_ecriture TEXT NOT NULL,
           libelle TEXT NOT NULL,
           reference TEXT NULL,
           statut TEXT NOT NULL CHECK (statut IN ('validated')),
           total_cents INTEGER NOT NULL CHECK (total_cents > 0),
           created_at TEXT NOT NULL
         );
         CREATE INDEX IF NOT EXISTS idx_ecritures_societe_date ON ecritures(societe_id, date_ecriture);
         CREATE TABLE IF NOT EXISTS lignes_ecriture (
           id TEXT PRIMARY KEY NOT NULL,
           ecriture_id TEXT NOT NULL REFERENCES ecritures(id) ON DELETE RESTRICT,
           ordre INTEGER NOT NULL,
           compte_code TEXT NOT NULL REFERENCES accounts(code),
           compte_libelle TEXT NOT NULL,
           libelle TEXT NULL,
           debit_cents INTEGER NOT NULL DEFAULT 0 CHECK (debit_cents >= 0),
           credit_cents INTEGER NOT NULL DEFAULT 0 CHECK (credit_cents >= 0),
           CHECK (NOT (debit_cents > 0 AND credit_cents > 0)),
           CHECK (debit_cents > 0 OR credit_cents > 0),
           UNIQUE(ecriture_id, ordre)
         );
         CREATE INDEX IF NOT EXISTS idx_lignes_compte ON lignes_ecriture(compte_code);
         CREATE TABLE IF NOT EXISTS journal_audit (
           id TEXT PRIMARY KEY NOT NULL,
           action TEXT NOT NULL,
           type_entite TEXT NOT NULL,
           entite_id TEXT NOT NULL,
           horodatage TEXT NOT NULL,
           details_json TEXT NOT NULL
         );
         PRAGMA user_version = 1;"
    )?;

    let transaction = connection.transaction()?;
    for account in starter_accounts() {
        transaction.execute(
            "INSERT OR IGNORE INTO accounts(code, label, account_class, is_seed) VALUES (?1, ?2, ?3, 1)",
            rusqlite::params![account.code, account.label, i64::from(account.account_class)],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

#[cfg(test)]
mod migration_tests {
    use super::*;

    #[test]
    fn migrates_v1_to_v3_preserves_existing_data_and_is_idempotent() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();
        create_schema(&mut connection).unwrap();

        connection
            .execute(
                "INSERT INTO societes (
                    id, nom, forme_juridique, regime_fiscal, regime_tva,
                    exigibilite_tva, periodicite_tva, siret, fin_exercice, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    "soc-legacy",
                    "SCI Héritage",
                    "sci",
                    "ir",
                    "non_renseigne",
                    "non_definie",
                    "non_definie",
                    Option::<String>::None,
                    "2026-12-31",
                    "2026-01-01T09:00:00Z"
                ],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO ecritures (
                    id, societe_id, date_ecriture, libelle, reference, statut, total_cents, created_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, 'validated', ?6, ?7)",
                rusqlite::params![
                    "entry-legacy",
                    "soc-legacy",
                    "2026-01-15",
                    "Loyer de janvier",
                    Some("FAC-2026-001"),
                    12_345_i64,
                    "2026-01-15T10:00:00Z"
                ],
            )
            .unwrap();

        let accounts: Vec<(String, String)> = {
            let mut statement = connection
                .prepare("SELECT code, label FROM accounts ORDER BY code LIMIT 2")
                .unwrap();
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap()
        };
        assert_eq!(
            accounts.len(),
            2,
            "le schéma v1 doit fournir des comptes de base"
        );
        for (index, (code, label)) in accounts.iter().enumerate() {
            let (debit, credit) = if index == 0 {
                (12_345_i64, 0_i64)
            } else {
                (0_i64, 12_345_i64)
            };
            connection
                .execute(
                    "INSERT INTO lignes_ecriture (
                        id, ecriture_id, ordre, compte_code, compte_libelle, libelle, debit_cents, credit_cents
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![
                        format!("line-legacy-{index}"),
                        "entry-legacy",
                        index as i64,
                        code,
                        label,
                        Option::<String>::None,
                        debit,
                        credit
                    ],
                )
                .unwrap();
        }
        connection
            .execute(
                "INSERT INTO journal_audit (
                    id, action, type_entite, entite_id, horodatage, details_json
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    "audit-legacy",
                    "validate",
                    "ecriture",
                    "entry-legacy",
                    "2026-01-15T10:00:01Z",
                    "{\"totalCents\":12345}"
                ],
            )
            .unwrap();

        migrate_schema(&mut connection).unwrap();
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 3);
        for table in ["flux_previsionnels", "flux_realisations", "flux_recurrences"] {
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "la table {table} doit être créée");
        }

        let recurrence_column_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('flux_previsionnels') WHERE name = 'recurrence_id'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(recurrence_column_count, 1, "le lien facultatif vers une série doit être ajouté");

        let company_name: String = connection
            .query_row(
                "SELECT nom FROM societes WHERE id = 'soc-legacy'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(company_name, "SCI Héritage");
        let (entry_label, entry_amount): (String, i64) = connection
            .query_row(
                "SELECT libelle, total_cents FROM ecritures WHERE id = 'entry-legacy'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(entry_label, "Loyer de janvier");
        assert_eq!(entry_amount, 12_345);
        let line_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM lignes_ecriture WHERE ecriture_id = 'entry-legacy'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(line_count, 2);
        let audit_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM journal_audit WHERE id = 'audit-legacy'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(audit_count, 1);

        migrate_schema(&mut connection).unwrap();
        let final_version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(final_version, 3);
        let final_entry_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM ecritures WHERE id = 'entry-legacy'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            final_entry_count, 1,
            "la migration répétée ne doit rien dupliquer"
        );
    }

    #[test]
    fn refuses_a_database_from_an_unknown_future_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("PRAGMA user_version = 99;")
            .unwrap();
        assert!(migrate_schema(&mut connection)
            .unwrap_err()
            .contains("plus récente"));
    }
}
