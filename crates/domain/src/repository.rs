// Repository trait for aggregate persistence
use crate::aggregates::User;
use crate::events::UserEvent;
use crate::errors::DomainResult;

/// Repository trait - abstraction for aggregate storage
pub trait IRepository: Send + Sync {
    fn save(&self, aggregate: &User, expected_version: i32) -> DomainResult<Vec<UserEvent>>;
    fn get_by_id(&self, id: u32) -> DomainResult<User>;
    fn find_by_name(&self, name: &str) -> DomainResult<Option<User>>;
}
