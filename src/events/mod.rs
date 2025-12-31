// Events Module: Domain events and event sourcing infrastructure
// Events represent immutable facts about what happened in the domain

pub mod event_store;
pub mod event_bus;
pub mod projections;
pub mod user_events;

pub use event_store::EventStore;
pub use event_bus::{EventBus, EventHandler};
pub use projections::UserProjection;
pub use user_events::UserEvent;
