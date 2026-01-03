// User-specific specifications for domain validation rules

use crate::value_objects::{UserId, UserName};
use crate::repository::IRepository;
use super::Specification;
use std::sync::Arc;

/// Specification: UserName must be unique across all users
pub struct UniqueUserNameSpecification {
    repository: Arc<dyn IRepository>,
    excluded_user_id: Option<UserId>,
}

impl UniqueUserNameSpecification {
    pub fn new(
        repository: Arc<dyn IRepository>,
        excluded_user_id: Option<UserId>,
    ) -> Self {
        UniqueUserNameSpecification {
            repository,
            excluded_user_id,
        }
    }
}

impl Specification<UserName> for UniqueUserNameSpecification {
    fn is_satisfied_by(&self, candidate: &UserName) -> bool {
        match self.repository.find_by_name(candidate.value()) {
            Ok(None) => true,
            Ok(Some(user)) => {
                // If user exists, only satisfied if it's the excluded user
                self.excluded_user_id
                    .map(|excluded_id| user.id() == excluded_id)
                    .unwrap_or(false)
            }
            Err(_) => false,
        }
    }

    fn reason_for_dissatisfaction(&self, candidate: &UserName) -> Option<String> {
        if !self.is_satisfied_by(candidate) {
            // Try to get the existing user's ID for a more informative error message
            match self.repository.find_by_name(candidate.value()) {
                Ok(Some(user)) => {
                    Some(format!(
                        "Username '{}' is already taken by user ID {}",
                        candidate.value(),
                        user.id().value()
                    ))
                }
                _ => {
                    Some(format!(
                        "Username '{}' is already taken",
                        candidate.value()
                    ))
                }
            }
        } else {
            None
        }
    }
}

/// Specification: UserName must be valid for registration
pub struct ValidUserNameSpecification;

impl Specification<UserName> for ValidUserNameSpecification {
    fn is_satisfied_by(&self, candidate: &UserName) -> bool {
        // Already enforced by UserName VO constructor, but can add additional rules
        candidate.is_valid_for_registration()
    }

    fn reason_for_dissatisfaction(&self, candidate: &UserName) -> Option<String> {
        if !self.is_satisfied_by(candidate) {
            Some(format!(
                "Username '{}' is not valid for registration",
                candidate.value()
            ))
        } else {
            None
        }
    }
}

/// Specification: UserId must be unique across all users
pub struct UniqueUserIdSpecification {
    repository: Arc<dyn IRepository>,
}

impl UniqueUserIdSpecification {
    pub fn new(repository: Arc<dyn IRepository>) -> Self {
        UniqueUserIdSpecification { repository }
    }
}

impl Specification<UserId> for UniqueUserIdSpecification {
    fn is_satisfied_by(&self, candidate: &UserId) -> bool {
        match self.repository.get_by_id(candidate.value()) {
            Ok(_) => false, // User with this ID already exists
            Err(_) => true, // User not found, ID is unique
        }
    }

    fn reason_for_dissatisfaction(&self, candidate: &UserId) -> Option<String> {
        if !self.is_satisfied_by(candidate) {
            Some(format!("User ID {} already exists", candidate.value()))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;

    struct MockRepository;

    impl IRepository for MockRepository {
        fn get_by_id(&self, id: u32) -> crate::DomainResult<crate::User> {
            if id == 1 {
                Err(AppError::AggregateNotFound(id))
            } else {
                Err(AppError::AggregateNotFound(id))
            }
        }

        fn find_by_name(&self, name: &str) -> crate::DomainResult<Option<crate::User>> {
            if name == "Alice" {
                Err(AppError::RepositoryError("Not implemented in mock".to_string()))
            } else {
                Ok(None)
            }
        }

        fn save(
            &self,
            _user: &crate::User,
            _expected_version: i32,
        ) -> crate::DomainResult<Vec<crate::UserEvent>> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn test_unique_user_name_specification() {
        let repo = Arc::new(MockRepository);
        let spec = UniqueUserNameSpecification::new(repo, None);

        let valid_name = UserName::new("Bob".to_string()).unwrap();
        assert!(spec.is_satisfied_by(&valid_name));
    }

    #[test]
    fn test_valid_user_name_specification() {
        let spec = ValidUserNameSpecification;
        let valid_name = UserName::new("Alice".to_string()).unwrap();
        assert!(spec.is_satisfied_by(&valid_name));
    }

    #[test]
    fn test_specifications_composition() {
        let repo = Arc::new(MockRepository);
        let unique_spec = UniqueUserNameSpecification::new(repo, None);
        let valid_spec = ValidUserNameSpecification;

        let combined = valid_spec.and(unique_spec);
        let name = UserName::new("Bob".to_string()).unwrap();
        assert!(combined.is_satisfied_by(&name));
    }
}
