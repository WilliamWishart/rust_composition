// User Repository Implementation
use domain::{User, events::UserEvent, errors::DomainResult, repository::IRepository};
use crate::event_store::EventStore;

pub struct Repository {
    event_store: EventStore,
}

impl Repository {
    pub fn new(event_store: EventStore) -> Self {
        Repository { event_store }
    }
}

impl IRepository for Repository {
    fn save(&self, aggregate: &User, expected_version: i32) -> DomainResult<Vec<UserEvent>> {
        let changes = aggregate.get_uncommitted_changes();

        if changes.is_empty() {
            return Ok(Vec::new());
        }

        if expected_version != -1 && aggregate.version != expected_version {
            return Err(domain::errors::AppError::ConcurrencyViolation {
                expected_version,
                actual_version: aggregate.version,
            });
        }

        for event in changes.iter() {
            self.event_store.append(aggregate.id, event.clone());
        }

        Ok(changes)
    }

    fn get_by_id(&self, id: u32) -> DomainResult<User> {
        let events = self.event_store.get_events(id);

        if events.is_empty() {
            return Err(domain::errors::AppError::AggregateNotFound(id));
        }

        User::load_from_history(events)
    }

    fn find_by_name(&self, name: &str) -> DomainResult<Option<User>> {
        self.event_store.find_user_by_name(name)
    }
}
