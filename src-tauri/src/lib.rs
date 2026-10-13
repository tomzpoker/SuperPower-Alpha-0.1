mod commands;
mod database;
mod task_worker;

use std::io;

use rusqlite::OptionalExtension;
use task_engine::{Task, TaskExecution, TaskHandlerRegistry};
use tauri::Manager;

use crate::task_worker::{
    reminder_message, ReminderNotificationQueue, TaskReminderEvent, TaskWorkerController,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let default_app_data_dir = app.path().app_data_dir()?;
            #[cfg(debug_assertions)]
            let app_data_dir = std::env::var_os("SUPERPOWER_TEST_DATA_DIR")
                .map(std::path::PathBuf::from)
                .map(|test_dir| {
                    if !test_dir.is_absolute()
                        || test_dir.as_path() == default_app_data_dir.as_path()
                    {
                        Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "SUPERPOWER_TEST_DATA_DIR doit pointer vers un dossier absolu distinct du coffre habituel.",
                        ))
                    } else {
                        Ok(test_dir)
                    }
                })
                .transpose()?
                .unwrap_or(default_app_data_dir);
            #[cfg(not(debug_assertions))]
            let app_data_dir = default_app_data_dir;
            std::fs::create_dir_all(&app_data_dir)?;

            let notifications = ReminderNotificationQueue::default();
            let handler_notifications = notifications.clone();
            let mut handlers = TaskHandlerRegistry::new();
            handlers
                .register_handler("general", "reminder", move |task, execution| {
                    handler_notifications.enqueue(TaskReminderEvent {
                        execution_id: execution.id.clone(),
                        title: task.title.clone(),
                        message: reminder_message(&task.title, &task.payload),
                        scheduled_for: execution.scheduled_for.clone(),
                    })
                })
                .map_err(|error| io::Error::other(error.to_string()))?;

            // Rappel lié à un flux de trésorerie existant : seul le solde restant est annoncé.
            let flux_reminder_app = app.handle().clone();
            let flux_reminder_queue = notifications.clone();
            handlers
                .register_handler("tresorerie", "rappel_flux", move |task, execution| {
                    let state = flux_reminder_app.state::<database::AppState>();
                    if let Some(event) =
                        build_flux_reminder_event(state.inner(), task, execution)?
                    {
                        flux_reminder_queue.enqueue(event)?;
                    }
                    Ok(())
                })
                .map_err(|error| io::Error::other(error.to_string()))?;

            app.manage(handlers);
            app.manage(notifications);
            app.manage(TaskWorkerController::default());
            app.manage(database::AppState::new(app_data_dir.join("superpower.db")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::database_status,
            commands::initialize_database,
            commands::unlock_database,
            commands::lock_database,
            commands::list_societes,
            commands::create_societe,
            commands::update_societe,
            commands::delete_societe,
            commands::list_flux,
            commands::create_flux,
            commands::update_flux,
            commands::cancel_flux,
            commands::arreter_flux_recurrence,
            commands::register_flux_realisation,
            commands::cancel_flux_realisation,
            commands::list_comptes,
            commands::create_ecriture,
            commands::list_ecritures,
            commands::list_tasks,
            commands::preview_cron_schedule,
            commands::create_task,
            commands::delete_task,
            commands::update_task,
            commands::update_task_status,
            commands::list_task_executions,
            commands::list_task_attempts,
            commands::dashboard_summary,
        ])
        .build(tauri::generate_context!())
        .expect("Erreur au démarrage de SuperPower Alpha");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            if let Err(error) = app_handle.state::<TaskWorkerController>().stop() {
                eprintln!("Arrêt du worker de tâches impossible : {error}");
            }
        }
    });
}

/// Construit une notification depuis un flux réel du coffre. Un flux annulé ou soldé
/// n'entraîne pas de notification ; les réalisations annulées ne diminuent pas le solde.
fn build_flux_reminder_event(
    state: &database::AppState,
    task: &Task,
    execution: &TaskExecution,
) -> Result<Option<TaskReminderEvent>, String> {
    let flux_id = task
        .payload
        .get("fluxId")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "Le rappel de trésorerie doit contenir un payload.fluxId non vide.".to_string()
        })?;

    let connection = state.connection()?;
    let flux: Option<(String, String, i64, String, String, i64)> = connection
        .query_row(
            "SELECT f.libelle, f.date_prevue, f.montant_prevu_cents, f.sens, f.statut,
                    COALESCE(
                        SUM(CASE WHEN r.annulee_at IS NULL THEN r.montant_cents ELSE 0 END),
                        0
                    )
             FROM flux_previsionnels f
             LEFT JOIN flux_realisations r ON r.flux_id = f.id
             WHERE f.id = ?1
             GROUP BY f.id, f.libelle, f.date_prevue, f.montant_prevu_cents, f.sens, f.statut",
            [flux_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()
        .map_err(|error| format!("Lecture du flux de trésorerie impossible : {error}"))?;

    let Some((label, due_date, planned_cents, sens, status, realized_cents)) = flux else {
        return Err(format!(
            "Le flux de trésorerie « {flux_id} » est introuvable."
        ));
    };
    if status == "annule" {
        return Ok(None);
    }
    if status != "actif" {
        return Err(format!(
            "Le flux de trésorerie « {flux_id} » a un statut inconnu : {status}."
        ));
    }

    let remaining_cents = planned_cents - realized_cents;
    if remaining_cents <= 0 {
        return Ok(None);
    }
    let direction = match sens.as_str() {
        "encaissement" => "à encaisser",
        "decaissement" => "à décaisser",
        _ => {
            return Err(format!(
                "Le flux de trésorerie « {flux_id} » a un sens inconnu : {sens}."
            ));
        }
    };

    let mut message = format!(
        "Échéance « {label} » prévue le {due_date} : reste {} {direction}.",
        format_amount_cents_fr(remaining_cents)
    );
    if let Some(extra) = task
        .payload
        .get("message")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        message.push(' ');
        message.push_str(extra);
    }
    let message = message.chars().take(300).collect();

    Ok(Some(TaskReminderEvent {
        execution_id: execution.id.clone(),
        title: task.title.clone(),
        message,
        scheduled_for: execution.scheduled_for.clone(),
    }))
}

fn format_amount_cents_fr(cents: i64) -> String {
    let euros = (cents / 100).to_string();
    let digit_count = euros.len();
    let mut grouped = String::with_capacity(digit_count + digit_count / 3);
    for (index, digit) in euros.chars().enumerate() {
        if index > 0 && (digit_count - index) % 3 == 0 {
            grouped.push('\u{202f}');
        }
        grouped.push(digit);
    }
    format!("{grouped},{:02} €", cents % 100)
}

#[cfg(test)]
mod flux_reminder_tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use std::{fs, path::PathBuf, sync::{Arc, Mutex}};
    use task_engine::{
        ExecutionStatus, NewTask, TaskEngine, TaskRecurrenceFrequency, TaskStatus, TaskWorker,
        TaskWorkerConfig, WorkerOutcome,
    };
    use uuid::Uuid;

    const TEST_PASSPHRASE: &str = "flux reminder temporary vault passphrase";

    struct TemporaryVault(PathBuf);

    impl Drop for TemporaryVault {
        fn drop(&mut self) {
            for path in [
                self.0.clone(),
                self.0.with_extension("salt"),
                self.0.with_extension("db-wal"),
                self.0.with_extension("db-shm"),
            ] {
                let _ = fs::remove_file(path);
            }
        }
    }

    fn test_state() -> (TemporaryVault, database::AppState) {
        let vault = TemporaryVault(
            std::env::temp_dir().join(format!("superpower-flux-reminder-{}.db", Uuid::new_v4())),
        );
        let state = database::AppState::new(vault.0.clone());
        state
            .initialize(TEST_PASSPHRASE)
            .expect("initialize temporary SQLCipher vault");
        (vault, state)
    }

    fn seed_flux(
        state: &database::AppState,
        id: &str,
        sens: &str,
        status: &str,
        planned_cents: i64,
    ) {
        let connection = state.connection().expect("open temporary vault");
        connection
            .execute(
                "INSERT INTO societes (
                    id, nom, forme_juridique, regime_fiscal, regime_tva, exigibilite_tva,
                    periodicite_tva, siret, fin_exercice, created_at
                 ) VALUES (?1, ?2, 'sci', 'ir', 'non_renseigne', 'non_definie',
                           'non_definie', NULL, '2026-12-31', '2026-12-01T00:00:00Z')",
                rusqlite::params!["society-flux-reminder", "SCI de test"],
            )
            .expect("insert test society");
        connection
            .execute(
                "INSERT INTO flux_previsionnels (
                    id, societe_id, libelle, sens, categorie, date_prevue,
                    montant_prevu_cents, statut, created_at, updated_at
                 ) VALUES (?1, 'society-flux-reminder', 'Loyer du local', ?2, 'loyer',
                           '2027-01-05', ?3, ?4, '2026-12-01T00:00:00Z',
                           '2026-12-01T00:00:00Z')",
                rusqlite::params![id, sens, planned_cents, status],
            )
            .expect("insert test cash flow");
    }

    fn seed_realisation(
        state: &database::AppState,
        id: &str,
        flux_id: &str,
        cents: i64,
        cancelled: bool,
    ) {
        let (cancelled_at, reason): (Option<&str>, Option<&str>) = if cancelled {
            (
                Some("2027-01-06T10:00:00Z"),
                Some("Réalisation de test annulée"),
            )
        } else {
            (None, None)
        };
        let connection = state.connection().expect("open temporary vault");
        connection
            .execute(
                "INSERT INTO flux_realisations (
                    id, flux_id, date_realisation, montant_cents, created_at,
                    annulee_at, motif_annulation
                 ) VALUES (?1, ?2, '2027-01-04', ?3, '2027-01-04T10:00:00Z', ?4, ?5)",
                rusqlite::params![id, flux_id, cents, cancelled_at, reason],
            )
            .expect("insert test cash realization");
    }

    fn task(flux_id: &str) -> Task {
        Task {
            id: "task-flux-reminder".to_string(),
            idempotency_key: "task-flux-reminder-key".to_string(),
            module: "tresorerie".to_string(),
            task_type: "rappel_flux".to_string(),
            title: "Rappeler le flux".to_string(),
            payload: serde_json::json!({
                "fluxId": flux_id,
                "message": "Vérifier le règlement."
            }),
            scheduled_at: "2027-01-05T08:00:00.000Z".to_string(),
            recurrence_frequency: TaskRecurrenceFrequency::Once,
            cron_expression: None,
            cron_anchor_at: None,
            status: TaskStatus::Scheduled,
            max_attempts: 3,
            created_at: "2026-12-01T10:00:00.000Z".to_string(),
            updated_at: "2026-12-01T10:00:00.000Z".to_string(),
        }
    }

    fn execution() -> TaskExecution {
        TaskExecution {
            id: "execution-flux-reminder".to_string(),
            task_id: "task-flux-reminder".to_string(),
            idempotency_key: "execution-flux-reminder-key".to_string(),
            scheduled_for: "2027-01-05T08:00:00.000Z".to_string(),
            available_at: "2027-01-05T08:00:00.000Z".to_string(),
            status: ExecutionStatus::Running,
            attempt_count: 1,
            max_attempts: 3,
            lease_until: None,
            last_error: None,
            created_at: "2027-01-05T08:00:00.000Z".to_string(),
            updated_at: "2027-01-05T08:00:00.000Z".to_string(),
            started_at: Some("2027-01-05T08:00:00.000Z".to_string()),
            finished_at: None,
        }
    }

    #[test]
    fn flux_reminder_uses_actual_unsettled_balance_and_ignores_cancelled_realizations() {
        let (_vault, state) = test_state();
        seed_flux(&state, "flux-1", "encaissement", "actif", 150_000);
        seed_realisation(&state, "real-1", "flux-1", 10_000, false);
        seed_realisation(&state, "real-cancelled", "flux-1", 20_000, true);

        let task = task("flux-1");
        let event = build_flux_reminder_event(&state, &task, &execution())
            .expect("build cash-flow reminder")
            .expect("an unpaid active flow should trigger a reminder");

        assert_eq!(event.execution_id, "execution-flux-reminder");
        assert_eq!(event.title, "Rappeler le flux");
        assert_eq!(event.scheduled_for, "2027-01-05T08:00:00.000Z");
        assert!(event
            .message
            .contains("Échéance « Loyer du local » prévue le 2027-01-05"));
        assert!(event
            .message
            .contains("reste 1\u{202f}400,00 € à encaisser."));
        assert!(event.message.ends_with("Vérifier le règlement."));
    }

    #[test]
    fn flux_reminder_uses_the_decaissement_direction_and_amount() {
        let (_vault, state) = test_state();
        seed_flux(&state, "flux-out", "decaissement", "actif", 75_000);

        let event = build_flux_reminder_event(&state, &task("flux-out"), &execution())
            .expect("build outflow reminder")
            .expect("an unpaid active outflow should trigger a reminder");

        assert!(event.message.contains("reste 750,00 € à décaisser."));
    }

    #[test]
    fn flux_reminder_skips_cancelled_or_fully_settled_flows() {
        let (_vault, cancelled_state) = test_state();
        seed_flux(
            &cancelled_state,
            "flux-cancelled",
            "encaissement",
            "annule",
            150_000,
        );
        assert!(
            build_flux_reminder_event(&cancelled_state, &task("flux-cancelled"), &execution())
                .expect("cancelled flows should be harmless")
                .is_none()
        );

        let (_vault, settled_state) = test_state();
        seed_flux(
            &settled_state,
            "flux-settled",
            "encaissement",
            "actif",
            150_000,
        );
        seed_realisation(&settled_state, "real-full", "flux-settled", 150_000, false);
        assert!(
            build_flux_reminder_event(&settled_state, &task("flux-settled"), &execution())
                .expect("settled flows should be harmless")
                .is_none()
        );
    }

    #[test]
    fn registered_worker_runs_treasury_reminder_once_and_persists_success() {
        let (_vault, raw_state) = test_state();
        seed_flux(&raw_state, "flux-worker", "encaissement", "actif", 150_000);
        seed_realisation(&raw_state, "real-worker", "flux-worker", 10_000, false);

        let state = Arc::new(raw_state);
        let observed_events = Arc::new(Mutex::new(Vec::<TaskReminderEvent>::new()));
        let registry = TaskHandlerRegistry::new();
        let handler_state = Arc::clone(&state);
        let handler_events = Arc::clone(&observed_events);
        registry
            .register_handler("tresorerie", "rappel_flux", move |task, execution| {
                if let Some(event) =
                    build_flux_reminder_event(handler_state.as_ref(), task, execution)?
                {
                    handler_events
                        .lock()
                        .map_err(|_| "Le stockage de test du rappel est indisponible.".to_string())?
                        .push(event);
                }
                Ok(())
            })
            .expect("register the real treasury reminder handler");

        let input = NewTask {
            idempotency_key: "flux-worker-integration-key".to_string(),
            module: "tresorerie".to_string(),
            task_type: "rappel_flux".to_string(),
            title: "Rappel de trésorerie — loyer".to_string(),
            payload: serde_json::json!({
                "fluxId": "flux-worker",
                "message": "Vérifier le règlement."
            }),
            scheduled_at: "2027-01-05T08:00:00.000Z".to_string(),
            recurrence_frequency: TaskRecurrenceFrequency::Once,
            cron_expression: None,
            max_attempts: 3,
        };

        let mut connection = state.connection().expect("open SQLCipher worker connection");
        let saved = TaskEngine::create_task(&connection, &input, "2027-01-01T00:00:00Z")
            .expect("persist the treasury reminder task");
        let now = DateTime::parse_from_rfc3339("2027-01-05T08:01:00Z")
            .expect("parse deterministic worker time")
            .with_timezone(&Utc);

        let first = TaskWorker::run_once_for_registered_handlers(
            &mut connection,
            &registry,
            now,
            TaskWorkerConfig::default(),
        )
        .expect("run the registered treasury handler through the real worker");
        let execution_id = match first {
            WorkerOutcome::Succeeded {
                execution_id,
                attempt_number,
            } => {
                assert_eq!(attempt_number, 1);
                execution_id
            }
            other => panic!("Expected a successful treasury reminder, got {other:?}"),
        };

        let history = TaskEngine::list_executions(&connection, &saved.id)
            .expect("read persisted worker history");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, execution_id);
        assert_eq!(history[0].status, ExecutionStatus::Succeeded);

        {
            let events = observed_events.lock().expect("read observed reminder");
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].execution_id, execution_id);
            assert!(events[0].message.contains("reste 1\u{202f}400,00 € à encaisser."));
            assert!(events[0].message.ends_with("Vérifier le règlement."));
        }

        let second = TaskWorker::run_once_for_registered_handlers(
            &mut connection,
            &registry,
            now,
            TaskWorkerConfig::default(),
        )
        .expect("repeat a worker pass with the same simulated time");
        assert_eq!(second, WorkerOutcome::Idle);
        assert_eq!(
            TaskEngine::list_executions(&connection, &saved.id)
                .expect("re-read persisted history")
                .len(),
            1,
            "a repeated pass must not create a duplicate execution"
        );
        assert_eq!(
            observed_events.lock().expect("read event count").len(),
            1,
            "a repeated pass must not invoke the reminder handler twice"
        );
    }

    #[test]
    fn flux_reminder_rejects_missing_or_unknown_flux_ids() {
        let (_vault, state) = test_state();
        let mut missing = task("flux-ignored");
        missing.payload = serde_json::json!({"message": "missing id"});
        assert!(build_flux_reminder_event(&state, &missing, &execution())
            .expect_err("fluxId is required")
            .contains("fluxId"));
        assert!(
            build_flux_reminder_event(&state, &task("does-not-exist"), &execution())
                .expect_err("unknown flux must not silently succeed")
                .contains("introuvable")
        );
    }
}
