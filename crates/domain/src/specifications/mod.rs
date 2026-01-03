// Specification Pattern - Encapsulate reusable domain rules
// Allows composable business logic validation

pub mod user_specs;

pub use user_specs::{
    UniqueUserNameSpecification, ValidUserNameSpecification,
    UniqueUserIdSpecification,
};

/// Specification trait - Checks if a candidate satisfies a business rule
pub trait Specification<T> {
    /// Check if the candidate satisfies this specification
    fn is_satisfied_by(&self, candidate: &T) -> bool;

    /// Get a human-readable reason why the candidate doesn't satisfy
    fn reason_for_dissatisfaction(&self, candidate: &T) -> Option<String> {
        if self.is_satisfied_by(candidate) {
            None
        } else {
            Some("Specification not satisfied".to_string())
        }
    }

    /// Combine with another specification (AND logic)
    fn and<S: Specification<T> + 'static>(self, other: S) -> AndSpecification<T, Self, S>
    where
        Self: Sized,
    {
        AndSpecification {
            left: self,
            right: other,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Combine with another specification (OR logic)
    fn or<S: Specification<T> + 'static>(self, other: S) -> OrSpecification<T, Self, S>
    where
        Self: Sized,
    {
        OrSpecification {
            left: self,
            right: other,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// AND composition of two specifications
pub struct AndSpecification<T, Left: Specification<T>, Right: Specification<T>> {
    left: Left,
    right: Right,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, Left: Specification<T>, Right: Specification<T>> Specification<T>
    for AndSpecification<T, Left, Right>
{
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.left.is_satisfied_by(candidate) && self.right.is_satisfied_by(candidate)
    }

    fn reason_for_dissatisfaction(&self, candidate: &T) -> Option<String> {
        if !self.left.is_satisfied_by(candidate) {
            self.left.reason_for_dissatisfaction(candidate)
        } else if !self.right.is_satisfied_by(candidate) {
            self.right.reason_for_dissatisfaction(candidate)
        } else {
            None
        }
    }
}

/// OR composition of two specifications
pub struct OrSpecification<T, Left: Specification<T>, Right: Specification<T>> {
    left: Left,
    right: Right,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, Left: Specification<T>, Right: Specification<T>> Specification<T>
    for OrSpecification<T, Left, Right>
{
    fn is_satisfied_by(&self, candidate: &T) -> bool {
        self.left.is_satisfied_by(candidate) || self.right.is_satisfied_by(candidate)
    }

    fn reason_for_dissatisfaction(&self, candidate: &T) -> Option<String> {
        if self.is_satisfied_by(candidate) {
            None
        } else {
            let left_reason = self.left.reason_for_dissatisfaction(candidate);
            let right_reason = self.right.reason_for_dissatisfaction(candidate);
            Some(format!(
                "Neither condition satisfied: {:?}, {:?}",
                left_reason, right_reason
            ))
        }
    }
}
