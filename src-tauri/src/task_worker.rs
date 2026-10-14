use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use task_engine::{TaskHandlerRegistry, TaskWorker, TaskWorkerConfig, WorkerOutcome};
use tauri::{AppHandle, Emitter, Manager};

use crate::database::AppState;

pub(crate) const TASK_REMINDER_EVENT: &str = "task-reminder";

const MAX_EXECUTIONS_PER_PASS: usize = 32;
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const POLL_TICKS: usize = 10;
const MAX_REMINDER_MESSAGE_CHARS: usize = 300;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TaskReminderEvent {
    pub(crate) execution_id: String,
    pub(crate) title: String,
    pub(crate) message: String,
    pub(crate) scheduled_for: String,
}

struct PendingReminder {
    event: TaskReminderEvent,
    completed: bool,
}

/// Les notifications restent en mémoire jusqu'à l'enregistrement du succès SQLCipher.
#[derive(Clone, Default)]
pub(crate) struct ReminderNotificationQueue(Arc<Mutex<VecDeque<PendingReminder>>>);

impl ReminderNotificationQueue {
    pub(crate) fn enqueue(&self, event: TaskReminderEvent) -> Result<(), String> {
        let mut pending = self
            .0
            .lock()
            .map_err(|_| "La file des rappels est indisponible.".to_string())?;
        if let Some(existing) = pending
            .iter_mut()
            .find(|item| item.event.execution_id == event.execution_id)
        {
            existing.event = event;
            return Ok(());
        }
        pending.push_back(PendingReminder {
            event,
            completed: false,
        });
        Ok(())
    }

    fn mark_completed(&self, execution_id: &str) -> Result<(), String> {
        let mut pending = self
            .0
            .lock()
            .map_err(|_| "La file des rappels est indisponible.".to_string())?;
        if let Some(item) = pending
            .iter_mut()
            .find(|item| item.event.execution_id == execution_id)
        {
            item.completed = true;
        }
        Ok(())
    }

    fn ready_events(&self) -> Result<Vec<TaskReminderEvent>, String> {
        let pending = self
            .0
            .lock()
            .map_err(|_| "La file des rappels est indisponible.".to_string())?;
        Ok(pending
            .iter()
            .filter(|item| item.completed)
            .map(|item| item.event.clone())
            .collect())
    }

    fn remove(&self, execution_id: &str) -> Result<(), String> {
        let mut pending = self
            .0
            .lock()
            .map_err(|_| "La file des rappels est indisponible.".to_string())?;
        pending.retain(|item| item.event.execution_id != execution_id);
        Ok(())
    }
}

pub(crate) fn reminder_message(title: &str, payload: &Value) -> String {
    let message = payload
        .get("message")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(title);
    message.chars().take(MAX_REMINDER_MESSAGE_CHARS).collect()
}

#[derive(Default)]
pub(crate) struct TaskWorkerController {
    running: Mutex<Option<RunningWorker>>,
}

struct RunningWorker {
    stop: Arc<AtomicBool>,
    join: JoinHandle<()>,
}

impl TaskWorkerController {
    /// Démarre au plus un worker, après l'ouverture réussie du coffre.
    pub(crate) fn start(&self, app: AppHandle) -> Result<(), String> {
        self.start_with(move |thread_stop| run_worker(app, thread_stop))
    }

    fn start_with<F>(&self, worker: F) -> Result<(), String>
    where
        F: FnOnce(Arc<AtomicBool>) + Send + 'static,
    {
        let mut slot = self
            .running
            .lock()
            .map_err(|_| "Le contrôleur du worker est indisponible.".to_string())?;

        if slot
            .as_ref()
            .is_some_and(|running| !running.join.is_finished())
        {
            return Ok(());
        }
        if let Some(finished) = slot.take() {
            if finished.join.join().is_err() {
                return Err("Le précédent worker de tâches s'est arrêté anormalement.".to_string());
            }
        }

        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let join = thread::Builder::new()
            .name("superpower-task-worker".to_string())
            .spawn(move || worker(thread_stop))
            .map_err(|error| format!("Démarrage du worker impossible : {error}"))?;

        *slot = Some(RunningWorker { stop, join });
        Ok(())
    }

    /// Demande l'arrêt et attend la fin de l'itération en cours avant le verrouillage.
    pub(crate) fn stop(&self) -> Result<(), String> {
        let worker = self
            .running
            .lock()
            .map_err(|_| "Le contrôleur du worker est indisponible.".to_string())?
            .take();

        if let Some(worker) = worker {
            worker.stop.store(true, Ordering::Release);
            worker
                .join
                .join()
                .map_err(|_| "Le worker de tâches s'est arrêté anormalement.".to_string())?;
        }
        Ok(())
    }
}

fn run_worker(app: AppHandle, stop: Arc<AtomicBool>) {
    while !stop.load(Ordering::Acquire) {
        if let Err(error) = process_due_tasks(&app, &stop) {
            eprintln!("Erreur du worker de tâches : {error}");
        }
        if stop.load(Ordering::Acquire) {
            break;
        }
        publish_ready_reminders(&app);

        for _ in 0..POLL_TICKS {
            if stop.load(Ordering::Acquire) {
                break;
            }
            thread::sleep(POLL_INTERVAL);
        }
    }
}

fn process_due_tasks(app: &AppHandle, stop: &AtomicBool) -> Result<(), String> {
    if stop.load(Ordering::Acquire) {
        return Ok(());
    }

    // Si le coffre n'est pas accessible, aucune tâche n'est réservée ou consommée.
    let mut connection = match app.state::<AppState>().connection() {
        Ok(connection) => connection,
        Err(error) if error == "La base est verrouillée." => return Ok(()),
        Err(error) => return Err(error),
    };
    let registry = app.state::<TaskHandlerRegistry>();

    for _ in 0..MAX_EXECUTIONS_PER_PASS {
        if stop.load(Ordering::Acquire) {
            break;
        }
        match TaskWorker::run_once_for_registered_handlers(
            &mut connection,
            &registry,
            Utc::now(),
            TaskWorkerConfig::default(),
        )
        .map_err(|error| error.to_string())?
        {
            WorkerOutcome::Idle => break,
            WorkerOutcome::Succeeded { execution_id, .. } => {
                app.state::<ReminderNotificationQueue>()
                    .mark_completed(&execution_id)?;
            }
            WorkerOutcome::RetryScheduled { error, .. } | WorkerOutcome::Failed { error, .. } => {
                eprintln!("Une tâche a échoué : {error}");
            }
        }
    }
    Ok(())
}

fn publish_ready_reminders(app: &AppHandle) {
    let queue = app.state::<ReminderNotificationQueue>();
    let events = match queue.ready_events() {
        Ok(events) => events,
        Err(error) => {
            eprintln!("Impossible de lire la file de rappels : {error}");
            return;
        }
    };

    for event in events {
        match app.emit(TASK_REMINDER_EVENT, event.clone()) {
            Ok(()) => {
                if let Err(error) = queue.remove(&event.execution_id) {
                    eprintln!("Impossible de clôturer un rappel publié : {error}");
                }
            }
            Err(error) => eprintln!("Publication du rappel impossible : {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reminder_uses_a_non_empty_payload_message_or_falls_back_to_title() {
        assert_eq!(
            reminder_message(
                "Vérifier le bail",
                &serde_json::json!({"message": "  Relancer le locataire  "})
            ),
            "Relancer le locataire"
        );
        assert_eq!(
            reminder_message("Vérifier le bail", &serde_json::json!({"message": "  "})),
            "Vérifier le bail"
        );
        assert_eq!(
            reminder_message("Vérifier le bail", &serde_json::json!({})),
            "Vérifier le bail"
        );
    }

    #[test]
    fn reminder_message_is_bounded() {
        let long_message = "x".repeat(MAX_REMINDER_MESSAGE_CHARS + 20);
        assert_eq!(
            reminder_message("Rappel", &serde_json::json!({"message": long_message}))
                .chars()
                .count(),
            MAX_REMINDER_MESSAGE_CHARS
        );
    }

    #[test]
    fn reminder_queue_deduplicates_a_retry_by_execution_id() {
        let queue = ReminderNotificationQueue::default();
        let original = TaskReminderEvent {
            execution_id: "exec-1".to_string(),
            title: "Titre".to_string(),
            message: "Premier contenu".to_string(),
            scheduled_for: "2026-10-13T10:00:00Z".to_string(),
        };
        queue.enqueue(original.clone()).unwrap();
        queue
            .enqueue(TaskReminderEvent {
                message: "Contenu réessayé".to_string(),
                ..original
            })
            .unwrap();
        assert!(queue.ready_events().unwrap().is_empty());
        queue.mark_completed("exec-1").unwrap();
        let ready = queue.ready_events().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].message, "Contenu réessayé");
        queue.remove("exec-1").unwrap();
        assert!(queue.ready_events().unwrap().is_empty());
    }

    #[test]
    fn worker_controller_ignores_duplicate_start_and_can_restart_after_stop() {
        let controller = TaskWorkerController::default();

        let (first_started_tx, first_started_rx) = std::sync::mpsc::channel();
        controller
            .start_with(move |stop| {
                first_started_tx.send(()).unwrap();
                while !stop.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(1));
                }
            })
            .unwrap();
        first_started_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("le premier worker aurait dû démarrer");

        let (duplicate_tx, duplicate_rx) = std::sync::mpsc::channel();
        controller
            .start_with(move |stop| {
                duplicate_tx.send(()).unwrap();
                while !stop.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(1));
                }
            })
            .unwrap();
        assert!(matches!(
            duplicate_rx.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Disconnected)
        ));

        controller.stop().unwrap();

        let (restart_tx, restart_rx) = std::sync::mpsc::channel();
        controller
            .start_with(move |stop| {
                restart_tx.send(()).unwrap();
                while !stop.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(1));
                }
            })
            .unwrap();
        restart_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("le worker aurait dû redémarrer après stop");
        controller.stop().unwrap();
    }
}
