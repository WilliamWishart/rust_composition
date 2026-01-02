use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::events::UserEvent;
use crate::events::event_bus::{EventHandler, HandlerPriority};
use async_trait::async_trait;

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
    fn handle_user_registered(&self, user_id: u32, name: String, timestamp: i64) {
        let user = UserReadModel {
            id: user_id,
            name,
            created_at: timestamp,
        };
        self.users.lock().unwrap().insert(user_id, user);
    }

    /// Handle user renamed event
    fn handle_user_renamed(&self, user_id: u32, new_name: String, _timestamp: i64) {
        let mut users = self.users.lock().unwrap();
        if let Some(user) = users.get_mut(&user_id) {
            user.name = new_name;
        }
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

/// Handles<T> pattern from m-r reference
/// Strongly-typed event handlers for projections
pub trait Handles<T> {
    fn handle(&self, event: &T);
}

/// TypedUserProjectionHandler - Implements m-r Handles<T> pattern
/// This ensures type safety and strong coupling to specific event types
/// No runtime casting needed - compiler guarantees correctness
pub struct TypedUserProjectionHandler {
    projection: UserProjection,
}

impl TypedUserProjectionHandler {
    pub fn new(projection: UserProjection) -> Self {
        TypedUserProjectionHandler { projection }
    }

    pub fn get_projection(&self) -> UserProjection {
        self.projection.clone()
    }
}

/// Implements Handles<UserEvent> - strongly typed with pattern matching
impl Handles<UserEvent> for TypedUserProjectionHandler {
    fn handle(&self, event: &UserEvent) {
        // Pattern match on the event enum - compiler ensures all variants handled
        match event {
            UserEvent::Registered {
                user_id,
                name,
                timestamp,
            } => {
                self.projection
                    .handle_user_registered(*user_id, name.clone(), *timestamp);
            }
            UserEvent::Renamed {
                user_id,
                new_name,
                timestamp,
            } => {
                self.projection
                    .handle_user_renamed(*user_id, new_name.clone(), *timestamp);
            }
        }
    }
}

/// Implements EventHandler trait for EventBus integration (async)
/// Allows TypedUserProjectionHandler to be registered as an EventBus subscriber
/// Marked as High priority to ensure projections stay synchronized with event store
#[async_trait]
impl EventHandler for TypedUserProjectionHandler {
    async fn handle_event(&self, event: &UserEvent) -> Result<(), Box<dyn std::error::Error>> {
        // Pattern match on the event enum - compiler ensures all variants handled
        match event {
            UserEvent::Registered {
                user_id,
                name,
                timestamp,
            } => {
                self.projection
                    .handle_user_registered(*user_id, name.clone(), *timestamp);
            }
            UserEvent::Renamed {
                user_id,
                new_name,
                timestamp,
            } => {
                self.projection
                    .handle_user_renamed(*user_id, new_name.clone(), *timestamp);
            }
        }
        Ok(())
    }
    
    fn priority(&self) -> HandlerPriority {
        // Projections are High priority to ensure consistency
        HandlerPriority::High
    }
    
    fn name(&self) -> &str {
        "TypedUserProjectionHandler"
    }
}


