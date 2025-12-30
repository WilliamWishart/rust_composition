use std::sync::Arc;
use crate::infrastructure::Logger;
use crate::domain::UserRepository;

/// UserService - Application Service
/// Orchestrates user-related business logic
/// Uses cases are implemented here
pub struct UserService {
    repository: Arc<UserRepository>,
    logger: Arc<dyn Logger>,
}

impl UserService {
    pub fn new(repository: Arc<UserRepository>, logger: Arc<dyn Logger>) -> Self {
        UserService { repository, logger }
    }

    /// Use case: Register a new user
    pub fn register_user(&self, id: u32, name: &str) {
        self.logger.log("=== Starting user registration ===");
        self.repository.save_user(id, name);
        self.logger.log("=== User registration complete ===");
    }

    /// Use case: Retrieve user information
    pub fn get_user(&self, id: u32) -> String {
        self.repository.get_user(id)
    }
}
