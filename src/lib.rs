// ============================================================================
// ENTERPRISE-SCALE RUST APPLICATION STRUCTURE
// ============================================================================
//
// This library demonstrates a properly organized, large-scale Rust application
// using Domain-Driven Design and Dependency Injection patterns.
//
// Module Organization:
// - infrastructure/  : Traits and external adapters (Logger, Database)
// - domain/          : Business logic and repositories
// - application/     : Services that orchestrate use cases
// - composition/     : Dependency injection and wiring

pub mod infrastructure;
pub mod domain;
pub mod application;
pub mod composition;

// Re-export commonly used types at the library root for convenience
pub use composition::AppBuilder;
pub use application::UserService;
pub use domain::UserRepository;
pub use infrastructure::{Logger, Database};
