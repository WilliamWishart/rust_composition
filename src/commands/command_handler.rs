use std::sync::Arc;
use crate::commands::RegisterUserCommand;
use crate::events::EventBus;
use crate::infrastructure::Logger;
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
    pub fn handle_register_user(&self, command: RegisterUserCommand) -> Result<Vec<Arc<dyn crate::events::DomainEvent>>, String> {
        self.logger.log(&format!(
            "Processing command: RegisterUser(id={}, name={})",
            command.user_id, command.name
        ));

        // Validate command (commands can fail)
        if command.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }

        if command.user_id == 0 {
            return Err("User ID must be greater than 0".to_string());
        }

        // Create aggregate - this applies events internally
        let user = User::new(command.user_id, command.name.clone());

        // Save through repository (handles optimistic locking, persistence)
        // Returns the events that were saved
        let saved_events = self.repository.save(&user, -1)?; // -1 indicates new aggregate

        // Publish events for subscribers (eventual consistency)
        for event in saved_events.iter() {
            // Get the concrete event type for publishing
            if let Some(reg_event) = event.as_any().downcast_ref::<crate::events::UserRegisteredEvent>() {
                self.event_bus.publish(reg_event);
            }
        }

        self.logger
            .log(&format!("User {} registered successfully", command.user_id));

        Ok(saved_events)
    }
}


