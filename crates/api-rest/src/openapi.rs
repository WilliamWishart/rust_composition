use utoipa::OpenApi;
use crate::dto::{RegisterUserRequest, RenameUserRequest, UserResponse, SuccessResponse, ErrorResponse};

/// OpenAPI documentation for the User Management API
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::commands::register_user,
        crate::handlers::commands::rename_user,
        crate::handlers::queries::get_user,
        crate::handlers::queries::get_all_users,
        crate::handlers::queries::find_user_by_name,
    ),
    components(
        schemas(RegisterUserRequest, RenameUserRequest, UserResponse, SuccessResponse, ErrorResponse)
    ),
    info(
        title = "User Management API",
        description = "A REST API for managing users using CQRS and Event Sourcing patterns",
        version = "0.1.0",
        contact(
            name = "API Support",
            url = "https://github.com/WilliamWishart/rust_composition"
        ),
        license(
            name = "MIT"
        )
    ),
    servers(
        (url = "http://127.0.0.1:3000", description = "Local development server")
    ),
    tags(
        (name = "Users", description = "User management endpoints")
    )
)]
pub struct ApiDoc;
