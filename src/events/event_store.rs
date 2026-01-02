use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::events::UserEvent;

/// DeadLetterQueueEntry - Record of failed events for inspection and replay
#[derive(Debug, Clone)]
pub struct DeadLetterQueueEntry {
    pub aggregate_id: u32,
    pub event: UserEvent,
    pub error_message: String,
    pub failure_count: usize,
    pub last_failed_at: chrono::DateTime<chrono::Utc>,
}

/// EventStore - Immutable event log with dead letter queue support
/// Stores all domain events (facts) - the single source of truth
/// Events are never modified, only appended
/// Failed events are captured in dead letter queue for inspection and replay
pub struct EventStore {
    events: Arc<Mutex<HashMap<u32, Vec<UserEvent>>>>, // Keyed by aggregate ID
    dead_letter_queue: Arc<Mutex<Vec<DeadLetterQueueEntry>>>, // Failed events
}

impl EventStore {
    pub fn new() -> Self {
        EventStore {
            events: Arc::new(Mutex::new(HashMap::new())),
            dead_letter_queue: Arc::new(Mutex::new(Vec::new())),
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
    
    /// Record a failed event in the dead letter queue
    pub fn record_failed_event(
        &self,
        aggregate_id: u32,
        event: UserEvent,
        error_message: String,
    ) {
        let mut dlq = self.dead_letter_queue.lock().unwrap();
        
        // Check if event already exists in DLQ
        if let Some(entry) = dlq.iter_mut().find(|e| e.aggregate_id == aggregate_id && e.event == event) {
            entry.failure_count += 1;
            entry.last_failed_at = chrono::Utc::now();
        } else {
            // New failed event
            dlq.push(DeadLetterQueueEntry {
                aggregate_id,
                event,
                error_message,
                failure_count: 1,
                last_failed_at: chrono::Utc::now(),
            });
        }
    }
    
    /// Retrieve all events from the dead letter queue
    pub fn get_dead_letter_queue(&self) -> Vec<DeadLetterQueueEntry> {
        self.dead_letter_queue.lock().unwrap().clone()
    }
    
    /// Clear a failed event from the dead letter queue (after successful replay)
    pub fn remove_from_dlq(&self, aggregate_id: u32, event: &UserEvent) {
        let mut dlq = self.dead_letter_queue.lock().unwrap();
        dlq.retain(|e| !(e.aggregate_id == aggregate_id && &e.event == event));
    }
    
    /// Get dead letter queue size
    pub fn dlq_size(&self) -> usize {
        self.dead_letter_queue.lock().unwrap().len()
    }

    /// Find a user by name across all events
    /// Reconstructs the latest state of all users and searches by name
    /// This is a convenience method - production systems would use a projection index
    pub fn find_user_by_name(&self, name: &str) -> crate::infrastructure::DomainResult<Option<crate::domain::User>> {
        let events = self.events.lock().unwrap();
        
        // Build a map of latest user state by scanning all events
        let mut users: HashMap<u32, crate::domain::User> = HashMap::new();
        
        for aggregate_events in events.values() {
            if let Ok(user) = crate::domain::User::load_from_history(aggregate_events.clone()) {
                users.insert(user.id, user);
            }
        }
        
        // Search for user by name (case-sensitive)
        Ok(users.values().find(|u| u.name == name).cloned())
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
            dead_letter_queue: Arc::clone(&self.dead_letter_queue),
        }
    }
}
