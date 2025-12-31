use std::sync::{Arc, Mutex};
use crate::events::UserEvent;

/// EventBus - Implements pub/sub for domain events
/// Publishes events to subscribers for eventual consistency
/// Uses strongly-typed events (enum-based) instead of trait objects
#[derive(Clone)]
pub struct EventBus {
    subscribers: Arc<Mutex<Vec<Arc<dyn EventHandler>>>>,
}

/// EventHandler - Trait for components that handle domain events
pub trait EventHandler: Send + Sync {
    fn handle_event(&self, event: &UserEvent);
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a subscriber to receive events
    pub fn subscribe<H: EventHandler + 'static>(&self, handler: Arc<H>) {
        self.subscribers.lock().unwrap().push(handler as Arc<dyn EventHandler>);
    }

    /// Publish a UserEvent - notify all registered subscribers (eventually consistent)
    pub fn publish(&self, event: &UserEvent) {
        let subscribers = self.subscribers.lock().unwrap();
        for subscriber in subscribers.iter() {
            subscriber.handle_event(event);
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
