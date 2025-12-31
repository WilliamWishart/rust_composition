// ============================================================================
// ENTERPRISE-SCALE RUST APPLICATION STRUCTURE
// ============================================================================
//
// This library demonstrates a properly organized, large-scale Rust application
// using Domain-Driven Design (DDD) with CQRS and Event Sourcing.
//
// Architecture follows Gregory Young's m-r CQRS pattern:
// - Aggregates apply events internally and track versions
// - Repository reconstructs aggregates from event history
// - Commands processed through aggregates (not directly to events)
// - Optimistic locking prevents concurrency violations
// - Eventual consistency via event bus and projections
//
// Module Organization:
// - infrastructure/  : Traits and external adapters (Logger)
// - domain/          : Business logic and repositories (pure domain)
// - events/          : Domain events, event store, and event bus (event sourcing)
// - commands/        : Command handlers (CQRS write side)
// - queries/         : Query handlers (CQRS read side)
// - composition/     : Dependency injection and wiring
//
// NOTE: UserService and UserRepository have been removed.
// Use CommandHandler + Repository for writes, UserQuery + Projections for reads.

pub mod infrastructure;
pub mod domain;
pub mod events;
pub mod commands;
pub mod queries;
pub mod application;
pub mod composition;

// Re-export commonly used types at the library root for convenience
pub use composition::AppBuilder;
pub use domain::{Repository, IRepository};
pub use infrastructure::Logger;
pub use events::{EventStore, EventBus, EventHandler, UserEvent};
pub use commands::{RegisterUserCommand, RenameUserCommand};
pub use events::projections::TypedUserProjectionHandler;
pub use queries::UserQuery;
