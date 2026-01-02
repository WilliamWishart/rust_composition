use std::sync::Arc;
use infrastructure::{Logger, ConsoleLogger, LogLevel};

/// AppBuilder - Composition Root
/// Centralizes dependency wiring and application composition
/// This is the only place that knows about concrete implementations
///
/// Example usage:
/// ```ignore
/// let builder = AppBuilder::new();
/// let logger = builder.get_logger();
/// ```
pub struct AppBuilder {
    logger: Arc<dyn Logger>,
}

impl AppBuilder {
    /// Create a new builder with production defaults
    pub fn new() -> Self {
        AppBuilder {
            logger: Arc::new(ConsoleLogger::new(LogLevel::Info)),
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
