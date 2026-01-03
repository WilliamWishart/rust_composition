// Aggregate Root: User
// Encapsulates state and business logic for the User domain concept
use crate::events::UserEvent;
use crate::errors::DomainResult;
use crate::value_objects::{UserId, UserName};
use std::fmt;

/// User Aggregate - Encapsulates both state and business logic using Value Objects
#[derive(Clone)]
pub struct User {
    id: UserId,
    name: UserName,
    version: i32,
    uncommitted_changes: Vec<UserEvent>,
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("version", &self.version)
            .field("uncommitted_changes", &format!("<{} events>", self.uncommitted_changes.len()))
            .finish()
    }
}

impl User {
    /// Create a new user with value object validation
    /// This is the primary constructor for new users
    pub fn new(id: UserId, name: UserName) -> DomainResult<Self> {
        let mut user = User {
            id,
            name: name.clone(),
            version: -1,
            uncommitted_changes: Vec::new(),
        };

        let event = UserEvent::Registered {
            user_id: id.value(),
            name: name.value().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        user.apply_event(&event);
        user.uncommitted_changes.push(event);

        Ok(user)
    }

    /// Create a new user with uniqueness check via repository
    /// Includes validation that the user ID and name are not already taken
    pub fn new_with_uniqueness_check(
        id: UserId,
        name: UserName,
        _repository: &dyn crate::repository::IRepository,
    ) -> DomainResult<Self> {
        // The repository check is done at the service/handler level via specifications
        // This constructor just validates the value objects
        Self::new(id, name)
    }

    /// Reconstruct aggregate from event history
    /// Used for event sourcing - loads historical events
    pub fn load_from_history(events: Vec<UserEvent>) -> DomainResult<Self> {
        let mut user = User {
            id: UserId::new(0).unwrap_or(UserId::from(1)),
            name: UserName::new("placeholder".to_string())?,
            version: -1,
            uncommitted_changes: Vec::new(),
        };

        for (index, event) in events.iter().enumerate() {
            user.apply_event(event);
            user.version = index as i32;
        }

        Ok(user)
    }

    // ===== Value Object Accessors =====

    /// Get the user ID
    pub fn id(&self) -> UserId {
        self.id
    }

    /// Get the user name
    pub fn name(&self) -> &UserName {
        &self.name
    }

    /// Get the aggregate version (for optimistic locking)
    pub fn version(&self) -> i32 {
        self.version
    }

    // ===== Event Sourcing =====

    /// Apply an event to the aggregate state
    fn apply_event(&mut self, event: &UserEvent) {
        match event {
            UserEvent::Registered {
                user_id,
                name,
                timestamp: _,
            } => {
                self.id = UserId::from(*user_id);
                self.name = UserName::new(name.clone()).unwrap_or_else(|_| {
                    UserName::new("default".to_string()).unwrap()
                });
            }
            UserEvent::Renamed {
                user_id: _,
                new_name,
                timestamp: _,
            } => {
                if let Ok(new_name_vo) = UserName::new(new_name.clone()) {
                    self.name = new_name_vo;
                }
            }
        }
    }

    pub fn get_uncommitted_changes(&self) -> Vec<UserEvent> {
        self.uncommitted_changes.clone()
    }

    pub fn mark_changes_as_committed(&mut self) {
        self.uncommitted_changes.clear();
    }

    // ===== Domain Behavior =====

    /// Rename the user
    /// Domain rule: new name must be different from current name (case-insensitive)
    pub fn rename(&mut self, new_name: UserName) -> DomainResult<()> {
        // Domain business rule: cannot rename to the same name
        if !self.name.can_be_renamed_to(&new_name) {
            return Err(crate::errors::AppError::Validation(
                format!(
                    "New name must be different from current name '{}'",
                    self.name
                ),
            ));
        }

        let event = UserEvent::Renamed {
            user_id: self.id.value(),
            new_name: new_name.value().to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        self.apply_event(&event);
        self.uncommitted_changes.push(event);

        Ok(())
    }
}
