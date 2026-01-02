use std::sync::Arc;
use crate::infrastructure::{Logger, ConsoleLogger};

/// AppBuilder - Composition Root
/// Centralizes dependency wiring and application composition
/// This is the only place that knows about concrete implementations
///
/// The proper composition pattern for CQRS:
/// 1. Setup infrastructure (Logger, EventStore)
/// 2. Create write side: Repository + EventStore → CommandHandler
/// 3. Create read side: UserProjection + EventBus → UserQuery
/// 4. Wire them together via EventBus
pub struct AppBuilder {
    logger: Arc<dyn Logger>,
}

impl AppBuilder {
    /// Create a new builder with production defaults
    pub fn new() -> Self {
        AppBuilder {
            logger: Arc::new(ConsoleLogger::default()),
        }
    }

    /// Replace the logger implementation
    pub fn with_logger(mut self, logger: Arc<dyn Logger>) -> Self {
        self.logger = logger;
        self
    }

    /// Get the logger (for wiring CQRS components)
    pub fn get_logger(&self) -> Arc<dyn Logger> {
        self.logger.clone()
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}
