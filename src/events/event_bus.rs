use std::sync::{Arc, Mutex};
use crate::events::DomainEvent;
use std::collections::HashMap;

/// EventHandler - Trait for handling domain events
/// Handlers react to events (event subscribers/listeners)
pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &dyn DomainEvent);
    fn event_type(&self) -> &str;
}

/// EventBus - Implements pub/sub for domain events
/// Handles eventual consistency by notifying subscribers of events
/// This enables async event processing and projections
pub struct EventBus {
    handlers: Arc<Mutex<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Subscribe to events of a specific type
    pub fn subscribe(&self, event_type: &str, handler: Arc<dyn EventHandler>) {
        let mut handlers = self.handlers.lock().unwrap();
        handlers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Publish an event - notify all subscribers (eventually consistent)
    pub fn publish(&self, event: &dyn DomainEvent) {
        let handlers = self.handlers.lock().unwrap();
        if let Some(event_handlers) = handlers.get(event.event_type()) {
            for handler in event_handlers {
                // In a real system, these would be dispatched asynchronously
                // For now, we handle synchronously but the pattern is clear
                handler.handle(event);
            }
        }
    }

    /// Get all registered event types
    pub fn registered_events(&self) -> Vec<String> {
        self.handlers
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        EventBus {
            handlers: Arc::clone(&self.handlers),
        }
    }
}
