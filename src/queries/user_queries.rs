use crate::events::UserProjection;

/// UserQuery - CQRS read side handler
/// Queries retrieve data from projections (read models)
/// Unlike commands, queries never modify state
pub struct UserQuery {
    projection: UserProjection,
}

impl UserQuery {
    pub fn new(projection: UserProjection) -> Self {
        UserQuery { projection }
    }

    /// Get user by ID from the read model
    pub fn get_user(&self, user_id: u32) -> Option<String> {
        self.projection
            .get_user(user_id)
            .map(|user| format!("{} (ID: {})", user.name, user.id))
    }

    /// Get all users from the read model
    pub fn get_all_users(&self) -> Vec<String> {
        self.projection
            .get_all_users()
            .into_iter()
            .map(|user| format!("{} (ID: {})", user.name, user.id))
            .collect()
    }

    /// Get user count
    pub fn get_user_count(&self) -> usize {
        self.projection.get_all_users().len()
    }
}
