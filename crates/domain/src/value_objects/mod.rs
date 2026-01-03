// Value Objects module
// Encapsulates domain concepts with validation and behavior

pub mod user_id;
pub mod user_name;
pub mod email;

pub use user_id::UserId;
pub use user_name::UserName;
pub use email::EmailAddress;
