use std::sync::Arc;
use infrastructure::Logger;
use application::UserCommandHandler;

pub mod handlers;

#[derive(Clone)]
pub struct AppState {
    pub command_handler: Arc<UserCommandHandler>,
    pub logger: Arc<dyn Logger>,
}

