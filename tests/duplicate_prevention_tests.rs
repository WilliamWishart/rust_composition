use rust_composition::{
    infrastructure::{MockLogger, DomainError},
    commands::{RegisterUserCommand, UserCommandHandler},
    events::{EventStore, EventBus},
    domain::Repository,
};
use std::sync::Arc;

/// Test helper to set up CQRS system
fn setup_cqrs_system() -> (EventStore, EventBus, UserCommandHandler, Arc<Repository>) {
    let logger = Arc::new(MockLogger::new());
    let event_store = EventStore::new();
    let event_bus = EventBus::new();
    let repository = Arc::new(Repository::new(event_store.clone()));

    let command_handler = UserCommandHandler::new(repository.clone(), event_bus, logger);

    (event_store, EventBus::new(), command_handler, repository)
}

#[tokio::test]
async fn test_duplicate_username_prevention() {
    let (_event_store, _event_bus, command_handler, _repository) = setup_cqrs_system();

    // Register first user with name "Bob"
    let cmd1 = RegisterUserCommand::new(1, "Bob".to_string())
        .expect("Command should be valid");

    let result1 = command_handler.handle_register_user(cmd1).await;
    assert!(
        result1.is_ok(),
        "First registration should succeed"
    );

    // Try to register another user with the same name "Bob"
    let cmd2 = RegisterUserCommand::new(2, "Bob".to_string())
        .expect("Command should be valid");

    let result2 = command_handler.handle_register_user(cmd2).await;

    // Should fail with validation error about duplicate username
    assert!(result2.is_err(), "Second registration with same name should fail");
    
    match result2.unwrap_err() {
        DomainError::Validation(msg) => {
            assert!(
                msg.contains("already taken"),
                "Error message should mention username is taken: {}",
                msg
            );
            assert!(
                msg.contains("Bob"),
                "Error message should contain the duplicate username"
            );
        }
        err => panic!("Expected Validation error, got: {:?}", err),
    }
}

#[tokio::test]
async fn test_duplicate_prevention_is_case_sensitive() {
    let (_event_store, _event_bus, command_handler, _repository) = setup_cqrs_system();

    // Register user "Alice"
    let cmd1 = RegisterUserCommand::new(1, "Alice".to_string())
        .expect("Command should be valid");

    command_handler.handle_register_user(cmd1).await
        .expect("First registration should succeed");

    // Try to register "alice" (lowercase) - should succeed since search is case-sensitive
    let cmd2 = RegisterUserCommand::new(2, "alice".to_string())
        .expect("Command should be valid");

    let result2 = command_handler.handle_register_user(cmd2).await;
    assert!(
        result2.is_ok(),
        "Registration with different case should succeed (case-sensitive)"
    );
}

#[tokio::test]
async fn test_different_usernames_allowed() {
    let (_event_store, _event_bus, command_handler, _repository) = setup_cqrs_system();

    // Register Bob
    let cmd1 = RegisterUserCommand::new(1, "Bob".to_string())
        .expect("Command should be valid");
    command_handler.handle_register_user(cmd1).await
        .expect("Bob registration should succeed");

    // Register Charlie - different name, should succeed
    let cmd2 = RegisterUserCommand::new(2, "Charlie".to_string())
        .expect("Command should be valid");
    let result2 = command_handler.handle_register_user(cmd2).await;
    assert!(result2.is_ok(), "Registration with different name should succeed");

    // Register Diana - different name, should succeed
    let cmd3 = RegisterUserCommand::new(3, "Diana".to_string())
        .expect("Command should be valid");
    let result3 = command_handler.handle_register_user(cmd3).await;
    assert!(result3.is_ok(), "Registration with different name should succeed");
}

#[tokio::test]
async fn test_duplicate_prevention_shows_existing_user_id() {
    let (_event_store, _event_bus, command_handler, _repository) = setup_cqrs_system();

    // Register user with specific ID
    let cmd1 = RegisterUserCommand::new(42, "ExistingUser".to_string())
        .expect("Command should be valid");
    command_handler.handle_register_user(cmd1).await
        .expect("First registration should succeed");

    // Try to register with same username but different ID
    let cmd2 = RegisterUserCommand::new(99, "ExistingUser".to_string())
        .expect("Command should be valid");

    let result2 = command_handler.handle_register_user(cmd2).await;

    match result2.unwrap_err() {
        DomainError::Validation(msg) => {
            assert!(
                msg.contains("42"),
                "Error message should show existing user's ID (42): {}",
                msg
            );
        }
        err => panic!("Expected Validation error, got: {:?}", err),
    }
}
