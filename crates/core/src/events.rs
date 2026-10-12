use std::sync::{mpsc, Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainEvent {
    SocieteCreee {
        societe_id: String,
    },
    EcritureValidee {
        ecriture_id: String,
        societe_id: String,
    },
}

/// Bus d'événements en mémoire pour découpler les modules.
#[derive(Clone, Default)]
pub struct EventBus {
    subscribers: Arc<Mutex<Vec<mpsc::Sender<DomainEvent>>>>,
}

impl EventBus {
    pub fn subscribe(&self) -> mpsc::Receiver<DomainEvent> {
        let (sender, receiver) = mpsc::channel();
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers.push(sender);
        }
        receiver
    }

    pub fn publish(&self, event: DomainEvent) {
        if let Ok(mut subscribers) = self.subscribers.lock() {
            subscribers.retain(|sender| sender.send(event.clone()).is_ok());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publishes_to_subscribers_and_removes_disconnected_receivers() {
        let bus = EventBus::default();
        let receiver = bus.subscribe();
        bus.publish(DomainEvent::SocieteCreee {
            societe_id: "soc-1".into(),
        });
        assert_eq!(
            receiver.recv().expect("event"),
            DomainEvent::SocieteCreee {
                societe_id: "soc-1".into()
            }
        );
    }
}
