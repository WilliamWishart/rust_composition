use std::sync::Arc;
use crate::commands::{RegisterUserCommand, RenameUserCommand};
use crate::events::EventBus;
use crate::infrastructure::{Logger, DomainResult};
use crate::domain::{Repository, IRepository, User};

/// UserCommandHandler - CQRS write side handler
/// Processes commands through aggregates (not directly to events)
/// Follows m-r pattern: Command → Aggregate → Events → EventStore
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

    /// Execute RegisterUserCommand
    /// 1. Validate command
    /// 2. Create aggregate (which produces UserRegisteredEvent)
    /// 3. Save aggregate (persists event via repository)
    /// 4. Publish event for eventual consistency
    /// Returns the published events so caller can update read models
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

        // Publish events for subscribers (eventual consistency)
        for event in saved_events {
            self.event_bus.publish(&event);
        }

        self.logger
            .log(&format!("User {} registered successfully", command.user_id));

        Ok(())
    }

    /// Execute RenameUserCommand
    /// 1. Validate command
    /// 2. Load aggregate from history (event sourcing)
    /// 3. Apply rename to aggregate (which produces UserRenamedEvent)
    /// 4. Save aggregate (persists event via repository)
    /// 5. Publish event for eventual consistency
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

        // Publish events for subscribers (eventual consistency)
        for event in saved_events {
            self.event_bus.publish(&event);
        }

        self.logger
            .log(&format!("User {} renamed successfully", command.user_id));

        Ok(())
    }
}
