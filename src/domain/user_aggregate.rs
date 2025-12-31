use crate::events::UserEvent;
use std::fmt;

/// User Aggregate - AggregateRoot pattern from m-r
/// Encapsulates both state and business logic
/// Uses enum-based events instead of trait objects - pure Rust idiom
#[derive(Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub version: i32, // Tracks event version for optimistic locking
    uncommitted_changes: Vec<UserEvent>, // Concrete events, no Arc, no trait objects!
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
        let event = UserEvent::Registered {
            user_id: id,
            name,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        // Apply the event to self (updates state)
        user.apply_event(&event);

        // Record the uncommitted change
        user.uncommitted_changes.push(event);

        user
    }

    /// Apply event - updates aggregate state
    /// Uses pattern matching on the enum - compiler enforces all cases handled
    fn apply_event(&mut self, event: &UserEvent) {
        match event {
            UserEvent::Registered {
                user_id,
                name,
                timestamp: _,
            } => {
                self.id = *user_id;
                self.name = name.clone();
            }
            UserEvent::Renamed {
                user_id: _,
                new_name,
                timestamp: _,
            } => {
                self.name = new_name.clone();
            }
        }
    }

    /// Load aggregate from event history (event sourcing reconstruction)
    /// Pure pattern matching - no downcasting, no runtime type checks needed!
    pub fn load_from_history(events: Vec<UserEvent>) -> Result<Self, String> {
        let mut user = User {
            id: 0,
            name: String::new(),
            version: -1,
            uncommitted_changes: Vec::new(),
        };

        for (index, event) in events.iter().enumerate() {
            // Compiler guarantees we handle all UserEvent variants
            user.apply_event(event);
            user.version = index as i32; // Version increments with each event
        }

        Ok(user)
    }

    /// Get uncommitted changes (for Repository.Save)
    pub fn get_uncommitted_changes(&self) -> Vec<UserEvent> {
        self.uncommitted_changes.clone()
    }

    /// Mark changes as committed (called after successful persist)
    pub fn mark_changes_as_committed(&mut self) {
        self.uncommitted_changes.clear();
    }

    /// Rename the user (domain command)
    pub fn rename(&mut self, new_name: String) {
        let event = UserEvent::Renamed {
            user_id: self.id,
            new_name,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        // Apply the event to self (updates state)
        self.apply_event(&event);

        // Record the uncommitted change
        self.uncommitted_changes.push(event);
    }
}
