// Command handlers
use std::sync::Arc;
use domain::{
    commands::{RegisterUserCommand, RenameUserCommand},
    errors::DomainResult,
    IRepository, UserRegistrationService, UserId, UserName,
};
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
    registration_service: UserRegistrationService,
    event_bus: EventBus,
    logger: Arc<dyn Logger>,
}

impl UserCommandHandler {
    pub fn new(
        repository: Arc<Repository>,
        event_bus: EventBus,
        logger: Arc<dyn Logger>,
    ) -> Self {
        let registration_service = UserRegistrationService::new(repository.clone());
        
        UserCommandHandler {
            repository,
            registration_service,
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

        // Convert primitives to value objects
        let user_id = UserId::new(command.user_id)?;
        let user_name = UserName::new(command.name.clone())?;

        // Use domain service to register user with all specifications
        let user = self.registration_service.register_user(user_id, user_name)?;

        let saved_events = self.repository.save(&user, -1)?;

        for (_index, event) in saved_events.iter().enumerate() {
            let _envelope = domain::events::EventEnvelope::new(
                user.id().value(),
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

        // Convert to value object with validation
        let new_name = UserName::new(command.new_name.clone())?;
        user.rename(new_name)?;

        let saved_events = self.repository.save(&user, user.version())?;

        for (_index, event) in saved_events.iter().enumerate() {
            let _envelope = domain::events::EventEnvelope::new(
                user.id().value(),
                event.clone(),
                user.version(),
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
