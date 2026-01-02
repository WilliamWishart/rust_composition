use std::sync::Arc;
use infrastructure::Logger;
use application::UserCommandHandler;
use persistence::UserProjection;

pub mod dto;
pub mod handlers;

#[derive(Clone)]
pub struct AppState {
    pub command_handler: Arc<UserCommandHandler>,
    pub projection: UserProjection,
    pub logger: Arc<dyn Logger>,
}

