// Domain Layer - Pure business logic, no external dependencies except std, serde, chrono
// This crate contains:
// - Aggregates (User)
// - Domain Events
// - Value Objects and Constraints
// - Repository trait (implementation in persistence crate)
// - Errors

pub mod errors;
pub mod events;
pub mod aggregates;
pub mod repository;
pub mod commands;
pub mod value_objects;
pub mod specifications;
pub mod services;

pub use errors::{AppError, DomainError, DomainResult};
pub use events::UserEvent;
pub use aggregates::User;
pub use repository::IRepository;
pub use commands::{RegisterUserCommand, RenameUserCommand};
pub use value_objects::{UserId, UserName, EmailAddress};
pub use specifications::Specification;
pub use services::UserRegistrationService;
