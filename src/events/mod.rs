// Events Module: Domain events and event sourcing infrastructure
// Events represent immutable facts about what happened in the domain

pub mod domain_events;
pub mod event_store;
pub mod event_bus;
pub mod projections;

pub use domain_events::{DomainEvent, UserRegisteredEvent};
pub use event_store::EventStore;
pub use event_bus::{EventBus, EventHandler};
pub use projections::UserProjection;
