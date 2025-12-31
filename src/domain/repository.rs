use crate::domain::User;
use crate::events::EventStore;

/// IRepository<T> pattern from m-r reference
/// Handles persistence and retrieval of aggregates using event sourcing
pub trait IRepository {
    fn save(&self, aggregate: &User, expected_version: i32) -> Result<Vec<std::sync::Arc<dyn crate::events::DomainEvent>>, String>;
    fn get_by_id(&self, id: u32) -> Result<User, String>;
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
    fn save(&self, aggregate: &User, expected_version: i32) -> Result<Vec<std::sync::Arc<dyn crate::events::DomainEvent>>, String> {
        // Get uncommitted changes
        let changes = aggregate.get_uncommitted_changes();

        if changes.is_empty() {
            return Ok(Vec::new()); // Nothing to persist
        }

        // Check optimistic lock - ensure expected version matches
        // (In real implementation, would verify against stored version)
        if expected_version != -1 && aggregate.version != expected_version {
            return Err(format!(
                "Concurrency violation: expected version {}, but aggregate version is {}",
                expected_version, aggregate.version
            ));
        }

        // Persist all uncommitted events
        for event in changes.iter() {
            self.event_store.append(event.clone());
        }

        Ok(changes)
    }

    /// Load an aggregate by ID - reconstructs from event history
    fn get_by_id(&self, id: u32) -> Result<User, String> {
        // Get all events for this aggregate
        let events = self.event_store.get_events(&id.to_string());

        if events.is_empty() {
            return Err(format!("Aggregate not found: {}", id));
        }

        // Reconstruct aggregate from history (event sourcing)
        User::load_from_history(events)
    }
}

