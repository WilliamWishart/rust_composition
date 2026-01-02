use crate::events::UserEvent;
use crate::infrastructure::DomainResult;
use std::fmt;

/// User Aggregate - AggregateRoot pattern from m-r
/// Encapsulates both state and business logic
/// Uses enum-based events instead of trait objects - pure Rust idiom
#[derive(Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub version: i32, // Tracks event version for optimistic locking
    uncommitted_changes: Vec<UserEvent>, // Concrete events, no Arc, no trait objects!
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
    /// Create a new user with invariant validation
    /// 
    /// This factory method encapsulates the business rule:
    /// "Every user must have a unique username"
    /// 
    /// IMPORTANT: This violates the principle that aggregates should be repository-free,
    /// but it's necessary for enforcing uniqueness constraints.
    /// 
    /// Alternative patterns:
    /// 1. Use a DomainService (UserRegistrationService) that accepts repository
    /// 2. Use a factory pattern (UserFactory) that's injected into aggregate
    /// 3. Separate "create" (factory) from "new" (constructor)
    /// 
    /// We've chosen option 1: the factory method includes repository dependency
    /// because it keeps all User validation logic in one place.
    pub fn new_with_uniqueness_check(
        id: u32,
        name: String,
        repository: &dyn crate::domain::IRepository,
    ) -> DomainResult<Self> {
        // ===== VALIDATE ALL INVARIANTS BEFORE CREATION =====
        
        // Invariant 1: User ID must be positive
        if id == 0 {
            return Err(crate::infrastructure::DomainError::Validation(
                "User ID must be greater than 0".to_string(),
            ));
        }
        
        // Invariant 2: Name must not be empty or whitespace
        if name.trim().is_empty() {
            return Err(crate::infrastructure::DomainError::Validation(
                "Name cannot be empty".to_string(),
            ));
        }
        
        // Invariant 3: Name must not exceed maximum length
        if name.len() > 255 {
            return Err(crate::infrastructure::DomainError::Validation(
                "Name cannot exceed 255 characters".to_string(),
            ));
        }

        // Invariant 4: Username must be unique
        if repository.find_by_name(&name)?.is_some() {
            let existing = repository.find_by_name(&name)?;
            return Err(crate::infrastructure::DomainError::Validation(
                format!("Username '{}' is already taken by user ID {}", 
                       name, existing.unwrap().id)
            ));
        }

        // ===== ALL INVARIANTS SATISFIED - SAFE TO CREATE =====
        
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

    /// Create a new user WITHOUT uniqueness validation
    /// 
    /// This method validates value constraints but NOT domain state constraints.
    /// Use for:
    /// - Testing purposes
    /// - Reconstructing from event store (load_from_history)
    /// - Internal domain operations
    /// 
    /// For user-facing operations, ALWAYS use new_with_uniqueness_check()
    /// 
    /// NOTE: This still validates value constraints (ID > 0, name length, etc.)
    /// to prevent invalid state even in testing scenarios.
    pub fn new(id: u32, name: String) -> DomainResult<Self> {
        // ===== VALUE CONSTRAINTS (no I/O required) =====
        
        // Invariant 1: User ID must be positive
        if id == 0 {
            return Err(crate::infrastructure::DomainError::Validation(
                "User ID must be greater than 0".to_string(),
            ));
        }
        
        // Invariant 2: Name must not be empty or whitespace
        if name.trim().is_empty() {
            return Err(crate::infrastructure::DomainError::Validation(
                "Name cannot be empty".to_string(),
            ));
        }
        
        // Invariant 3: Name must not exceed maximum length
        if name.len() > 255 {
            return Err(crate::infrastructure::DomainError::Validation(
                "Name cannot exceed 255 characters".to_string(),
            ));
        }

        // ===== VALUE CONSTRAINTS SATISFIED - SAFE TO CREATE =====
        
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

    /// Apply event - updates aggregate state
    /// Uses pattern matching on the enum - compiler enforces all cases handled
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

    /// Load aggregate from event history (event sourcing reconstruction)
    /// Pure pattern matching - no downcasting, no runtime type checks needed!
    pub fn load_from_history(events: Vec<UserEvent>) -> DomainResult<Self> {
        let mut user = User {
            id: 0,
            name: String::new(),
            version: -1,
            uncommitted_changes: Vec::new(),
        };

        for (index, event) in events.iter().enumerate() {
            // Compiler guarantees we handle all UserEvent variants
            user.apply_event(event);
            user.version = index as i32; // Version increments with each event
        }

        Ok(user)
    }

    /// Get uncommitted changes (for Repository.Save)
    pub fn get_uncommitted_changes(&self) -> Vec<UserEvent> {
        self.uncommitted_changes.clone()
    }

    /// Mark changes as committed (called after successful persist)
    pub fn mark_changes_as_committed(&mut self) {
        self.uncommitted_changes.clear();
    }

    /// Rename the user (domain command)
    /// 
    /// Validates the new name before applying the change.
    /// Ensures name constraints are always maintained.
    pub fn rename(&mut self, new_name: String) -> DomainResult<()> {
        // ===== VALIDATE NEW NAME BEFORE APPLYING CHANGE =====
        
        // Invariant 1: Name must not be empty or whitespace
        if new_name.trim().is_empty() {
            return Err(crate::infrastructure::DomainError::Validation(
                "New name cannot be empty".to_string(),
            ));
        }
        
        // Invariant 2: Name must not exceed maximum length
        if new_name.len() > 255 {
            return Err(crate::infrastructure::DomainError::Validation(
                "New name cannot exceed 255 characters".to_string(),
            ));
        }

        // ===== VALIDATION PASSED - SAFE TO APPLY CHANGE =====

        let event = UserEvent::Renamed {
            user_id: self.id,
            new_name,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        // Apply the event to self (updates state)
        self.apply_event(&event);

        // Record the uncommitted change
        self.uncommitted_changes.push(event);
        
        Ok(())
    }
}
