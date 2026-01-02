// Projection event handler adapter
use async_trait::async_trait;
use domain::events::UserEvent;
use persistence::projections::{UserProjection, Handles, TypedUserProjectionHandler};
use crate::event_bus::{EventHandler, HandlerPriority};

/// ProjectionEventHandler - Adapts UserProjection to work with EventBus
pub struct ProjectionEventHandler {
    handler: TypedUserProjectionHandler,
}

impl ProjectionEventHandler {
    pub fn new(projection: UserProjection) -> Self {
        ProjectionEventHandler {
            handler: TypedUserProjectionHandler::new(projection),
        }
    }
}

#[async_trait]
impl EventHandler for ProjectionEventHandler {
    async fn handle_event(&self, event: &UserEvent) -> Result<(), Box<dyn std::error::Error>> {
        self.handler.handle(event);
        Ok(())
    }
    
    fn priority(&self) -> HandlerPriority {
        HandlerPriority::Critical
    }
    
    fn name(&self) -> &str {
        "ProjectionEventHandler"
    }
}
