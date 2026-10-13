//! Prototype de moteur de tâches persistant pour SuperPower.
//!
//! Cette crate utilise la connexion SQLCipher fournie par l'application : elle ne crée
//! pas de base annexe non chiffrée. L'installation du schéma ne modifie pas
//! PRAGMA user_version ; la migration de production doit rester orchestrée par
//! src-tauri/src/database.rs.
//!
//! Cette première tranche couvre les tâches ponctuelles, l'idempotence des demandes,
//! la file persistante, l'historique des tentatives et la reprise après expiration d'un
//! bail. Le calcul cron, les handlers métier et les commandes/UI Tauri viennent ensuite.

use std::io;

use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{
    params, types::Type, Connection, OptionalExtension, Row, Transaction, TransactionBehavior,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS task_definitions (
    id TEXT PRIMARY KEY NOT NULL,
    idempotency_key TEXT NOT NULL UNIQUE,
    module TEXT NOT NULL CHECK (length(trim(module)) > 0),
    task_type TEXT NOT NULL CHECK (length(trim(task_type)) > 0),
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    payload_json TEXT NOT NULL,
    scheduled_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled'
        CHECK (status IN ('scheduled', 'paused', 'cancelled')),
    max_attempts INTEGER NOT NULL CHECK (max_attempts BETWEEN 1 AND 100),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_task_definitions_due
    ON task_definitions(status, scheduled_at);

CREATE TABLE IF NOT EXISTS task_executions (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL REFERENCES task_definitions(id) ON DELETE RESTRICT,
    idempotency_key TEXT NOT NULL,
    scheduled_for TEXT NOT NULL,
    available_at TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued'
        CHECK (status IN ('queued', 'running', 'succeeded', 'failed', 'cancelled')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    max_attempts INTEGER NOT NULL CHECK (max_attempts BETWEEN 1 AND 100),
    lease_until TEXT NULL,
    last_error TEXT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    started_at TEXT NULL,
    finished_at TEXT NULL,
    UNIQUE(task_id, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_task_executions_claim
    ON task_executions(status, available_at, lease_until);

CREATE TABLE IF NOT EXISTS task_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    execution_id TEXT NOT NULL REFERENCES task_executions(id) ON DELETE RESTRICT,
    attempt_number INTEGER NOT NULL CHECK (attempt_number > 0),
    status TEXT NOT NULL CHECK (status IN ('running', 'succeeded', 'failed')),
    started_at TEXT NOT NULL,
    finished_at TEXT NULL,
    error_message TEXT NULL,
    UNIQUE(execution_id, attempt_number),
    CHECK (
        (status = 'running' AND finished_at IS NULL)
        OR (status <> 'running' AND finished_at IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_task_attempts_history
    ON task_attempts(execution_id, attempt_number);
"#;

#[derive(Debug, Error)]
pub enum TaskEngineError {
    #[error("Paramètre invalide : {0}")]
    InvalidInput(String),
    #[error(transparent)]
    Database(#[from] rusqlite::Error),
    #[error("Cette clé d'idempotence est déjà utilisée pour une demande différente.")]
    IdempotencyConflict,
    #[error("La tâche « {0} » n'existe pas.")]
    TaskNotFound(String),
    #[error("La tâche est en pause ou annulée ; aucune nouvelle exécution ne peut être ajoutée.")]
    TaskInactive,
    #[error("L'exécution « {0} » n'existe pas.")]
    ExecutionNotFound(String),
    #[error("Transition impossible : l'exécution n'est pas en cours.")]
    InvalidExecutionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Scheduled,
    Paused,
    Cancelled,
}

impl TaskStatus {
    fn as_db_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Paused => "paused",
            Self::Cancelled => "cancelled",
        }
    }

    fn from_db_str(value: &str) -> Result<Self, String> {
        match value {
            "scheduled" => Ok(Self::Scheduled),
            "paused" => Ok(Self::Paused),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("statut de tâche inconnu : {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl ExecutionStatus {
    fn from_db_str(value: &str) -> Result<Self, String> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("statut d'exécution inconnu : {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStatus {
    Running,
    Succeeded,
    Failed,
}

impl AttemptStatus {
    fn from_db_str(value: &str) -> Result<Self, String> {
        match value {
            "running" => Ok(Self::Running),
            "succeeded" => Ok(Self::Succeeded),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("statut de tentative inconnu : {value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTask {
    pub idempotency_key: String,
    pub module: String,
    pub task_type: String,
    pub title: String,
    pub payload: Value,
    pub scheduled_at: String,
    pub max_attempts: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub idempotency_key: String,
    pub module: String,
    pub task_type: String,
    pub title: String,
    pub payload: Value,
    pub scheduled_at: String,
    pub status: TaskStatus,
    pub max_attempts: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskExecution {
    pub id: String,
    pub task_id: String,
    pub idempotency_key: String,
    pub scheduled_for: String,
    pub available_at: String,
    pub status: ExecutionStatus,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub lease_until: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAttempt {
    pub id: String,
    pub execution_id: String,
    pub attempt_number: u32,
    pub status: AttemptStatus,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub error_message: Option<String>,
}

pub struct TaskEngine;

impl TaskEngine {
    /// Ajoute le schéma du prototype à la connexion fournie par l'appelant.
    /// La version de schéma reste la responsabilité de la migration applicative.
    pub fn install_schema(connection: &Connection) -> Result<(), TaskEngineError> {
        connection.execute_batch(SCHEMA)?;
        Ok(())
    }

    /// Crée une tâche ponctuelle. Une même clé réutilisée à l'identique retourne
    /// la tâche existante ; réutilisée pour des paramètres différents, elle échoue.
    pub fn create_task(
        connection: &Connection,
        input: &NewTask,
        now: &str,
    ) -> Result<Task, TaskEngineError> {
        let idempotency_key = required_text(&input.idempotency_key, "clé d'idempotence", 200)?;
        let module = required_text(&input.module, "module", 100)?;
        let task_type = required_text(&input.task_type, "type de tâche", 100)?;
        let title = required_text(&input.title, "titre", 200)?;
        if !(1..=100).contains(&input.max_attempts) {
            return Err(TaskEngineError::InvalidInput(
                "Le nombre de tentatives doit être compris entre 1 et 100.".to_string(),
            ));
        }

        let scheduled_at = normalize_timestamp(&input.scheduled_at, "date programmée")?;
        let now = normalize_timestamp(now, "date de création")?;
        let payload_json = input.payload.to_string();
        let id = Uuid::new_v4().to_string();

        connection.execute(
            "INSERT INTO task_definitions (
                id, idempotency_key, module, task_type, title, payload_json,
                scheduled_at, status, max_attempts, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'scheduled', ?8, ?9, ?9)
             ON CONFLICT(idempotency_key) DO NOTHING",
            params![
                id,
                idempotency_key,
                module,
                task_type,
                title,
                payload_json,
                scheduled_at,
                i64::from(input.max_attempts),
                now
            ],
        )?;

        let existing =
            find_task_by_idempotency_key(connection, &idempotency_key)?.ok_or_else(|| {
                TaskEngineError::InvalidInput(
                    "La tâche créée est introuvable après insertion.".to_string(),
                )
            })?;

        let same_request = existing.module == module
            && existing.task_type == task_type
            && existing.title == title
            && existing.payload == input.payload
            && existing.scheduled_at == scheduled_at
            && existing.max_attempts == input.max_attempts;

        if !same_request {
            return Err(TaskEngineError::IdempotencyConflict);
        }

        Ok(existing)
    }

    pub fn list_tasks(connection: &Connection) -> Result<Vec<Task>, TaskEngineError> {
        let mut statement = connection.prepare(
            "SELECT id, idempotency_key, module, task_type, title, payload_json,
                    scheduled_at, status, max_attempts, created_at, updated_at
             FROM task_definitions
             ORDER BY scheduled_at, created_at, id",
        )?;
        let rows = statement.query_map([], task_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn update_task_status(
        connection: &mut Connection,
        task_id: &str,
        status: TaskStatus,
        now: &str,
    ) -> Result<Task, TaskEngineError> {
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let task = Self::update_task_status_in_transaction(&transaction, task_id, status, now)?;
        transaction.commit()?;
        Ok(task)
    }

    /// Même mutation, mais laisse à l'appelant la responsabilité du commit afin que
    /// l'audit applicatif et le changement de statut soient atomiques.
    pub fn update_task_status_in_transaction(
        transaction: &Transaction<'_>,
        task_id: &str,
        status: TaskStatus,
        now: &str,
    ) -> Result<Task, TaskEngineError> {
        let now = normalize_timestamp(now, "date de modification")?;
        let changed = transaction.execute(
            "UPDATE task_definitions SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_db_str(), now, task_id],
        )?;
        if changed == 0 {
            return Err(TaskEngineError::TaskNotFound(task_id.to_string()));
        }

        if status == TaskStatus::Cancelled {
            transaction.execute(
                "UPDATE task_executions
                 SET status = 'cancelled', updated_at = ?1, finished_at = ?1,
                     lease_until = NULL
                 WHERE task_id = ?2 AND status = 'queued'",
                params![now, task_id],
            )?;
        }

        find_task_by_id(transaction, task_id)?
            .ok_or_else(|| TaskEngineError::TaskNotFound(task_id.to_string()))
    }

    pub fn get_task_by_id(
        connection: &Connection,
        task_id: &str,
    ) -> Result<Option<Task>, TaskEngineError> {
        Ok(find_task_by_id(connection, task_id)?)
    }

    pub fn get_task_by_idempotency_key(
        connection: &Connection,
        idempotency_key: &str,
    ) -> Result<Option<Task>, TaskEngineError> {
        Ok(find_task_by_idempotency_key(connection, idempotency_key)?)
    }

    /// Enfile une occurrence. La paire (task_id, idempotency_key) empêche
    /// les doublons lorsqu'un planificateur répète sa matérialisation.
    pub fn enqueue_execution(
        connection: &Connection,
        task_id: &str,
        idempotency_key: &str,
        scheduled_for: &str,
        now: &str,
    ) -> Result<TaskExecution, TaskEngineError> {
        let idempotency_key = required_text(idempotency_key, "clé d'exécution", 200)?;
        let scheduled_for = normalize_timestamp(scheduled_for, "date d'exécution")?;
        let now = normalize_timestamp(now, "date de création de l'exécution")?;

        let task_data: Option<(String, i64)> = connection
            .query_row(
                "SELECT status, max_attempts FROM task_definitions WHERE id = ?1",
                [task_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((status, max_attempts)) = task_data else {
            return Err(TaskEngineError::TaskNotFound(task_id.to_string()));
        };
        if status != "scheduled" {
            return Err(TaskEngineError::TaskInactive);
        }

        connection.execute(
            "INSERT INTO task_executions (
                id, task_id, idempotency_key, scheduled_for, available_at,
                status, attempt_count, max_attempts, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?4, 'queued', 0, ?5, ?6, ?6)
             ON CONFLICT(task_id, idempotency_key) DO NOTHING",
            params![
                Uuid::new_v4().to_string(),
                task_id,
                idempotency_key,
                scheduled_for,
                max_attempts,
                now
            ],
        )?;

        let execution =
            find_execution_by_key(connection, task_id, &idempotency_key)?.ok_or_else(|| {
                TaskEngineError::InvalidInput(
                    "L'exécution est introuvable après insertion.".to_string(),
                )
            })?;
        if execution.scheduled_for != scheduled_for {
            return Err(TaskEngineError::IdempotencyConflict);
        }
        Ok(execution)
    }

    /// Réserve atomiquement la prochaine exécution arrivée à échéance.
    /// Une exécution "running" dont le bail a expiré est reprise après inscription
    /// de l'interruption dans l'historique ; une tentative épuisée devient "failed".
    pub fn claim_next_due(
        connection: &mut Connection,
        now: &str,
        lease_until: &str,
    ) -> Result<Option<TaskExecution>, TaskEngineError> {
        let now = normalize_timestamp(now, "date de réservation")?;
        let lease_until = normalize_timestamp(lease_until, "expiration du bail")?;
        if lease_until <= now {
            return Err(TaskEngineError::InvalidInput(
                "L'expiration du bail doit être postérieure à la réservation.".to_string(),
            ));
        }

        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let interrupted_message =
            "Exécution interrompue ; reprise après expiration du bail.".to_string();

        let exhausted: Vec<(String, i64)> = {
            let mut statement = transaction.prepare(
                "SELECT id, attempt_count
                 FROM task_executions
                 WHERE status = 'running'
                   AND lease_until IS NOT NULL
                   AND lease_until <= ?1
                   AND attempt_count >= max_attempts",
            )?;
            let rows = statement.query_map([&now], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        for (execution_id, attempt_count) in exhausted {
            transaction.execute(
                "UPDATE task_attempts
                 SET status = 'failed', finished_at = ?1,
                     error_message = COALESCE(error_message, ?2)
                 WHERE execution_id = ?3 AND attempt_number = ?4 AND status = 'running'",
                params![now, interrupted_message, execution_id, attempt_count],
            )?;
            transaction.execute(
                "UPDATE task_executions
                 SET status = 'failed', lease_until = NULL, last_error = ?1,
                     finished_at = ?2, updated_at = ?2
                 WHERE id = ?3",
                params![interrupted_message, now, execution_id],
            )?;
        }

        let candidate: Option<(String, String, i64)> = transaction
            .query_row(
                "SELECT e.id, e.status, e.attempt_count
                 FROM task_executions e
                 JOIN task_definitions t ON t.id = e.task_id
                 WHERE t.status = 'scheduled'
                   AND (
                     (e.status = 'queued' AND e.available_at <= ?1)
                     OR
                     (e.status = 'running' AND e.lease_until IS NOT NULL
                       AND e.lease_until <= ?1 AND e.attempt_count < e.max_attempts)
                   )
                 ORDER BY e.available_at, e.created_at, e.id
                 LIMIT 1",
                [&now],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;

        let Some((execution_id, previous_status, previous_attempt_count)) = candidate else {
            transaction.commit()?;
            return Ok(None);
        };

        if previous_status == "running" {
            transaction.execute(
                "UPDATE task_attempts
                 SET status = 'failed', finished_at = ?1,
                     error_message = COALESCE(error_message, ?2)
                 WHERE execution_id = ?3 AND attempt_number = ?4 AND status = 'running'",
                params![
                    now,
                    interrupted_message,
                    execution_id,
                    previous_attempt_count
                ],
            )?;
            transaction.execute(
                "UPDATE task_executions
                 SET status = 'queued', available_at = ?1, lease_until = NULL,
                     last_error = ?2, updated_at = ?1
                 WHERE id = ?3",
                params![now, interrupted_message, execution_id],
            )?;
        }

        let attempt_number = previous_attempt_count + 1;
        transaction.execute(
            "UPDATE task_executions
             SET status = 'running', attempt_count = ?1, lease_until = ?2,
                 started_at = COALESCE(started_at, ?3), updated_at = ?3,
                 last_error = NULL, finished_at = NULL
             WHERE id = ?4",
            params![attempt_number, lease_until, now, execution_id],
        )?;
        transaction.execute(
            "INSERT INTO task_attempts (
                id, execution_id, attempt_number, status, started_at
             ) VALUES (?1, ?2, ?3, 'running', ?4)",
            params![
                Uuid::new_v4().to_string(),
                execution_id,
                attempt_number,
                now
            ],
        )?;

        let execution = find_execution_by_id(&transaction, &execution_id)?
            .ok_or_else(|| TaskEngineError::ExecutionNotFound(execution_id.clone()))?;
        transaction.commit()?;
        Ok(Some(execution))
    }

    pub fn complete_execution(
        connection: &mut Connection,
        execution_id: &str,
        now: &str,
    ) -> Result<TaskExecution, TaskEngineError> {
        let now = normalize_timestamp(now, "date de fin")?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let state: Option<(String, i64)> = transaction
            .query_row(
                "SELECT status, attempt_count FROM task_executions WHERE id = ?1",
                [execution_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let Some((status, attempt_count)) = state else {
            return Err(TaskEngineError::ExecutionNotFound(execution_id.to_string()));
        };
        if status != "running" {
            return Err(TaskEngineError::InvalidExecutionState);
        }

        let changed = transaction.execute(
            "UPDATE task_attempts
             SET status = 'succeeded', finished_at = ?1, error_message = NULL
             WHERE execution_id = ?2 AND attempt_number = ?3 AND status = 'running'",
            params![now, execution_id, attempt_count],
        )?;
        if changed != 1 {
            return Err(TaskEngineError::InvalidExecutionState);
        }
        transaction.execute(
            "UPDATE task_executions
             SET status = 'succeeded', lease_until = NULL, last_error = NULL,
                 finished_at = ?1, updated_at = ?1
             WHERE id = ?2",
            params![now, execution_id],
        )?;

        let execution = find_execution_by_id(&transaction, execution_id)?
            .ok_or_else(|| TaskEngineError::ExecutionNotFound(execution_id.to_string()))?;
        transaction.commit()?;
        Ok(execution)
    }

    /// Termine la tentative courante. Si le budget de tentatives le permet,
    /// l'exécution revient dans la file à retry_at ; sinon elle devient terminale.
    pub fn fail_execution(
        connection: &mut Connection,
        execution_id: &str,
        now: &str,
        retry_at: &str,
        error_message: &str,
    ) -> Result<TaskExecution, TaskEngineError> {
        let now = normalize_timestamp(now, "date d'échec")?;
        let retry_at = normalize_timestamp(retry_at, "date de nouvelle tentative")?;
        let error_message = required_text(error_message, "erreur d'exécution", 4_000)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let state: Option<(String, i64, i64)> = transaction
            .query_row(
                "SELECT status, attempt_count, max_attempts
                 FROM task_executions WHERE id = ?1",
                [execution_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        let Some((status, attempt_count, max_attempts)) = state else {
            return Err(TaskEngineError::ExecutionNotFound(execution_id.to_string()));
        };
        if status != "running" {
            return Err(TaskEngineError::InvalidExecutionState);
        }

        let attempt_changed = transaction.execute(
            "UPDATE task_attempts
             SET status = 'failed', finished_at = ?1, error_message = ?2
             WHERE execution_id = ?3 AND attempt_number = ?4 AND status = 'running'",
            params![now, error_message, execution_id, attempt_count],
        )?;
        if attempt_changed != 1 {
            return Err(TaskEngineError::InvalidExecutionState);
        }

        if attempt_count < max_attempts {
            transaction.execute(
                "UPDATE task_executions
                 SET status = 'queued', available_at = ?1, lease_until = NULL,
                     last_error = ?2, updated_at = ?3, finished_at = NULL
                 WHERE id = ?4",
                params![retry_at, error_message, now, execution_id],
            )?;
        } else {
            transaction.execute(
                "UPDATE task_executions
                 SET status = 'failed', lease_until = NULL, last_error = ?1,
                     updated_at = ?2, finished_at = ?2
                 WHERE id = ?3",
                params![error_message, now, execution_id],
            )?;
        }

        let execution = find_execution_by_id(&transaction, execution_id)?
            .ok_or_else(|| TaskEngineError::ExecutionNotFound(execution_id.to_string()))?;
        transaction.commit()?;
        Ok(execution)
    }

    pub fn list_executions(
        connection: &Connection,
        task_id: &str,
    ) -> Result<Vec<TaskExecution>, TaskEngineError> {
        let mut statement = connection.prepare(
            "SELECT id, task_id, idempotency_key, scheduled_for, available_at, status,
                    attempt_count, max_attempts, lease_until, last_error, created_at,
                    updated_at, started_at, finished_at
             FROM task_executions WHERE task_id = ?1
             ORDER BY scheduled_for DESC, created_at DESC, id",
        )?;
        let rows = statement.query_map([task_id], execution_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn list_attempts(
        connection: &Connection,
        execution_id: &str,
    ) -> Result<Vec<TaskAttempt>, TaskEngineError> {
        let mut statement = connection.prepare(
            "SELECT id, execution_id, attempt_number, status, started_at, finished_at,
                    error_message
             FROM task_attempts WHERE execution_id = ?1
             ORDER BY attempt_number",
        )?;
        let rows = statement.query_map([execution_id], attempt_from_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}

fn required_text(value: &str, label: &str, max_chars: usize) -> Result<String, TaskEngineError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(TaskEngineError::InvalidInput(format!(
            "Le champ « {label} » est obligatoire."
        )));
    }
    if value.chars().count() > max_chars {
        return Err(TaskEngineError::InvalidInput(format!(
            "Le champ « {label} » dépasse {max_chars} caractères."
        )));
    }
    Ok(value.to_string())
}

fn normalize_timestamp(value: &str, label: &str) -> Result<String, TaskEngineError> {
    let parsed = DateTime::parse_from_rfc3339(value).map_err(|_| {
        TaskEngineError::InvalidInput(format!(
            "Le champ « {label} » doit être une date ISO 8601 avec fuseau horaire."
        ))
    })?;
    Ok(parsed
        .with_timezone(&Utc)
        .to_rfc3339_opts(SecondsFormat::Millis, true))
}

fn conversion_error(index: usize, kind: &str, value: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        index,
        Type::Text,
        Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{kind} inconnu : {value}"),
        )),
    )
}

fn task_status_from_db(value: &str) -> Result<TaskStatus, rusqlite::Error> {
    TaskStatus::from_db_str(value)
        .map_err(|message| conversion_error(7, "statut de tâche", &message))
}

fn attempt_status_from_db(value: &str) -> Result<AttemptStatus, rusqlite::Error> {
    AttemptStatus::from_db_str(value)
        .map_err(|message| conversion_error(3, "statut de tentative", &message))
}

fn task_from_row(row: &Row<'_>) -> rusqlite::Result<Task> {
    let payload_json: String = row.get(5)?;
    let payload = serde_json::from_str(&payload_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, Type::Text, Box::new(error))
    })?;
    let status_raw: String = row.get(7)?;
    let max_attempts_raw: i64 = row.get(8)?;
    let max_attempts = u32::try_from(max_attempts_raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(8, Type::Integer, Box::new(error))
    })?;
    Ok(Task {
        id: row.get(0)?,
        idempotency_key: row.get(1)?,
        module: row.get(2)?,
        task_type: row.get(3)?,
        title: row.get(4)?,
        payload,
        scheduled_at: row.get(6)?,
        status: task_status_from_db(&status_raw)?,
        max_attempts,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn execution_from_row(row: &Row<'_>) -> rusqlite::Result<TaskExecution> {
    let status_raw: String = row.get(5)?;
    let attempt_count_raw: i64 = row.get(6)?;
    let attempt_count = u32::try_from(attempt_count_raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(6, Type::Integer, Box::new(error))
    })?;
    let max_attempts_raw: i64 = row.get(7)?;
    let max_attempts = u32::try_from(max_attempts_raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(7, Type::Integer, Box::new(error))
    })?;
    Ok(TaskExecution {
        id: row.get(0)?,
        task_id: row.get(1)?,
        idempotency_key: row.get(2)?,
        scheduled_for: row.get(3)?,
        available_at: row.get(4)?,
        status: ExecutionStatus::from_db_str(&status_raw)
            .map_err(|message| conversion_error(5, "statut d'exécution", &message))?,
        attempt_count,
        max_attempts,
        lease_until: row.get(8)?,
        last_error: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        started_at: row.get(12)?,
        finished_at: row.get(13)?,
    })
}

fn attempt_from_row(row: &Row<'_>) -> rusqlite::Result<TaskAttempt> {
    let attempt_number_raw: i64 = row.get(2)?;
    let attempt_number = u32::try_from(attempt_number_raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(2, Type::Integer, Box::new(error))
    })?;
    let status_raw: String = row.get(3)?;
    Ok(TaskAttempt {
        id: row.get(0)?,
        execution_id: row.get(1)?,
        attempt_number,
        status: attempt_status_from_db(&status_raw)?,
        started_at: row.get(4)?,
        finished_at: row.get(5)?,
        error_message: row.get(6)?,
    })
}

fn find_task_by_idempotency_key(
    connection: &Connection,
    idempotency_key: &str,
) -> Result<Option<Task>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT id, idempotency_key, module, task_type, title, payload_json,
                    scheduled_at, status, max_attempts, created_at, updated_at
             FROM task_definitions WHERE idempotency_key = ?1",
            [idempotency_key],
            task_from_row,
        )
        .optional()
}

fn find_task_by_id(
    connection: &Connection,
    task_id: &str,
) -> Result<Option<Task>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT id, idempotency_key, module, task_type, title, payload_json,
                    scheduled_at, status, max_attempts, created_at, updated_at
             FROM task_definitions WHERE id = ?1",
            [task_id],
            task_from_row,
        )
        .optional()
}

fn find_execution_by_key(
    connection: &Connection,
    task_id: &str,
    idempotency_key: &str,
) -> Result<Option<TaskExecution>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT id, task_id, idempotency_key, scheduled_for, available_at, status,
                    attempt_count, max_attempts, lease_until, last_error, created_at,
                    updated_at, started_at, finished_at
             FROM task_executions WHERE task_id = ?1 AND idempotency_key = ?2",
            params![task_id, idempotency_key],
            execution_from_row,
        )
        .optional()
}

fn find_execution_by_id(
    connection: &Connection,
    execution_id: &str,
) -> Result<Option<TaskExecution>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT id, task_id, idempotency_key, scheduled_for, available_at, status,
                    attempt_count, max_attempts, lease_until, last_error, created_at,
                    updated_at, started_at, finished_at
             FROM task_executions WHERE id = ?1",
            [execution_id],
            execution_from_row,
        )
        .optional()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OpenFlags;
    use std::path::PathBuf;

    const KEY_HEX: &str = "2727272727272727272727272727272727272727272727272727272727272727";
    const BEFORE_DUE: &str = "2026-10-13T09:00:00Z";
    const DUE: &str = "2026-10-13T10:00:00Z";

    struct TemporaryDatabase {
        path: PathBuf,
    }

    impl TemporaryDatabase {
        fn new() -> Self {
            Self {
                path: std::env::temp_dir()
                    .join(format!("superpower-task-engine-{}.db", Uuid::new_v4())),
            }
        }

        fn connect(&self) -> Connection {
            let connection = Connection::open_with_flags(
                &self.path,
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE
                    | OpenFlags::SQLITE_OPEN_NO_MUTEX,
            )
            .expect("open encrypted test database");
            connection
                .execute_batch(&format!(
                    "PRAGMA key = \"x'{KEY_HEX}'\";
                     PRAGMA cipher_memory_security = ON;
                     PRAGMA foreign_keys = ON;"
                ))
                .expect("configure SQLCipher");
            let cipher_version: Option<String> = connection
                .query_row("PRAGMA cipher_version", [], |row| row.get(0))
                .optional()
                .expect("read SQLCipher version");
            assert!(
                cipher_version.is_some_and(|value| !value.trim().is_empty()),
                "tests must use SQLCipher, never plain SQLite"
            );
            connection
        }
    }

    impl Drop for TemporaryDatabase {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
            let _ = std::fs::remove_file(self.path.with_extension("db-wal"));
            let _ = std::fs::remove_file(self.path.with_extension("db-shm"));
        }
    }

    fn new_task(key: &str, max_attempts: u32) -> NewTask {
        NewTask {
            idempotency_key: key.to_string(),
            module: "documents".to_string(),
            task_type: "classify_document".to_string(),
            title: "Classer un document".to_string(),
            payload: serde_json::json!({"documentId": "document-test"}),
            scheduled_at: DUE.to_string(),
            max_attempts,
        }
    }

    fn init(connection: &Connection) {
        TaskEngine::install_schema(connection).expect("install schema");
    }

    #[test]
    fn task_creation_is_idempotent_and_rejects_key_reuse_with_different_payload() {
        let database = TemporaryDatabase::new();
        let connection = database.connect();
        init(&connection);

        let input = new_task("create-document-test-1", 3);
        let first = TaskEngine::create_task(&connection, &input, BEFORE_DUE).unwrap();
        let repeated = TaskEngine::create_task(&connection, &input, BEFORE_DUE).unwrap();
        assert_eq!(first.id, repeated.id);
        assert_eq!(TaskEngine::list_tasks(&connection).unwrap().len(), 1);

        let mut different_request = input;
        different_request.title = "Autre titre".to_string();
        assert!(matches!(
            TaskEngine::create_task(&connection, &different_request, BEFORE_DUE),
            Err(TaskEngineError::IdempotencyConflict)
        ));
        assert_eq!(TaskEngine::list_tasks(&connection).unwrap().len(), 1);
    }

    #[test]
    fn execution_is_deduplicated_and_recovered_after_database_reopen() {
        let database = TemporaryDatabase::new();
        let connection = database.connect();
        init(&connection);
        let task = TaskEngine::create_task(
            &connection,
            &new_task("create-document-test-2", 2),
            BEFORE_DUE,
        )
        .unwrap();
        let first = TaskEngine::enqueue_execution(
            &connection,
            &task.id,
            "document-test-2026-10-13",
            DUE,
            BEFORE_DUE,
        )
        .unwrap();
        let repeated = TaskEngine::enqueue_execution(
            &connection,
            &task.id,
            "document-test-2026-10-13",
            DUE,
            BEFORE_DUE,
        )
        .unwrap();
        assert_eq!(first.id, repeated.id);
        assert_eq!(
            TaskEngine::list_executions(&connection, &task.id)
                .unwrap()
                .len(),
            1
        );
        drop(connection);

        // Reopen with the same key to prove that the queued execution survived restart.
        let mut connection = database.connect();
        let claimed = TaskEngine::claim_next_due(&mut connection, DUE, "2026-10-13T10:01:00Z")
            .unwrap()
            .expect("first claim");
        assert_eq!(claimed.attempt_count, 1);
        assert_eq!(claimed.status, ExecutionStatus::Running);
        let execution_id = claimed.id.clone();
        drop(connection);

        // Simulate a process crash: the lease expires without completing the attempt.
        let mut connection = database.connect();
        let recovered = TaskEngine::claim_next_due(
            &mut connection,
            "2026-10-13T10:02:00Z",
            "2026-10-13T10:03:00Z",
        )
        .unwrap()
        .expect("recovered claim");
        assert_eq!(recovered.id, execution_id);
        assert_eq!(recovered.attempt_count, 2);
        assert_eq!(recovered.status, ExecutionStatus::Running);

        let attempts = TaskEngine::list_attempts(&connection, &execution_id).unwrap();
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0].status, AttemptStatus::Failed);
        assert!(attempts[0]
            .error_message
            .as_deref()
            .is_some_and(|message| message.contains("expiration du bail")));
        assert_eq!(attempts[1].status, AttemptStatus::Running);

        let failed = TaskEngine::fail_execution(
            &mut connection,
            &execution_id,
            "2026-10-13T10:02:30Z",
            "2026-10-13T10:03:00Z",
            "Erreur de test",
        )
        .unwrap();
        assert_eq!(failed.status, ExecutionStatus::Failed);
        assert_eq!(failed.attempt_count, 2);
        let attempts = TaskEngine::list_attempts(&connection, &execution_id).unwrap();
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[1].status, AttemptStatus::Failed);
        assert_eq!(attempts[1].error_message.as_deref(), Some("Erreur de test"));
    }

    #[test]
    fn cancellation_is_transactional_and_cancels_queued_executions() {
        let database = TemporaryDatabase::new();
        let mut connection = database.connect();
        init(&connection);
        let task = TaskEngine::create_task(
            &connection,
            &new_task("cancel-task-test", 2),
            BEFORE_DUE,
        )
        .unwrap();
        let execution = TaskEngine::enqueue_execution(
            &connection,
            &task.id,
            "cancel-task-occurrence",
            DUE,
            BEFORE_DUE,
        )
        .unwrap();
        assert_eq!(execution.status, ExecutionStatus::Queued);

        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let cancelled = TaskEngine::update_task_status_in_transaction(
            &transaction,
            &task.id,
            TaskStatus::Cancelled,
            "2026-10-13T09:30:00Z",
        )
        .unwrap();
        assert_eq!(cancelled.status, TaskStatus::Cancelled);
        transaction.commit().unwrap();

        let persisted = TaskEngine::list_executions(&connection, &task.id).unwrap();
        assert_eq!(persisted.len(), 1);
        assert_eq!(persisted[0].status, ExecutionStatus::Cancelled);
        assert_eq!(
            persisted[0].finished_at.as_deref(),
            Some("2026-10-13T09:30:00.000Z")
        );
        assert_eq!(
            TaskEngine::get_task_by_id(&connection, &task.id)
                .unwrap()
                .unwrap()
                .status,
            TaskStatus::Cancelled
        );
    }

    #[test]
    fn successful_execution_is_terminal_and_keeps_attempt_history() {
        let database = TemporaryDatabase::new();
        let connection = database.connect();
        init(&connection);
        let task = TaskEngine::create_task(
            &connection,
            &new_task("create-document-test-3", 1),
            BEFORE_DUE,
        )
        .unwrap();
        let execution = TaskEngine::enqueue_execution(
            &connection,
            &task.id,
            "single-run-test",
            DUE,
            BEFORE_DUE,
        )
        .unwrap();

        let mut connection = connection;
        let claimed = TaskEngine::claim_next_due(&mut connection, DUE, "2026-10-13T10:01:00Z")
            .unwrap()
            .unwrap();
        let completed =
            TaskEngine::complete_execution(&mut connection, &execution.id, "2026-10-13T10:00:30Z")
                .unwrap();
        assert_eq!(completed.status, ExecutionStatus::Succeeded);
        assert!(TaskEngine::claim_next_due(
            &mut connection,
            "2026-10-13T10:05:00Z",
            "2026-10-13T10:06:00Z",
        )
        .unwrap()
        .is_none());
        let attempts = TaskEngine::list_attempts(&connection, &claimed.id).unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].status, AttemptStatus::Succeeded);
        assert_eq!(
            attempts[0].finished_at.as_deref(),
            Some("2026-10-13T10:00:30.000Z")
        );
    }
}
