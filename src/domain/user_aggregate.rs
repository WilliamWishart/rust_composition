use crate::events::{DomainEvent, UserRegisteredEvent};
use std::sync::Arc;
use std::fmt;

/// User Aggregate - AggregateRoot pattern from m-r
/// Encapsulates both state and business logic
/// Accumulates uncommitted events until explicitly committed
#[derive(Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub version: i32, // Tracks event version for optimistic locking
    uncommitted_changes: Vec<Arc<dyn DomainEvent>>, // Events not yet persisted
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("version", &self.version)
            .field("uncommitted_changes", &format!("<{} events>", self.uncommitted_changes.len()))
            .finish()
    }
}



impl User {
    /// Create a new user (from command in CQRS write model)
    pub fn new(id: u32, name: String) -> Self {
        let mut user = User {
            id,
            name: String::new(),
            version: -1, // New aggregates start at version -1 (no events yet)
            uncommitted_changes: Vec::new(),
        };

        // Create the registration event
        let event = UserRegisteredEvent::new(id, name);
        
        // Apply the event to self (updates state)
        user.apply_user_registered_event(&event);
        
        // Record the uncommitted change (as Arc)
        user.uncommitted_changes.push(Arc::new(event.clone()));

        user
    }

    /// Apply UserRegisteredEvent - updates aggregate state
    /// This is called both when replaying history and when handling new commands
    fn apply_user_registered_event(&mut self, event: &UserRegisteredEvent) {
        self.id = event.user_id;
        self.name = event.name.clone();
    }

    /// Load aggregate from event history (event sourcing reconstruction)
    pub fn load_from_history(events: Vec<Arc<dyn DomainEvent>>) -> Result<Self, String> {
        let mut user = User {
            id: 0,
            name: String::new(),
            version: -1,
            uncommitted_changes: Vec::new(),
        };

        for (index, event) in events.iter().enumerate() {
            // Match event type and apply
            if let Some(reg_event) = event.as_any().downcast_ref::<UserRegisteredEvent>() {
                user.apply_user_registered_event(reg_event);
                user.version = index as i32; // Version increments with each event
            }
        }

        Ok(user)
    }

    /// Get uncommitted changes (for Repository.Save)
    pub fn get_uncommitted_changes(&self) -> Vec<Arc<dyn DomainEvent>> {
        self.uncommitted_changes.clone()
    }

    /// Mark changes as committed (called after successful persist)
    pub fn mark_changes_as_committed(&mut self) {
        self.uncommitted_changes.clear();
    }
}
