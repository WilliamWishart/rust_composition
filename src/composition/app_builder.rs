use std::sync::Arc;
use crate::infrastructure::{Logger, Database, ConsoleLogger, MockDatabase};
use crate::domain::UserRepository;
use crate::application::UserService;

/// AppBuilder - Composition Root
/// Centralizes dependency wiring and application composition
/// This is the only place that knows about concrete implementations
pub struct AppBuilder {
    logger: Arc<dyn Logger>,
    database: Arc<dyn Database>,
}

impl AppBuilder {
    /// Create a new builder with production defaults
    pub fn new() -> Self {
        AppBuilder {
            logger: Arc::new(ConsoleLogger),
            database: Arc::new(MockDatabase),
        }
    }

    /// Replace the logger implementation
    pub fn with_logger(mut self, logger: Arc<dyn Logger>) -> Self {
        self.logger = logger;
        self
    }

    /// Replace the database implementation
    pub fn with_database(mut self, database: Arc<dyn Database>) -> Self {
        self.database = database;
        self
    }

    /// Build and return the fully wired UserService
    pub fn build_user_service(self) -> UserService {
        let repository = Arc::new(UserRepository::new(
            self.logger.clone(),
            self.database.clone(),
        ));

        UserService::new(repository, self.logger)
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}
