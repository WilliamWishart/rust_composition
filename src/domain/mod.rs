// Domain Layer: Business logic models and repositories
// This layer has NO external dependencies - it's pure business logic
// It depends only on infrastructure traits, not implementations

pub mod user_repository;
pub mod user_aggregate;
pub mod repository;

pub use user_repository::UserRepository;
pub use user_aggregate::User;
pub use repository::{Repository, IRepository};

