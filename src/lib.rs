// ============================================================================
// ENTERPRISE-SCALE RUST APPLICATION STRUCTURE
// ============================================================================
//
// This library demonstrates a properly organized, large-scale Rust application
// using Domain-Driven Design (DDD) with CQRS and Event Sourcing.
//
// Module Organization:
// - infrastructure/  : Traits and external adapters (Logger, Database)
// - domain/          : Business logic and repositories (pure domain)
// - events/          : Domain events, event store, and event bus (event sourcing)
// - commands/        : Command handlers (CQRS write side)
// - queries/         : Query handlers (CQRS read side)
// - application/     : Services that orchestrate use cases
// - composition/     : Dependency injection and wiring

pub mod infrastructure;
pub mod domain;
pub mod events;
pub mod commands;
pub mod queries;
pub mod application;
pub mod composition;

// Re-export commonly used types at the library root for convenience
pub use composition::AppBuilder;
pub use application::UserService;
pub use domain::UserRepository;
pub use infrastructure::{Logger, Database};
pub use events::{EventStore, EventBus, DomainEvent, UserRegisteredEvent};
pub use events::projections::TypedUserProjectionHandler;
pub use commands::{RegisterUserCommand, UserCommandHandler};
pub use queries::UserQuery;
