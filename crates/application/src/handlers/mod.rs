// Command handlers
use std::sync::Arc;
use domain::{commands::{RegisterUserCommand, RenameUserCommand}, User, errors::DomainResult, IRepository};
use infrastructure::Logger;
use persistence::Repository;
use crate::EventBus;

fn generate_correlation_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("cmd_{}", nanos)
}

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

    pub async fn handle_register_user(&self, command: RegisterUserCommand) -> DomainResult<()> {
        let correlation_id = generate_correlation_id();
        
        self.logger.info(&format!(
            "Processing command: RegisterUser(id={}, name={}) [corr_id={}]",
            command.user_id, command.name, correlation_id
        ));

        let user = User::new_with_uniqueness_check(
            command.user_id,
            command.name.clone(),
            self.repository.as_ref(),
        )?;

        let saved_events = self.repository.save(&user, -1)?;

        for (_index, event) in saved_events.iter().enumerate() {
            let _envelope = domain::events::EventEnvelope::new(
                user.id,
                event.clone(),
                0,
                correlation_id.clone(),
            );
            
            match self.event_bus.publish(event).await {
                Ok(errors) if errors.is_empty() => {},
                Ok(errors) => {
                    for err in errors {
                        self.logger.warn(&format!("Non-critical handler error: {}", err));
                    }
                }
                Err(e) => {
                    self.logger.error(&format!("Critical error publishing event: {}", e));
                    return Err(domain::errors::AppError::PublishError(
                        format!("Failed to publish event: {}", e)
                    ));
                }
            }
        }

        self.logger
            .info(&format!("User {} registered successfully", command.user_id));

        Ok(())
    }

    pub async fn handle_rename_user(&self, command: RenameUserCommand) -> DomainResult<()> {
        let correlation_id = generate_correlation_id();
        
        self.logger.info(&format!(
            "Processing command: RenameUser(id={}, new_name={}) [corr_id={}]",
            command.user_id, command.new_name, correlation_id
        ));

        let mut user = self.repository.get_by_id(command.user_id)?;

        user.rename(command.new_name.clone())?;

        let saved_events = self.repository.save(&user, user.version)?;

        for (_index, event) in saved_events.iter().enumerate() {
            let _envelope = domain::events::EventEnvelope::new(
                user.id,
                event.clone(),
                user.version,
                correlation_id.clone(),
            );
            
            match self.event_bus.publish(event).await {
                Ok(errors) if errors.is_empty() => {},
                Ok(errors) => {
                    for err in errors {
                        self.logger.warn(&format!("Non-critical handler error: {}", err));
                    }
                }
                Err(e) => {
                    self.logger.error(&format!("Critical error publishing event: {}", e));
                    return Err(domain::errors::AppError::PublishError(
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
