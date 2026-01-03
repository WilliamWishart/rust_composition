// Domain Services module
// Services encapsulating business logic that involves multiple aggregates or repositories

pub mod user_registration_service;

pub use user_registration_service::UserRegistrationService;
