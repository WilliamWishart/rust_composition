use crate::domain::User;
use crate::events::{EventStore, UserEvent};
use crate::infrastructure::{DomainError, DomainResult};

/// IRepository<T> pattern from m-r reference
/// Handles persistence and retrieval of aggregates using event sourcing
pub trait IRepository {
    fn save(&self, aggregate: &User, expected_version: i32) -> DomainResult<Vec<UserEvent>>;
    fn get_by_id(&self, id: u32) -> DomainResult<User>;
}

/// Repository<User> - Concrete implementation
/// Converts between aggregate and event stream
pub struct Repository {
    event_store: EventStore,
}

impl Repository {
    pub fn new(event_store: EventStore) -> Self {
        Repository { event_store }
    }
}

impl IRepository for Repository {
    /// Save an aggregate - persists uncommitted events with optimistic locking
    /// Returns the events that were saved
    fn save(&self, aggregate: &User, expected_version: i32) -> DomainResult<Vec<UserEvent>> {
        // Get uncommitted changes
        let changes = aggregate.get_uncommitted_changes();

        if changes.is_empty() {
            return Ok(Vec::new()); // Nothing to persist
        }

        // Check optimistic lock - ensure expected version matches
        // (In real implementation, would verify against stored version)
        if expected_version != -1 && aggregate.version != expected_version {
            return Err(DomainError::ConcurrencyViolation {
                expected_version,
                actual_version: aggregate.version,
            });
        }

        // Persist all uncommitted events
        for event in changes.iter() {
            self.event_store.append(aggregate.id, event.clone());
        }

        Ok(changes)
    }

    /// Load an aggregate by ID - reconstructs from event history
    fn get_by_id(&self, id: u32) -> DomainResult<User> {
        // Get all events for this aggregate
        let events = self.event_store.get_events(id);

        if events.is_empty() {
            return Err(DomainError::AggregateNotFound(id));
        }

        // Reconstruct aggregate from history (event sourcing)
        User::load_from_history(events)
    }
}


