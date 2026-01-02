use crate::domain::User;
use crate::events::{EventStore, UserEvent};
use crate::infrastructure::{DomainError, DomainResult};

/// IRepository<T> pattern from m-r reference
/// Handles persistence and retrieval of aggregates using event sourcing
pub trait IRepository {
    fn save(&self, aggregate: &User, expected_version: i32) -> DomainResult<Vec<UserEvent>>;
    fn get_by_id(&self, id: u32) -> DomainResult<User>;
    
    /// Check if a user with the given name already exists
    /// Used for enforcing business rules like "usernames must be unique"
    fn find_by_name(&self, name: &str) -> DomainResult<Option<User>>;
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

    /// Find a user by name - enforces business rule: usernames must be unique
    /// 
    /// This is a business rule check implemented at the repository level.
    /// In Event Sourcing, we need to scan all aggregates since the event store
    /// is indexed by aggregate ID, not by business properties like name.
    /// 
    /// In a production system, you would typically:
    /// 1. Maintain a read model projection indexed by name
    /// 2. Or use a secondary index in your data store
    /// 3. Or cache recently accessed users
    fn find_by_name(&self, name: &str) -> DomainResult<Option<User>> {
        // In this simplified implementation, we scan all users
        // Production systems would use a projection or index
        self.event_store.find_user_by_name(name)
    }
}


