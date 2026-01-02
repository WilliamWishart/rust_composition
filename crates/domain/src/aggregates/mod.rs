// Aggregate Root: User
// Encapsulates state and business logic for the User domain concept
use crate::events::UserEvent;
use crate::errors::DomainResult;
use std::fmt;

/// User Aggregate - Encapsulates both state and business logic
#[derive(Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub version: i32,
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
    /// Create a new user with all invariants validated
    /// Includes uniqueness check via repository dependency
    pub fn new_with_uniqueness_check(
        id: u32,
        name: String,
        repository: &dyn crate::repository::IRepository,
    ) -> DomainResult<Self> {
        // Validate invariants
        if id == 0 {
            return Err(crate::errors::AppError::Validation(
                "User ID must be greater than 0".to_string(),
            ));
        }
        
        if name.trim().is_empty() {
            return Err(crate::errors::AppError::Validation(
                "Name cannot be empty".to_string(),
            ));
        }
        
        if name.len() > 255 {
            return Err(crate::errors::AppError::Validation(
                "Name cannot exceed 255 characters".to_string(),
            ));
        }

        // Check uniqueness via repository
        let existing = repository.find_by_name(&name)?;
        if let Some(existing_user) = existing {
            return Err(crate::errors::AppError::Validation(
                format!("Username '{}' is already taken by user ID {}", 
                       name, existing_user.id)
            ));
        }

        // Create new user
        let mut user = User {
            id,
            name: String::new(),
            version: -1,
            uncommitted_changes: Vec::new(),
        };

        let event = UserEvent::Registered {
            user_id: id,
            name,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        user.apply_event(&event);
        user.uncommitted_changes.push(event);

        Ok(user)
    }

    /// Create a new user with value constraint validation only
    /// For testing and event sourcing reconstruction
    pub fn new(id: u32, name: String) -> DomainResult<Self> {
        if id == 0 {
            return Err(crate::errors::AppError::Validation(
                "User ID must be greater than 0".to_string(),
            ));
        }
        
        if name.trim().is_empty() {
            return Err(crate::errors::AppError::Validation(
                "Name cannot be empty".to_string(),
            ));
        }
        
        if name.len() > 255 {
            return Err(crate::errors::AppError::Validation(
                "Name cannot exceed 255 characters".to_string(),
            ));
        }

        let mut user = User {
            id,
            name: String::new(),
            version: -1,
            uncommitted_changes: Vec::new(),
        };

        let event = UserEvent::Registered {
            user_id: id,
            name,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        user.apply_event(&event);
        user.uncommitted_changes.push(event);

        Ok(user)
    }

    /// Apply an event to the aggregate state
    fn apply_event(&mut self, event: &UserEvent) {
        match event {
            UserEvent::Registered {
                user_id,
                name,
                timestamp: _,
            } => {
                self.id = *user_id;
                self.name = name.clone();
            }
            UserEvent::Renamed {
                user_id: _,
                new_name,
                timestamp: _,
            } => {
                self.name = new_name.clone();
            }
        }
    }

    /// Reconstruct aggregate from event history
    pub fn load_from_history(events: Vec<UserEvent>) -> DomainResult<Self> {
        let mut user = User {
            id: 0,
            name: String::new(),
            version: -1,
            uncommitted_changes: Vec::new(),
        };

        for (index, event) in events.iter().enumerate() {
            user.apply_event(event);
            user.version = index as i32;
        }

        Ok(user)
    }

    pub fn get_uncommitted_changes(&self) -> Vec<UserEvent> {
        self.uncommitted_changes.clone()
    }

    pub fn mark_changes_as_committed(&mut self) {
        self.uncommitted_changes.clear();
    }

    /// Rename the user with validation
    pub fn rename(&mut self, new_name: String) -> DomainResult<()> {
        if new_name.trim().is_empty() {
            return Err(crate::errors::AppError::Validation(
                "New name cannot be empty".to_string(),
            ));
        }
        
        if new_name.len() > 255 {
            return Err(crate::errors::AppError::Validation(
                "New name cannot exceed 255 characters".to_string(),
            ));
        }

        let event = UserEvent::Renamed {
            user_id: self.id,
            new_name,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        self.apply_event(&event);
        self.uncommitted_changes.push(event);
        
        Ok(())
    }
}
