use axum::{
    routing::{post, put, get},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use application::{EventBus, UserCommandHandler, ProjectionEventHandler};
use infrastructure::{ConsoleLogger, LogLevel};
use persistence::{EventStore, Repository, UserProjection};
use api_rest::{handlers::{register_user, rename_user, get_user, get_all_users, find_user_by_name}, AppState, openapi::ApiDoc};

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
    let repository = Arc::new(Repository::new(event_store, projection.clone()));

    // Create command handler
    let command_handler = Arc::new(UserCommandHandler::new(
        repository.clone(),
        event_bus,
        logger.clone(),
    ));

    let state = AppState {
        command_handler,
        projection: projection.clone(),
        logger: logger.clone(),
    };

    // Build router with routes
    let app = Router::new()
        .route("/users", post(register_user))
        .route("/users", get(get_all_users))
        .route("/users", put(rename_user))
        .route("/users/:user_id", get(get_user))
        .route("/users/search/:name", get(find_user_by_name))
        .merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("API_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap();
    
    (logger.as_ref() as &dyn infrastructure::Logger).info(&format!("Starting REST API server on http://0.0.0.0:{}", port));
    (logger.as_ref() as &dyn infrastructure::Logger).info(&format!("OpenAPI documentation available at http://0.0.0.0:{}/swagger-ui", port));

    axum::serve(listener, app).await.unwrap();
}
