use std::sync::{Arc, Mutex};
use crate::events::DomainEvent;

/// EventStore - Immutable event log
/// Implements Event Sourcing pattern - stores complete history of events
/// This is the source of truth for the domain
pub struct EventStore {
    events: Arc<Mutex<Vec<Arc<dyn DomainEvent>>>>,
}

impl EventStore {
    pub fn new() -> Self {
        EventStore {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Append an event to the store
    /// Events are immutable - never modified, only appended
    pub fn append(&self, event: Arc<dyn DomainEvent>) {
        let mut events = self.events.lock().unwrap();
        events.push(event);
    }

    /// Retrieve all events for an aggregate
    pub fn get_events(&self, aggregate_id: &str) -> Vec<Arc<dyn DomainEvent>> {
        let events = self.events.lock().unwrap();
        events
            .iter()
            .filter(|e| e.aggregate_id() == aggregate_id)
            .cloned()
            .collect()
    }

    /// Retrieve all events in order
    pub fn get_all_events(&self) -> Vec<Arc<dyn DomainEvent>> {
        self.events.lock().unwrap().clone()
    }

    /// Get the total number of events
    pub fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }
}

impl Default for EventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventStore {
    fn clone(&self) -> Self {
        EventStore {
            events: Arc::clone(&self.events),
        }
    }
}
