use std::sync::Arc;
use crate::commands::{RegisterUserCommand, RenameUserCommand};
use crate::events::EventBus;
use crate::infrastructure::{Logger, DomainResult};
use crate::domain::{Repository, IRepository, User};

/// UserCommandHandler - CQRS write side handler
/// Processes commands through aggregates (not directly to events)
/// Follows m-r pattern: Command → Aggregate → Events → EventStore
/// All methods are async - uses app's tokio runtime for proper concurrency
pub struct UserCommandHandler {
    repository: Arc<Repository>,
    event_bus: EventBus,
    logger: Arc<dyn Logger>,
}

impl UserCommandHandler {
    pub fn new(
        repository: Arc<Repository>,
        event_bus: EventBus,
        logger: Arc<dyn Logger>,
    ) -> Self {
        UserCommandHandler {
            repository,
            event_bus,
            logger,
        }
    }

    /// Execute RegisterUserCommand (asynchronous)
    /// Publishes events with proper error handling and awaiting
    pub async fn handle_register_user(&self, command: RegisterUserCommand) -> DomainResult<()> {
        self.logger.info(&format!(
            "Processing command: RegisterUser(id={}, name={})",
            command.user_id, command.name
        ));

        // Create aggregate - this applies events internally
        let user = User::new(command.user_id, command.name.clone());

        // Save through repository (handles optimistic locking, persistence)
        // Returns the events that were saved
        let saved_events = self.repository.save(&user, -1)?; // -1 indicates new aggregate

        // Publish events with proper error handling and awaiting
        for event in saved_events {
            match self.event_bus.publish(&event).await {
                Ok(errors) if errors.is_empty() => {},
                Ok(errors) => {
                    for err in errors {
                        self.logger.warn(&format!("Non-critical handler error: {}", err));
                    }
                }
                Err(e) => {
                    self.logger.error(&format!("Critical error publishing event: {}", e));
                    return Err(crate::infrastructure::DomainError::RepositoryError(
                        format!("Failed to publish event: {}", e)
                    ));
                }
            }
        }

        self.logger
            .info(&format!("User {} registered successfully", command.user_id));

        Ok(())
    }

    /// Execute RenameUserCommand (asynchronous)
    /// Publishes events with proper error handling and awaiting
    pub async fn handle_rename_user(&self, command: RenameUserCommand) -> DomainResult<()> {
        self.logger.info(&format!(
            "Processing command: RenameUser(id={}, new_name={})",
            command.user_id, command.new_name
        ));

        // Load aggregate from history (event sourcing reconstruction)
        let mut user = self.repository.get_by_id(command.user_id)?;

        // Apply the rename to the aggregate
        user.rename(command.new_name.clone());

        // Save through repository (handles optimistic locking, persistence)
        let saved_events = self.repository.save(&user, user.version)?;

        // Publish events with proper error handling and awaiting
        for event in saved_events {
            match self.event_bus.publish(&event).await {
                Ok(errors) if errors.is_empty() => {},
                Ok(errors) => {
                    for err in errors {
                        self.logger.warn(&format!("Non-critical handler error: {}", err));
                    }
                }
                Err(e) => {
                    self.logger.error(&format!("Critical error publishing event: {}", e));
                    return Err(crate::infrastructure::DomainError::RepositoryError(
                        format!("Failed to publish event: {}", e)
                    ));
                }
            }
        }

        self.logger
            .info(&format!("User {} renamed successfully", command.user_id));

        Ok(())
    }
}
