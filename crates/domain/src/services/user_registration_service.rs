// UserRegistrationService - Domain Service
// Encapsulates the business logic for user registration
// Coordinates between aggregate, repository, and specifications

use crate::value_objects::{UserId, UserName};
use crate::repository::IRepository;
use crate::errors::DomainResult;
use crate::User;
use crate::specifications::Specification;
use crate::specifications::{UniqueUserNameSpecification, UniqueUserIdSpecification};
use std::sync::Arc;

/// Domain Service for user registration
/// Coordinates validation, uniqueness checks, and aggregate creation
pub struct UserRegistrationService {
    repository: Arc<dyn IRepository>,
}

impl UserRegistrationService {
    /// Create a new registration service with a repository
    pub fn new(repository: Arc<dyn IRepository>) -> Self {
        UserRegistrationService { repository }
    }

    /// Register a new user with all necessary validation
    ///
    /// This service:
    /// 1. Validates the user ID is unique
    /// 2. Validates the user name is unique
    /// 3. Validates the user name is valid for registration
    /// 4. Creates the user aggregate
    ///
    /// Returns the created User aggregate or an error
    pub fn register_user(
        &self,
        user_id: UserId,
        user_name: UserName,
    ) -> DomainResult<User> {
        // Specification 1: User ID must be unique
        let unique_id_spec = UniqueUserIdSpecification::new(self.repository.clone());
        if !unique_id_spec.is_satisfied_by(&user_id) {
            return Err(unique_id_spec.reason_for_dissatisfaction(&user_id)
                .map(crate::AppError::Validation)
                .unwrap_or_else(|| crate::AppError::Validation(
                    "User ID validation failed".to_string()
                )));
        }

        // Specification 2: User name must be unique
        let unique_name_spec =
            UniqueUserNameSpecification::new(self.repository.clone(), None);
        if !unique_name_spec.is_satisfied_by(&user_name) {
            return Err(unique_name_spec.reason_for_dissatisfaction(&user_name)
                .map(crate::AppError::Validation)
                .unwrap_or_else(|| crate::AppError::Validation(
                    "User name validation failed".to_string()
                )));
        }

        // Create the user aggregate (which already validates value objects)
        let user = User::new(user_id, user_name)?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;

    struct MockRepository;

    impl IRepository for MockRepository {
        fn get_by_id(&self, id: u32) -> DomainResult<User> {
            // Mock: always return not found
            Err(AppError::AggregateNotFound(id))
        }

        fn find_by_name(&self, _name: &str) -> DomainResult<Option<User>> {
            // Mock: always return None (user doesn't exist)
            Ok(None)
        }

        fn save(
            &self,
            _user: &User,
            _expected_version: i32,
        ) -> DomainResult<Vec<crate::UserEvent>> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn test_register_user_success() {
        let repo = Arc::new(MockRepository);
        let service = UserRegistrationService::new(repo);

        let user_id = UserId::new(1).unwrap();
        let user_name = UserName::new("Alice".to_string()).unwrap();

        let result = service.register_user(user_id, user_name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_user_invalid_id() {
        let repo = Arc::new(MockRepository);
        let _service = UserRegistrationService::new(repo);

        let result = UserId::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_user_invalid_name() {
        let repo = Arc::new(MockRepository);
        let _service = UserRegistrationService::new(repo);

        let result = UserName::new("".to_string());
        assert!(result.is_err());
    }
}
