use std::sync::Arc;
use crate::commands::{RegisterUserCommand, RenameUserCommand};
use crate::events::EventBus;
use crate::infrastructure::{Logger, DomainResult};
use crate::domain::{Repository, IRepository, User};

/// Generate a unique correlation ID for a command
fn generate_correlation_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("cmd_{}", nanos)
}

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
    /// 
    /// Note: The aggregate enforces its own business rules (username uniqueness)
    /// via User::new_with_uniqueness_check(). The command handler is now purely
    /// an orchestrator.
    pub async fn handle_register_user(&self, command: RegisterUserCommand) -> DomainResult<()> {
        let correlation_id = generate_correlation_id();
        
        self.logger.info(&format!(
            "Processing command: RegisterUser(id={}, name={}) [corr_id={}]",
            command.user_id, command.name, correlation_id
        ));

        // ===== Aggregate validates its own invariants =====
        // The aggregate checks uniqueness - business logic encapsulated in domain model
        let user = User::new_with_uniqueness_check(
            command.user_id,
            command.name.clone(),
            self.repository.as_ref(),
        )?;

        // Save through repository (handles optimistic locking, persistence)
        // Returns the events that were saved
        let saved_events = self.repository.save(&user, -1)?; // -1 indicates new aggregate

        // Publish events with proper error handling and awaiting
        // Wrap events in envelopes with metadata
        for (index, event) in saved_events.iter().enumerate() {
            let _envelope = crate::events::EventEnvelope::new(
                user.id,
                event.clone(),
                index as i32,
                correlation_id.clone(),
            );
            
            match self.event_bus.publish(&event).await {
                Ok(errors) if errors.is_empty() => {},
                Ok(errors) => {
                    for err in errors {
                        self.logger.warn(&format!("Non-critical handler error: {}", err));
                    }
                }
                Err(e) => {
                    self.logger.error(&format!("Critical error publishing event: {}", e));
                    return Err(crate::infrastructure::DomainError::PublishError(
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
        let correlation_id = generate_correlation_id();
        
        self.logger.info(&format!(
            "Processing command: RenameUser(id={}, new_name={}) [corr_id={}]",
            command.user_id, command.new_name, correlation_id
        ));

        // Load aggregate from history (event sourcing reconstruction)
        let mut user = self.repository.get_by_id(command.user_id)?;

        // Apply the rename to the aggregate - now validates
        user.rename(command.new_name.clone())?;

        // Save through repository (handles optimistic locking, persistence)
        let saved_events = self.repository.save(&user, user.version)?;

        // Publish events with proper error handling and awaiting
        // Wrap events in envelopes with metadata
        for (index, event) in saved_events.iter().enumerate() {
            let event_version = user.version + index as i32;
            let _envelope = crate::events::EventEnvelope::new(
                user.id,
                event.clone(),
                event_version,
                correlation_id.clone(),
            );
            
            match self.event_bus.publish(&event).await {
                Ok(errors) if errors.is_empty() => {},
                Ok(errors) => {
                    for err in errors {
                        self.logger.warn(&format!("Non-critical handler error: {}", err));
                    }
                }
                Err(e) => {
                    self.logger.error(&format!("Critical error publishing event: {}", e));
                    return Err(crate::infrastructure::DomainError::PublishError(
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
