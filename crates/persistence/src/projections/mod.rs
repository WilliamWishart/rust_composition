// Projections - Read models built from domain events
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use domain::events::UserEvent;

/// UserReadModel - Denormalized data for queries
#[derive(Debug, Clone)]
pub struct UserReadModel {
    pub id: u32,
    pub name: String,
    pub created_at: i64,
}

/// UserProjection - Builds and maintains the read model
pub struct UserProjection {
    users: Arc<Mutex<HashMap<u32, UserReadModel>>>,
}

impl UserProjection {
    pub fn new() -> Self {
        UserProjection {
            users: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_user(&self, user_id: u32) -> Option<UserReadModel> {
        self.users.lock().unwrap().get(&user_id).cloned()
    }

    pub fn get_all_users(&self) -> Vec<UserReadModel> {
        self.users
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    fn handle_user_registered(&self, user_id: u32, name: String, timestamp: i64) {
        let user = UserReadModel {
            id: user_id,
            name,
            created_at: timestamp,
        };
        self.users.lock().unwrap().insert(user_id, user);
    }

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

/// Handles<T> - Strongly-typed event handlers
pub trait Handles<T> {
    fn handle(&self, event: &T);
}

/// TypedUserProjectionHandler - Implements m-r Handles<T> pattern
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

impl Handles<UserEvent> for TypedUserProjectionHandler {
    fn handle(&self, event: &UserEvent) {
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
