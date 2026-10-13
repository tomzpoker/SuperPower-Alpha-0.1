//! Exécuteur synchrone d'une tâche pour le moteur persistant.
//!
//! Le worker est piloté par l'appelant : cette tranche ne démarre pas encore de thread
//! de fond Tauri. Les handlers sont exécutés hors transaction SQL.

use std::{
    collections::HashMap,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{Arc, RwLock},
};

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use rusqlite::Connection;
use thiserror::Error;

use crate::{
    CronSchedule, ExecutionStatus, Task, TaskEngine, TaskEngineError, TaskExecution,
    TaskRecurrenceFrequency, TaskStatus,
};

type Handler = dyn Fn(&Task, &TaskExecution) -> Result<(), String> + Send + Sync + 'static;

/// Maximum d'occurrences cron rattrapées par tâche et par appel du worker.
const MAX_CRON_CATCH_UP_PER_TASK: usize = 1_000;
type HandlerKey = (String, String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HandlerRegistryError {
    #[error("Le module du gestionnaire doit contenir entre 1 et 100 caractères.")]
    InvalidModule,
    #[error("Le type de tâche doit contenir entre 1 et 100 caractères.")]
    InvalidTaskType,
    #[error("Un gestionnaire existe déjà pour « {module}/{task_type} ».")]
    Duplicate { module: String, task_type: String },
    #[error("Le registre des gestionnaires est indisponible : son verrou est empoisonné.")]
    Poisoned,
}

/// Registre thread-safe indexé par le couple module/type de tâche.
#[derive(Clone, Default)]
pub struct TaskHandlerRegistry {
    handlers: Arc<RwLock<HashMap<HandlerKey, Arc<Handler>>>>,
}

impl TaskHandlerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_handler<F>(
        &self,
        module: &str,
        task_type: &str,
        handler: F,
    ) -> Result<(), HandlerRegistryError>
    where
        F: Fn(&Task, &TaskExecution) -> Result<(), String> + Send + Sync + 'static,
    {
        let module = normalize_key(module).ok_or(HandlerRegistryError::InvalidModule)?;
        let task_type = normalize_key(task_type).ok_or(HandlerRegistryError::InvalidTaskType)?;
        let key = (module.clone(), task_type.clone());
        let mut handlers = self
            .handlers
            .write()
            .map_err(|_| HandlerRegistryError::Poisoned)?;
        if handlers.contains_key(&key) {
            return Err(HandlerRegistryError::Duplicate { module, task_type });
        }
        handlers.insert(key, Arc::new(handler));
        Ok(())
    }

    /// Indique si le couple module/type dispose d'un handler enregistré.
    pub fn has_handler(&self, module: &str, task_type: &str) -> Result<bool, HandlerRegistryError> {
        let module = normalize_key(module).ok_or(HandlerRegistryError::InvalidModule)?;
        let task_type = normalize_key(task_type).ok_or(HandlerRegistryError::InvalidTaskType)?;
        let handlers = self
            .handlers
            .read()
            .map_err(|_| HandlerRegistryError::Poisoned)?;
        Ok(handlers.contains_key(&(module, task_type)))
    }

    fn invoke(&self, task: &Task, execution: &TaskExecution) -> Result<(), String> {
        let key = (task.module.clone(), task.task_type.clone());
        let handler = self
            .handlers
            .read()
            .map_err(|_| "Le registre des gestionnaires est indisponible.".to_string())?
            .get(&key)
            .cloned();

        let Some(handler) = handler else {
            return Err(format!(
                "Aucun gestionnaire n'est enregistré pour « {}/{} ».",
                task.module, task.task_type
            ));
        };

        match catch_unwind(AssertUnwindSafe(|| handler(task, execution))) {
            Ok(result) => result,
            Err(_) => Err("Le gestionnaire a paniqué pendant l'exécution.".to_string()),
        }
    }
}

fn normalize_key(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 100 {
        return None;
    }
    Some(value.to_string())
}

/// Durées du bail et de nouvelle tentative. Les deux délais doivent être positifs.
#[derive(Debug, Clone, Copy)]
pub struct TaskWorkerConfig {
    pub lease_duration: Duration,
    pub retry_delay: Duration,
}

impl Default for TaskWorkerConfig {
    fn default() -> Self {
        Self {
            lease_duration: Duration::seconds(60),
            retry_delay: Duration::seconds(30),
        }
    }
}

impl TaskWorkerConfig {
    fn validate(self) -> Result<(), TaskWorkerError> {
        if self.lease_duration.num_milliseconds() < 1 {
            return Err(TaskWorkerError::InvalidConfig(
                "La durée du bail doit être d'au moins une milliseconde.",
            ));
        }
        if self.retry_delay.num_milliseconds() < 1 {
            return Err(TaskWorkerError::InvalidConfig(
                "Le délai de nouvelle tentative doit être d'au moins une milliseconde.",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerOutcome {
    Idle,
    Succeeded {
        execution_id: String,
        attempt_number: u32,
    },
    RetryScheduled {
        execution_id: String,
        attempt_number: u32,
        retry_at: String,
        error: String,
    },
    Failed {
        execution_id: String,
        attempt_number: u32,
        error: String,
    },
}

#[derive(Debug, Error)]
pub enum TaskWorkerError {
    #[error("{0}")]
    InvalidConfig(&'static str),
    #[error(transparent)]
    Engine(#[from] TaskEngineError),
    #[error(transparent)]
    Registry(#[from] HandlerRegistryError),
}

/// Exécuteur d'une seule tâche. L'appelant choisit quand appeler run_once.
pub struct TaskWorker;

impl TaskWorker {
    /// Mode explicite : un handler absent est enregistré comme échec. À réserver aux
    /// diagnostics et appels manuels ; pas pour une boucle de fond.
    pub fn run_once(
        connection: &mut Connection,
        registry: &TaskHandlerRegistry,
        now: DateTime<Utc>,
        config: TaskWorkerConfig,
    ) -> Result<WorkerOutcome, TaskWorkerError> {
        Self::run_once_internal(connection, registry, now, config, false)
    }

    /// Mode sûr pour un futur worker de fond : seules les tâches dont le handler est
    /// enregistré sont matérialisées et réservées ; les autres restent intactes.
    pub fn run_once_for_registered_handlers(
        connection: &mut Connection,
        registry: &TaskHandlerRegistry,
        now: DateTime<Utc>,
        config: TaskWorkerConfig,
    ) -> Result<WorkerOutcome, TaskWorkerError> {
        Self::run_once_internal(connection, registry, now, config, true)
    }

    fn run_once_internal(
        connection: &mut Connection,
        registry: &TaskHandlerRegistry,
        now: DateTime<Utc>,
        config: TaskWorkerConfig,
        registered_handlers_only: bool,
    ) -> Result<WorkerOutcome, TaskWorkerError> {
        config.validate()?;
        let now_text = timestamp(&now);
        let mut eligible_task_ids = Vec::new();
        let lease_until = timestamp(&(now.clone() + config.lease_duration));

        // Les occurrences récurrentes utilisent une clé stable liée à l'instant prévu.
        // Le plafond borne le rattrapage d'un coffre resté fermé longtemps ; un appel
        // ultérieur reprend où le précédent s'est arrêté.
        for task in TaskEngine::list_tasks(connection)? {
            if task.status != TaskStatus::Scheduled {
                continue;
            }

            if registered_handlers_only && !registry.has_handler(&task.module, &task.task_type)? {
                continue;
            }
            if registered_handlers_only {
                eligible_task_ids.push(task.id.clone());
            }

            if task.recurrence_frequency.is_recurring() {
                if task.scheduled_at > now_text {
                    continue;
                }
                let cron_schedule =
                    if task.recurrence_frequency == TaskRecurrenceFrequency::CustomCron {
                        let expression = task.cron_expression.as_deref().ok_or_else(|| {
                            TaskEngineError::InvalidInput(format!(
                                "Expression cron absente pour la tâche « {} ».",
                                task.id
                            ))
                        })?;
                        Some(CronSchedule::parse(expression).map_err(|error| {
                            TaskEngineError::InvalidInput(format!(
                                "Expression cron persistée invalide pour la tâche « {} » : {error}",
                                task.id
                            ))
                        })?)
                    } else {
                        None
                    };
                let mut scheduled_for = task.scheduled_at.clone();
                for _ in 0..MAX_CRON_CATCH_UP_PER_TASK {
                    if scheduled_for > now_text {
                        break;
                    }
                    let transaction = connection
                        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
                        .map_err(TaskEngineError::from)?;
                    let key = format!("cron:{}:{}", task.id, scheduled_for);
                    match TaskEngine::enqueue_execution(
                        &transaction,
                        &task.id,
                        &key,
                        &scheduled_for,
                        &now_text,
                    ) {
                        Ok(_) => {}
                        Err(TaskEngineError::TaskInactive) => {
                            transaction.rollback().map_err(TaskEngineError::from)?;
                            break;
                        }
                        Err(error) => return Err(error.into()),
                    }
                    let next_scheduled_for = if let Some(schedule) = cron_schedule.as_ref() {
                        let scheduled_time = DateTime::parse_from_rfc3339(&scheduled_for).map_err(|_| TaskEngineError::InvalidInput("Une échéance récurrente persistée n'est pas une date ISO 8601 valide.".to_string()))?.with_timezone(&Utc);
                        timestamp(&schedule.next_after(&scheduled_time).ok_or_else(|| TaskEngineError::InvalidInput("L'expression cron ne produit aucune occurrence suivante dans les cinq prochaines années.".to_string()))?)
                    } else {
                        let anchor = task.cron_anchor_at.as_deref().ok_or_else(|| {
                            TaskEngineError::InvalidInput(format!(
                                "Ancre absente pour la récurrence métier de la tâche « {} ».",
                                task.id
                            ))
                        })?;
                        TaskEngine::next_anchored_occurrence(
                            task.recurrence_frequency,
                            anchor,
                            &scheduled_for,
                        )?
                    };
                    match TaskEngine::advance_cron_occurrence_in_transaction(
                        &transaction,
                        &task.id,
                        &scheduled_for,
                        &next_scheduled_for,
                        &now_text,
                    ) {
                        Ok(()) => {}
                        Err(TaskEngineError::StaleSchedule) => {
                            transaction.rollback().map_err(TaskEngineError::from)?;
                            break;
                        }
                        Err(error) => return Err(error.into()),
                    }
                    transaction.commit().map_err(TaskEngineError::from)?;
                    scheduled_for = next_scheduled_for;
                }
                continue;
            }

            // Tâche ponctuelle : matérialisation idempotente unique.
            if task.scheduled_at > now_text {
                continue;
            }
            let key = format!("one-off:{}", task.id);
            match TaskEngine::enqueue_execution(
                connection,
                &task.id,
                &key,
                &task.scheduled_at,
                &now_text,
            ) {
                Ok(_) | Err(TaskEngineError::TaskInactive) => {}
                Err(error) => return Err(error.into()),
            }
        }

        let claimed = if registered_handlers_only {
            TaskEngine::claim_next_due_for_task_ids(
                connection,
                &now_text,
                &lease_until,
                &eligible_task_ids,
            )?
        } else {
            TaskEngine::claim_next_due(connection, &now_text, &lease_until)?
        };
        let Some(execution) = claimed else {
            return Ok(WorkerOutcome::Idle);
        };
        let task = TaskEngine::get_task_by_id(connection, &execution.task_id)?
            .ok_or_else(|| TaskEngineError::TaskNotFound(execution.task_id.clone()))?;

        // La transaction de réservation est terminée avant l'appel au code métier.
        match registry.invoke(&task, &execution) {
            Ok(()) => {
                TaskEngine::complete_execution(
                    connection,
                    &execution.id,
                    execution.attempt_count,
                    &now_text,
                )?;
                Ok(WorkerOutcome::Succeeded {
                    execution_id: execution.id,
                    attempt_number: execution.attempt_count,
                })
            }
            Err(message) => {
                let error = bounded_error(message);
                let retry_at = timestamp(&(now.clone() + config.retry_delay));
                let updated = TaskEngine::fail_execution(
                    connection,
                    &execution.id,
                    execution.attempt_count,
                    &now_text,
                    &retry_at,
                    &error,
                )?;
                match updated.status {
                    ExecutionStatus::Queued => Ok(WorkerOutcome::RetryScheduled {
                        execution_id: updated.id,
                        attempt_number: updated.attempt_count,
                        retry_at,
                        error,
                    }),
                    ExecutionStatus::Failed => Ok(WorkerOutcome::Failed {
                        execution_id: updated.id,
                        attempt_number: updated.attempt_count,
                        error,
                    }),
                    _ => Err(TaskEngineError::InvalidExecutionState.into()),
                }
            }
        }
    }
}

fn timestamp(value: &DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn bounded_error(message: String) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return "Le gestionnaire a échoué sans fournir de détail.".to_string();
    }
    trimmed.chars().take(4_000).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AttemptStatus, ExecutionStatus, NewTask, TaskEngine};
    use rusqlite::{OpenFlags, OptionalExtension};
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };
    use uuid::Uuid;

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
                    .join(format!("superpower-task-worker-{}.db", Uuid::new_v4())),
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

    fn at(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .expect("test timestamp")
            .with_timezone(&Utc)
    }

    fn create_due_task(connection: &Connection, idempotency_key: &str, max_attempts: u32) -> Task {
        let input = NewTask {
            idempotency_key: idempotency_key.to_string(),
            module: "documents".to_string(),
            task_type: "classify_document".to_string(),
            title: "Classer un document".to_string(),
            payload: serde_json::json!({"documentId": "document-test"}),
            scheduled_at: DUE.to_string(),
            recurrence_frequency: TaskRecurrenceFrequency::Once,
            cron_expression: None,
            max_attempts,
        };
        TaskEngine::create_task(connection, &input, BEFORE_DUE).expect("create task")
    }

    #[test]
    fn worker_materializes_due_task_and_records_success() {
        let database = TemporaryDatabase::new();
        let mut connection = database.connect();
        TaskEngine::install_schema(&connection).expect("install schema");
        let task = create_due_task(&connection, "worker-success", 3);

        let registry = TaskHandlerRegistry::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let handler_calls = Arc::clone(&calls);
        registry
            .register_handler(
                " documents ",
                "classify_document",
                move |task, execution| {
                    assert_eq!(task.title, "Classer un document");
                    assert_eq!(execution.attempt_count, 1);
                    handler_calls.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
            )
            .expect("register handler");

        let outcome = TaskWorker::run_once(
            &mut connection,
            &registry,
            at("2026-10-13T10:01:00Z"),
            TaskWorkerConfig::default(),
        )
        .expect("run task");

        assert!(matches!(
            outcome,
            WorkerOutcome::Succeeded {
                attempt_number: 1,
                ..
            }
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let executions = TaskEngine::list_executions(&connection, &task.id).unwrap();
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].status, ExecutionStatus::Succeeded);
        let attempts = TaskEngine::list_attempts(&connection, &executions[0].id).unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].status, AttemptStatus::Succeeded);

        assert_eq!(
            TaskWorker::run_once(
                &mut connection,
                &registry,
                at("2026-10-13T10:02:00Z"),
                TaskWorkerConfig::default(),
            )
            .unwrap(),
            WorkerOutcome::Idle
        );
    }

    #[test]
    fn worker_records_handler_errors_and_schedules_retry() {
        let database = TemporaryDatabase::new();
        let mut connection = database.connect();
        TaskEngine::install_schema(&connection).expect("install schema");
        let task = create_due_task(&connection, "worker-retry", 2);
        let registry = TaskHandlerRegistry::new();
        registry
            .register_handler("documents", "classify_document", |_task, _execution| {
                Err("Service temporairement indisponible".to_string())
            })
            .unwrap();

        let outcome = TaskWorker::run_once(
            &mut connection,
            &registry,
            at("2026-10-13T10:01:00Z"),
            TaskWorkerConfig::default(),
        )
        .unwrap();
        assert!(matches!(
            outcome,
            WorkerOutcome::RetryScheduled {
                attempt_number: 1,
                ref error,
                ..
            } if error == "Service temporairement indisponible"
        ));

        let executions = TaskEngine::list_executions(&connection, &task.id).unwrap();
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].status, ExecutionStatus::Queued);
        assert_eq!(
            executions[0].last_error.as_deref(),
            Some("Service temporairement indisponible")
        );
        let attempts = TaskEngine::list_attempts(&connection, &executions[0].id).unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].status, AttemptStatus::Failed);
        assert_eq!(
            attempts[0].error_message.as_deref(),
            Some("Service temporairement indisponible")
        );
    }

    #[test]
    fn worker_catches_handler_panics_and_records_them_as_failed_attempts() {
        let database = TemporaryDatabase::new();
        let mut connection = database.connect();
        TaskEngine::install_schema(&connection).expect("install schema");
        let task = create_due_task(&connection, "worker-handler-panic", 2);
        let registry = TaskHandlerRegistry::new();
        registry
            .register_handler(
                "documents",
                "classify_document",
                |_task, _execution| -> Result<(), String> { panic!("simulated handler panic") },
            )
            .unwrap();

        let outcome = TaskWorker::run_once(
            &mut connection,
            &registry,
            at("2026-10-13T10:01:00Z"),
            TaskWorkerConfig::default(),
        )
        .expect("convert handler panic into a recorded failure");

        assert!(matches!(
            outcome,
            WorkerOutcome::RetryScheduled {
                attempt_number: 1,
                ref error,
                ..
            } if error == "Le gestionnaire a paniqué pendant l'exécution."
        ));

        let executions = TaskEngine::list_executions(&connection, &task.id).unwrap();
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].status, ExecutionStatus::Queued);
        assert_eq!(
            executions[0].last_error.as_deref(),
            Some("Le gestionnaire a paniqué pendant l'exécution.")
        );
        let attempts = TaskEngine::list_attempts(&connection, &executions[0].id).unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].status, AttemptStatus::Failed);
        assert_eq!(
            attempts[0].error_message.as_deref(),
            Some("Le gestionnaire a paniqué pendant l'exécution.")
        );
    }

    #[test]
    fn safe_worker_leaves_due_tasks_untouched_without_a_registered_handler() {
        let database = TemporaryDatabase::new();
        let mut connection = database.connect();
        TaskEngine::install_schema(&connection).expect("install schema");
        let task = create_due_task(&connection, "safe-worker-no-handler", 2);
        let registry = TaskHandlerRegistry::new();

        let outcome = TaskWorker::run_once_for_registered_handlers(
            &mut connection,
            &registry,
            at("2026-10-13T10:01:00Z"),
            TaskWorkerConfig::default(),
        )
        .expect("missing handler should defer the task safely");

        assert_eq!(outcome, WorkerOutcome::Idle);
        assert!(TaskEngine::list_executions(&connection, &task.id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn safe_worker_skips_unhandled_tasks_without_starving_registered_tasks() {
        let database = TemporaryDatabase::new();
        let mut connection = database.connect();
        TaskEngine::install_schema(&connection).expect("install schema");
        let unhandled = create_due_task(&connection, "safe-worker-unhandled-first", 2);
        let handled = TaskEngine::create_task(
            &connection,
            &NewTask {
                idempotency_key: "safe-worker-handled-second".to_string(),
                module: "documents".to_string(),
                task_type: "registered_type".to_string(),
                title: "Tâche avec handler".to_string(),
                payload: serde_json::json!({}),
                scheduled_at: DUE.to_string(),
                recurrence_frequency: TaskRecurrenceFrequency::Once,
                cron_expression: None,
                max_attempts: 2,
            },
            BEFORE_DUE,
        )
        .expect("create registered task");
        let registry = TaskHandlerRegistry::new();
        registry
            .register_handler("documents", "registered_type", |_task, _execution| Ok(()))
            .unwrap();

        let outcome = TaskWorker::run_once_for_registered_handlers(
            &mut connection,
            &registry,
            at("2026-10-13T10:01:00Z"),
            TaskWorkerConfig::default(),
        )
        .expect("process only a registered handler");

        assert!(matches!(outcome, WorkerOutcome::Succeeded { .. }));
        assert!(TaskEngine::list_executions(&connection, &unhandled.id)
            .unwrap()
            .is_empty());
        let executions = TaskEngine::list_executions(&connection, &handled.id).unwrap();
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].status, ExecutionStatus::Succeeded);
    }

    #[test]
    fn safe_worker_does_not_materialize_cron_for_unhandled_task() {
        let database = TemporaryDatabase::new();
        let mut connection = database.connect();
        TaskEngine::install_schema(&connection).expect("install schema");
        let task = TaskEngine::create_task(
            &connection,
            &NewTask {
                idempotency_key: "safe-worker-unhandled-cron".to_string(),
                module: "documents".to_string(),
                task_type: "future_handler".to_string(),
                title: "Cron sans handler".to_string(),
                payload: serde_json::json!({}),
                scheduled_at: DUE.to_string(),
                recurrence_frequency: TaskRecurrenceFrequency::Once,
                cron_expression: Some("*/5 * * * *".to_string()),
                max_attempts: 2,
            },
            BEFORE_DUE,
        )
        .expect("create cron task");
        let original_scheduled_at = task.scheduled_at.clone();
        let registry = TaskHandlerRegistry::new();

        let outcome = TaskWorker::run_once_for_registered_handlers(
            &mut connection,
            &registry,
            at("2026-10-13T10:01:00Z"),
            TaskWorkerConfig::default(),
        )
        .expect("defer cron task without a handler");

        assert_eq!(outcome, WorkerOutcome::Idle);
        assert!(TaskEngine::list_executions(&connection, &task.id)
            .unwrap()
            .is_empty());
        let unchanged = TaskEngine::get_task_by_id(&connection, &task.id)
            .unwrap()
            .unwrap();
        assert_eq!(unchanged.scheduled_at, original_scheduled_at);
    }

    #[test]
    fn missing_handler_fails_visibly_and_exhausts_the_attempt_budget() {
        let database = TemporaryDatabase::new();
        let mut connection = database.connect();
        TaskEngine::install_schema(&connection).expect("install schema");
        let task = create_due_task(&connection, "worker-missing-handler", 1);
        let registry = TaskHandlerRegistry::new();

        let outcome = TaskWorker::run_once(
            &mut connection,
            &registry,
            at("2026-10-13T10:01:00Z"),
            TaskWorkerConfig::default(),
        )
        .expect("persist missing-handler failure");

        assert!(matches!(
            outcome,
            WorkerOutcome::Failed {
                attempt_number: 1,
                ref error,
                ..
            } if error.contains("Aucun gestionnaire")
        ));
        let executions = TaskEngine::list_executions(&connection, &task.id).unwrap();
        assert_eq!(executions.len(), 1);
        assert_eq!(executions[0].status, ExecutionStatus::Failed);
        assert!(executions[0]
            .last_error
            .as_deref()
            .is_some_and(|error| error.contains("Aucun gestionnaire")));
        let attempts = TaskEngine::list_attempts(&connection, &executions[0].id).unwrap();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].status, AttemptStatus::Failed);
    }

    #[test]
    fn worker_materializes_monthly_occurrences_from_original_anchor_without_duplicates() {
        let database = TemporaryDatabase::new();
        let mut connection = database.connect();
        TaskEngine::install_schema(&connection).expect("install schema");

        let input = NewTask {
            idempotency_key: "worker-monthly-anchor".to_string(),
            module: "documents".to_string(),
            task_type: "classify_document".to_string(),
            title: "Contrôle mensuel du document".to_string(),
            payload: serde_json::json!({"documentId": "document-monthly-test"}),
            scheduled_at: "2026-01-31T10:00:00Z".to_string(),
            recurrence_frequency: TaskRecurrenceFrequency::Monthly,
            cron_expression: None,
            max_attempts: 2,
        };
        let task = TaskEngine::create_task(&connection, &input, "2026-01-01T09:00:00Z")
            .expect("create monthly task");
        assert_eq!(
            task.cron_anchor_at.as_deref(),
            Some("2026-01-31T10:00:00.000Z")
        );

        let registry = TaskHandlerRegistry::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let handler_calls = Arc::clone(&calls);
        registry
            .register_handler(
                "documents",
                "classify_document",
                move |_task, _execution| {
                    handler_calls.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
            )
            .expect("register handler");

        let occurrences = [
            (
                "2026-01-31T10:01:00Z",
                "2026-01-31T10:00:00.000Z",
                "2026-02-28T10:00:00.000Z",
                "2026-01-31T10:02:00Z",
            ),
            (
                "2026-02-28T10:01:00Z",
                "2026-02-28T10:00:00.000Z",
                "2026-03-31T10:00:00.000Z",
                "2026-02-28T10:02:00Z",
            ),
            (
                "2026-03-31T10:01:00Z",
                "2026-03-31T10:00:00.000Z",
                "2026-04-30T10:00:00.000Z",
                "2026-03-31T10:02:00Z",
            ),
        ];

        for (index, (now, expected_occurrence, expected_next, repeated_at)) in
            occurrences.iter().enumerate()
        {
            let outcome = TaskWorker::run_once(
                &mut connection,
                &registry,
                at(now),
                TaskWorkerConfig::default(),
            )
            .expect("run monthly occurrence");
            assert!(matches!(outcome, WorkerOutcome::Succeeded { .. }));

            let mut executions = TaskEngine::list_executions(&connection, &task.id).unwrap();
            executions.sort_by(|left, right| left.scheduled_for.cmp(&right.scheduled_for));
            assert_eq!(executions.len(), index + 1);
            assert_eq!(executions[index].scheduled_for, *expected_occurrence);
            assert_eq!(executions[index].status, ExecutionStatus::Succeeded);

            let updated = TaskEngine::get_task_by_id(&connection, &task.id)
                .unwrap()
                .unwrap();
            assert_eq!(updated.scheduled_at, *expected_next);
            assert_eq!(
                updated.cron_anchor_at.as_deref(),
                Some("2026-01-31T10:00:00.000Z"),
                "l'ancre d'origine ne doit pas dériver"
            );

            let repeated = TaskWorker::run_once(
                &mut connection,
                &registry,
                at(repeated_at),
                TaskWorkerConfig::default(),
            )
            .expect("repeat the worker pass");
            assert_eq!(repeated, WorkerOutcome::Idle);
            assert_eq!(
                TaskEngine::list_executions(&connection, &task.id)
                    .unwrap()
                    .len(),
                index + 1,
                "la passe répétée ne doit pas dupliquer l'occurrence"
            );
            assert_eq!(calls.load(Ordering::SeqCst), index + 1);
        }
    }

    #[test]
    fn worker_materializes_due_cron_occurrences_idempotently_and_advances_next_date() {
        let database = TemporaryDatabase::new();
        let mut connection = database.connect();
        TaskEngine::install_schema(&connection).expect("install schema");

        let input = NewTask {
            idempotency_key: "worker-cron".to_string(),
            module: "documents".to_string(),
            task_type: "classify_document".to_string(),
            title: "Contrôler les documents chaque matin".to_string(),
            payload: serde_json::json!({"documentId": "document-test"}),
            scheduled_at: "2026-10-13T10:00:00Z".to_string(),
            recurrence_frequency: TaskRecurrenceFrequency::Once,
            cron_expression: Some("0 10 * * 1-5".to_string()),
            max_attempts: 2,
        };
        let task = TaskEngine::create_task(&connection, &input, BEFORE_DUE).unwrap();

        let registry = TaskHandlerRegistry::new();
        registry
            .register_handler("documents", "classify_document", |_task, _execution| Ok(()))
            .unwrap();

        TaskWorker::run_once(
            &mut connection,
            &registry,
            at("2026-10-15T10:01:00Z"),
            TaskWorkerConfig::default(),
        )
        .expect("materialize and run the earliest due occurrence");

        let mut executions = TaskEngine::list_executions(&connection, &task.id).unwrap();
        executions.sort_by(|left, right| left.scheduled_for.cmp(&right.scheduled_for));
        assert_eq!(executions.len(), 3);
        assert_eq!(executions[0].scheduled_for, "2026-10-13T10:00:00.000Z");
        assert_eq!(executions[0].status, ExecutionStatus::Succeeded);
        assert_eq!(executions[1].scheduled_for, "2026-10-14T10:00:00.000Z");
        assert_eq!(executions[2].scheduled_for, "2026-10-15T10:00:00.000Z");

        let updated = TaskEngine::get_task_by_id(&connection, &task.id)
            .unwrap()
            .unwrap();
        assert_eq!(updated.scheduled_at, "2026-10-16T10:00:00.000Z");

        TaskWorker::run_once(
            &mut connection,
            &registry,
            at("2026-10-15T10:01:00Z"),
            TaskWorkerConfig::default(),
        )
        .expect("repeat run");
        assert_eq!(
            TaskEngine::list_executions(&connection, &task.id)
                .unwrap()
                .len(),
            3,
            "la matérialisation répétée ne doit pas dupliquer les occurrences"
        );
    }

    #[test]
    fn registry_rejects_duplicate_handler_registration() {
        let registry = TaskHandlerRegistry::new();
        registry
            .register_handler("documents", "classify_document", |_task, _execution| Ok(()))
            .unwrap();
        assert_eq!(
            registry.register_handler(" documents ", "classify_document", |_task, _execution| {
                Ok(())
            }),
            Err(HandlerRegistryError::Duplicate {
                module: "documents".to_string(),
                task_type: "classify_document".to_string(),
            })
        );
    }
}
