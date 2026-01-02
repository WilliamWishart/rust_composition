use std::sync::Arc;
use crate::commands::{RegisterUserCommand, RenameUserCommand};
use crate::events::EventBus;
use crate::infrastructure::{Logger, DomainResult};
use crate::domain::{Repository, IRepository, User};

/// UserCommandHandler - CQRS write side handler
/// Processes commands through aggregates (not directly to events)
/// Follows m-r pattern: Command → Aggregate → Events → EventStore
/// Supports both sync and async event publishing
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

    /// Execute RegisterUserCommand (synchronous)
    /// Uses blocking event publishing for backward compatibility
    pub fn handle_register_user(&self, command: RegisterUserCommand) -> DomainResult<()> {
        self.logger.log(&format!(
            "Processing command: RegisterUser(id={}, name={})",
            command.user_id, command.name
        ));

        // Create aggregate - this applies events internally
        let user = User::new(command.user_id, command.name.clone());

        // Save through repository (handles optimistic locking, persistence)
        // Returns the events that were saved
        let saved_events = self.repository.save(&user, -1)?; // -1 indicates new aggregate

        // Publish events synchronously (blocking) for backward compatibility
        // In async contexts, use handle_register_user_async instead
        for event in saved_events {
            self.event_bus.publish_blocking(&event);
        }

        self.logger
            .log(&format!("User {} registered successfully", command.user_id));

        Ok(())
    }

    /// Execute RegisterUserCommand (asynchronous)
    /// Non-blocking event publishing - recommended for high-throughput scenarios
    pub async fn handle_register_user_async(&self, command: RegisterUserCommand) -> DomainResult<()> {
        self.logger.log(&format!(
            "Processing command (async): RegisterUser(id={}, name={})",
            command.user_id, command.name
        ));

        // Create aggregate - this applies events internally
        let user = User::new(command.user_id, command.name.clone());

        // Save through repository (handles optimistic locking, persistence)
        let saved_events = self.repository.save(&user, -1)?;

        // Publish events asynchronously - non-blocking
        for event in saved_events {
            self.event_bus.publish(&event).await;
        }

        self.logger
            .log(&format!("User {} registered successfully (async)", command.user_id));

        Ok(())
    }

    /// Execute RenameUserCommand (synchronous)
    /// Uses blocking event publishing for backward compatibility
    pub fn handle_rename_user(&self, command: RenameUserCommand) -> DomainResult<()> {
        self.logger.log(&format!(
            "Processing command: RenameUser(id={}, new_name={})",
            command.user_id, command.new_name
        ));

        // Load aggregate from history (event sourcing reconstruction)
        let mut user = self.repository.get_by_id(command.user_id)?;

        // Apply the rename to the aggregate
        user.rename(command.new_name.clone());

        // Save through repository (handles optimistic locking, persistence)
        let saved_events = self.repository.save(&user, user.version)?;

        // Publish events synchronously (blocking) for backward compatibility
        for event in saved_events {
            self.event_bus.publish_blocking(&event);
        }

        self.logger
            .log(&format!("User {} renamed successfully", command.user_id));

        Ok(())
    }

    /// Execute RenameUserCommand (asynchronous)
    /// Non-blocking event publishing - recommended for high-throughput scenarios
    pub async fn handle_rename_user_async(&self, command: RenameUserCommand) -> DomainResult<()> {
        self.logger.log(&format!(
            "Processing command (async): RenameUser(id={}, new_name={})",
            command.user_id, command.new_name
        ));

        // Load aggregate from history (event sourcing reconstruction)
        let mut user = self.repository.get_by_id(command.user_id)?;

        // Apply the rename to the aggregate
        user.rename(command.new_name.clone());

        // Save through repository (handles optimistic locking, persistence)
        let saved_events = self.repository.save(&user, user.version)?;

        // Publish events asynchronously - non-blocking
        for event in saved_events {
            self.event_bus.publish(&event).await;
        }

        self.logger
            .log(&format!("User {} renamed successfully (async)", command.user_id));

        Ok(())
    }
}
