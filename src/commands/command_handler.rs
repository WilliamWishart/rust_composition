use std::sync::Arc;
use crate::commands::RegisterUserCommand;
use crate::events::{EventStore, EventBus, UserRegisteredEvent};
use crate::infrastructure::Logger;

/// UserCommandHandler - CQRS write side handler
/// Processes commands, validates them, and emits domain events
/// This is where business logic lives in event sourcing
pub struct UserCommandHandler {
    event_store: EventStore,
    event_bus: EventBus,
    logger: Arc<dyn Logger>,
}

impl UserCommandHandler {
    pub fn new(
        event_store: EventStore,
        event_bus: EventBus,
        logger: Arc<dyn Logger>,
    ) -> Self {
        UserCommandHandler {
            event_store,
            event_bus,
            logger,
        }
    }

    /// Execute RegisterUserCommand
    /// Validates the command and emits UserRegisteredEvent
    pub fn handle_register_user(&self, command: RegisterUserCommand) -> Result<(), String> {
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

        // Create domain event (events never fail - they're facts)
        let event = UserRegisteredEvent::new(command.user_id, command.name.clone());

        // Store event (write to event store)
        self.event_store
            .append(Arc::new(event.clone()));

        // Publish event (notify subscribers for eventual consistency)
        self.event_bus.publish(&event);

        self.logger
            .log(&format!("User {} registered successfully", command.user_id));

        Ok(())
    }

    pub fn get_event_store(&self) -> EventStore {
        self.event_store.clone()
    }

    pub fn get_event_bus(&self) -> EventBus {
        self.event_bus.clone()
    }
}
