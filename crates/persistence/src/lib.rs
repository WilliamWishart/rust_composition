// Persistence Layer - Event store, repository implementation, projections
pub mod event_store;
pub mod user_repository;
pub mod projections;

pub use event_store::EventStore;
pub use user_repository::Repository;
pub use projections::UserProjection;
