use std::sync::Arc;
use crate::infrastructure::{Logger, Database};

/// UserRepository - Data Access Layer
/// Handles all user-related database operations
pub struct UserRepository {
    logger: Arc<dyn Logger>,
    db: Arc<dyn Database>,
}

impl UserRepository {
    pub fn new(logger: Arc<dyn Logger>, db: Arc<dyn Database>) -> Self {
        UserRepository { logger, db }
    }

    pub fn get_user(&self, user_id: u32) -> String {
        self.logger.log(&format!("Fetching user {}", user_id));
        self.db.query(&format!("SELECT * FROM users WHERE id = {}", user_id))
    }

    pub fn save_user(&self, user_id: u32, name: &str) {
        self.logger.log(&format!("Saving user {} with name '{}'", user_id, name));
        self.db.query(&format!(
            "INSERT INTO users (id, name) VALUES ({}, '{}')",
            user_id, name
        ));
    }
}
