use crate::events::UserEvent;

/// EventBus - Implements pub/sub for domain events
/// Publishes events to subscribers for eventual consistency
/// Uses strongly-typed events (enum-based) instead of trait objects
pub struct EventBus;

impl EventBus {
    pub fn new() -> Self {
        EventBus
    }

    /// Publish a UserEvent - notify all subscribers (eventually consistent)
    /// In a production system, this would dispatch to async handlers
    pub fn publish(&self, _event: &UserEvent) {
        // Event is published - subscribers would be notified here
        // For the demo, we handle event updates manually in main.rs
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        EventBus
    }
}
