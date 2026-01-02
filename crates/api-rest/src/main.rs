use axum::{
    routing::{post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use application::{EventBus, UserCommandHandler, ProjectionEventHandler};
use infrastructure::{ConsoleLogger, LogLevel};
use persistence::{EventStore, Repository, UserProjection};
use api_rest::{handlers::{register_user, rename_user}, AppState};

#[tokio::main]
async fn main() {
    // Initialize infrastructure
    let logger = Arc::new(ConsoleLogger::new(LogLevel::Info));
    let event_store = EventStore::new();
    
    // Initialize projection and event bus
    let projection = UserProjection::new();
    let event_bus = EventBus::new().with_logger(logger.clone());
    
    // Subscribe projection to events
    let projection_handler = Arc::new(ProjectionEventHandler::new(projection.clone()));
    event_bus.subscribe(projection_handler);
    
    // Create repository with both event store and projection
    let repository = Arc::new(Repository::new(event_store, projection));

    // Create command handler
    let command_handler = Arc::new(UserCommandHandler::new(
        repository.clone(),
        event_bus,
        logger.clone(),
    ));

    let state = AppState {
        command_handler,
        logger: logger.clone(),
    };

    // Build router with routes
    let app = Router::new()
        .route("/users", post(register_user))
        .route("/users", put(rename_user))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    (logger.as_ref() as &dyn infrastructure::Logger).info("Starting REST API server on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
