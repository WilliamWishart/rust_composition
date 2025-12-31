use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::events::UserEvent;

/// EventStore - Immutable event log
/// Stores all domain events (facts) - the single source of truth
/// Events are never modified, only appended
pub struct EventStore {
    events: Arc<Mutex<HashMap<u32, Vec<UserEvent>>>>, // Keyed by aggregate ID
}

impl EventStore {
    pub fn new() -> Self {
        EventStore {
            events: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Append an event to the store
    /// Events are immutable - never modified, only appended
    pub fn append(&self, aggregate_id: u32, event: UserEvent) {
        let mut events = self.events.lock().unwrap();
        events
            .entry(aggregate_id)
            .or_insert_with(Vec::new)
            .push(event);
    }

    /// Retrieve all events for an aggregate
    pub fn get_events(&self, aggregate_id: u32) -> Vec<UserEvent> {
        let events = self.events.lock().unwrap();
        events
            .get(&aggregate_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Retrieve all events in order
    pub fn get_all_events(&self) -> Vec<UserEvent> {
        let events = self.events.lock().unwrap();
        events
            .values()
            .flat_map(|v| v.iter().cloned())
            .collect()
    }

    /// Get the total number of events
    pub fn event_count(&self) -> usize {
        self.events.lock().unwrap().values().map(|v| v.len()).sum()
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
