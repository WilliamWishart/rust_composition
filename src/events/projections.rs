use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::events::UserRegisteredEvent;

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

/// Handles<T> pattern from m-r reference
/// Strongly-typed event handlers for projections
pub trait Handles<T> {
    fn handle(&self, event: &T);
}

/// TypedUserProjectionHandler - Implements m-r Handles<T> pattern
/// This ensures type safety and strong coupling to specific event types
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

/// Implements Handles<UserRegisteredEvent> - strong typing from m-r
impl Handles<UserRegisteredEvent> for TypedUserProjectionHandler {
    fn handle(&self, event: &UserRegisteredEvent) {
        self.projection.handle_user_registered(event);
    }
}

