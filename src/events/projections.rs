use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::events::{DomainEvent, EventHandler, UserRegisteredEvent};

/// UserProjection - Read model built from domain events
/// Projections are eventually consistent - they're built by replaying events
/// This is the CQRS read side
#[derive(Debug, Clone)]
pub struct UserReadModel {
    pub id: u32,
    pub name: String,
    pub created_at: i64,
}

pub struct UserProjection {
    users: Arc<Mutex<HashMap<u32, UserReadModel>>>,
}

impl UserProjection {
    pub fn new() -> Self {
        UserProjection {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a user from the read model
    pub fn get_user(&self, user_id: u32) -> Option<UserReadModel> {
        self.users.lock().unwrap().get(&user_id).cloned()
    }

    /// Get all users from the read model
    pub fn get_all_users(&self) -> Vec<UserReadModel> {
        self.users
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Update the projection based on events
    fn handle_user_registered(&self, event: &UserRegisteredEvent) {
        let user = UserReadModel {
            id: event.user_id,
            name: event.name.clone(),
            created_at: event.timestamp,
        };
        self.users.lock().unwrap().insert(event.user_id, user);
    }
}

impl Default for UserProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for UserProjection {
    fn clone(&self) -> Self {
        UserProjection {
            users: Arc::clone(&self.users),
        }
    }
}

/// UserProjectionHandler - Subscribes to user events and updates the read model
/// This handler processes events and maintains eventual consistency
pub struct UserProjectionHandler {
    projection: UserProjection,
}

impl UserProjectionHandler {
    pub fn new(projection: UserProjection) -> Self {
        UserProjectionHandler { projection }
    }

    pub fn get_projection(&self) -> UserProjection {
        self.projection.clone()
    }
}

impl EventHandler for UserProjectionHandler {
    fn handle(&self, event: &dyn DomainEvent) {
        if event.event_type() == "UserRegistered" {
            // In a real implementation, we'd properly downcast here
            // For this example, we reconstruct from the event data
            let _ = event.aggregate_id().parse::<u32>().ok();
            // We need access to the actual UserRegisteredEvent
            // This is a limitation of trait objects - we'd handle this better in production
            // by storing the serialized event and deserializing
        }
    }

    fn event_type(&self) -> &str {
        "UserRegistered"
    }
}

// Better approach: Create a strongly-typed projection handler
pub struct TypedUserProjectionHandler {
    projection: UserProjection,
}

impl TypedUserProjectionHandler {
    pub fn new(projection: UserProjection) -> Self {
        TypedUserProjectionHandler { projection }
    }

    pub fn handle_user_registered(&self, event: &UserRegisteredEvent) {
        self.projection.handle_user_registered(event);
    }

    pub fn get_projection(&self) -> UserProjection {
        self.projection.clone()
    }
}
