use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use domain::events::UserEvent;

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
pub struct EventStore {
    events: Arc<Mutex<HashMap<u32, Vec<UserEvent>>>>,
    dead_letter_queue: Arc<Mutex<Vec<DeadLetterQueueEntry>>>,
}

impl EventStore {
    pub fn new() -> Self {
        EventStore {
            events: Arc::new(Mutex::new(HashMap::new())),
            dead_letter_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn append(&self, aggregate_id: u32, event: UserEvent) {
        let mut events = self.events.lock().unwrap();
        events
            .entry(aggregate_id)
            .or_insert_with(Vec::new)
            .push(event);
    }

    pub fn get_events(&self, aggregate_id: u32) -> Vec<UserEvent> {
        let events = self.events.lock().unwrap();
        events
            .get(&aggregate_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_all_events(&self) -> Vec<UserEvent> {
        let events = self.events.lock().unwrap();
        events
            .values()
            .flat_map(|v| v.iter().cloned())
            .collect()
    }

    pub fn event_count(&self) -> usize {
        self.events.lock().unwrap().values().map(|v| v.len()).sum()
    }
    
    pub fn record_failed_event(
        &self,
        aggregate_id: u32,
        event: UserEvent,
        error_message: String,
    ) {
        let mut dlq = self.dead_letter_queue.lock().unwrap();
        
        if let Some(entry) = dlq.iter_mut().find(|e| e.aggregate_id == aggregate_id && e.event == event) {
            entry.failure_count += 1;
            entry.last_failed_at = chrono::Utc::now();
        } else {
            dlq.push(DeadLetterQueueEntry {
                aggregate_id,
                event,
                error_message,
                failure_count: 1,
                last_failed_at: chrono::Utc::now(),
            });
        }
    }
    
    pub fn get_dead_letter_queue(&self) -> Vec<DeadLetterQueueEntry> {
        self.dead_letter_queue.lock().unwrap().clone()
    }
    
    pub fn remove_from_dlq(&self, aggregate_id: u32, event: &UserEvent) {
        let mut dlq = self.dead_letter_queue.lock().unwrap();
        dlq.retain(|e| !(e.aggregate_id == aggregate_id && &e.event == event));
    }
    
    pub fn dlq_size(&self) -> usize {
        self.dead_letter_queue.lock().unwrap().len()
    }

    pub fn find_user_by_name(&self, name: &str) -> domain::errors::DomainResult<Option<domain::User>> {
        let events = self.events.lock().unwrap();
        
        let mut users: HashMap<u32, domain::User> = HashMap::new();
        
        for aggregate_events in events.values() {
            if let Ok(user) = domain::User::load_from_history(aggregate_events.clone()) {
                users.insert(user.id, user);
            }
        }
        
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
