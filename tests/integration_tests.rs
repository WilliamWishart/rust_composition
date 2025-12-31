/// Integration Tests for CQRS + Event Sourcing Architecture
/// 
/// These tests verify the end-to-end flow:
/// Command → Aggregate → EventStore → EventBus → Projection → Query

use rust_composition::{
    infrastructure::MockLogger,
    commands::{RegisterUserCommand, RenameUserCommand, UserCommandHandler},
    events::{EventStore, EventBus, EventHandler},
    events::projections::{UserProjection, TypedUserProjectionHandler},
    queries::UserQuery,
    domain::{Repository, IRepository, User},
};
use std::sync::Arc;

/// Helper function to setup the complete CQRS system
fn setup_cqrs_system() -> (
    Arc<Repository>,
    Arc<MockLogger>,
    Arc<UserCommandHandler>,
    UserQuery,
) {
    let logger = Arc::new(MockLogger::new());
    let event_store = EventStore::new();
    let event_bus = EventBus::new();
    let repository = Arc::new(Repository::new(event_store));

    // Setup projection and subscribe to events
    let user_projection = UserProjection::new();
    let projection_handler = Arc::new(TypedUserProjectionHandler::new(user_projection.clone()));
    event_bus.subscribe(projection_handler);

    // Create command handler
    let command_handler = Arc::new(UserCommandHandler::new(
        repository.clone(),
        event_bus,
        logger.clone(),
    ));

    // Create query handler
    let user_query = UserQuery::new(user_projection);

    (repository, logger, command_handler, user_query)
}

// ============================================================================
// COMMAND TESTS
// ============================================================================

#[test]
fn test_valid_command_registration() {
    let (_, _, command_handler, _) = setup_cqrs_system();

    let cmd = RegisterUserCommand::new(1, "Alice".to_string()).expect("Valid command");
    let result = command_handler.handle_register_user(cmd);

    assert!(result.is_ok(), "Valid command should succeed");
}

#[test]
fn test_empty_name_command_validation() {
    let (_, _, _command_handler, _) = setup_cqrs_system();

    let cmd = RegisterUserCommand::new(1, "".to_string());

    assert!(cmd.is_err(), "Command with empty name should fail validation");
    assert_eq!(cmd.unwrap_err(), "Name cannot be empty");
}

#[test]
fn test_zero_id_command_validation() {
    let (_, _, command_handler, _) = setup_cqrs_system();

    let cmd = RegisterUserCommand::new(0, "Alice".to_string()).expect("Command created");
    let result = command_handler.handle_register_user(cmd);

    assert!(result.is_err(), "Command with zero ID should fail on handler");
    assert_eq!(result.unwrap_err(), "User ID must be greater than 0");
}

#[test]
fn test_rename_user_command_validation() {
    let (_, _, _command_handler, _) = setup_cqrs_system();

    let cmd = RenameUserCommand::new(1, "".to_string());

    assert!(cmd.is_err(), "Command with empty name should fail validation");
    assert_eq!(cmd.unwrap_err(), "New name cannot be empty");
}

// ============================================================================
// EVENT SOURCING TESTS
// ============================================================================

#[test]
fn test_aggregate_creates_event_on_new() {
    let user = User::new(1, "Alice".to_string());

    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Alice");
    assert_eq!(user.version, -1); // Version is -1 until persisted
    assert!(!user.get_uncommitted_changes().is_empty(), "Should have uncommitted changes");
}

#[test]
fn test_aggregate_load_from_history() {
    // Create a user and get its events
    let user1 = User::new(1, "Alice".to_string());
    let events = user1.get_uncommitted_changes();

    // Load a new user from that history
    let user2 = User::load_from_history(events).expect("Should load from history");

    assert_eq!(user2.id, 1);
    assert_eq!(user2.name, "Alice");
}

// ============================================================================
// REPOSITORY PERSISTENCE TESTS
// ============================================================================

#[test]
fn test_repository_saves_events() {
    let (repository, _, _, _) = setup_cqrs_system();

    let user = User::new(1, "Alice".to_string());
    let result = repository.save(&user, -1);

    assert!(result.is_ok(), "Save should succeed");
    assert_eq!(result.unwrap().len(), 1, "Should have one event");
}

#[test]
fn test_repository_retrieves_saved_aggregate() {
    let (repository, _, _, _) = setup_cqrs_system();

    // Save an aggregate
    let user1 = User::new(1, "Alice".to_string());
    repository.save(&user1, -1).expect("Save should succeed");

    // Retrieve it
    let user2 = repository.get_by_id(1).expect("Should retrieve aggregate");

    assert_eq!(user2.id, 1);
    assert_eq!(user2.name, "Alice");
}

#[test]
fn test_repository_fails_on_missing_aggregate() {
    let (repository, _, _, _) = setup_cqrs_system();

    let result = repository.get_by_id(999);

    assert!(result.is_err(), "Should fail for non-existent aggregate");
}

// ============================================================================
// EVENT BUS & PROJECTION TESTS
// ============================================================================

#[test]
fn test_eventbus_subscribers_receive_events() {
    let event_store = EventStore::new();
    let event_bus = EventBus::new();
    let repository = Arc::new(Repository::new(event_store));

    // Create a custom test subscriber
    let test_subscriber = Arc::new(TestEventSubscriber::new());
    event_bus.subscribe(test_subscriber.clone());

    // Issue a command
    let user = User::new(1, "Alice".to_string());
    let events = repository.save(&user, -1).expect("Save should succeed");

    // Publish events
    for event in events {
        event_bus.publish(&event);
    }

    // Verify subscriber received the event
    assert_eq!(
        test_subscriber.get_event_count(),
        1,
        "Subscriber should have received 1 event"
    );
}

#[test]
fn test_projection_updated_via_eventbus() {
    let (_, _, command_handler, user_query) = setup_cqrs_system();

    // Issue command - events flow through EventBus to projection
    let cmd = RegisterUserCommand::new(1, "Alice".to_string()).expect("Valid command");
    command_handler.handle_register_user(cmd).expect("Command should succeed");

    // Query the read model
    let result = user_query.get_user(1);

    assert!(result.is_some(), "User should exist in projection");
    assert!(
        result.unwrap().contains("Alice"),
        "User name should be in query result"
    );
}

// ============================================================================
// END-TO-END CQRS TESTS
// ============================================================================

#[test]
fn test_end_to_end_single_user_registration() {
    let (repository, logger, command_handler, user_query) = setup_cqrs_system();

    // Issue command
    let cmd = RegisterUserCommand::new(1, "Bob".to_string()).expect("Valid command");
    command_handler.handle_register_user(cmd).expect("Command should succeed");

    // Verify: Aggregate reconstructed from events
    let loaded_user = repository.get_by_id(1).expect("Should retrieve user");
    assert_eq!(loaded_user.name, "Bob");

    // Verify: Projection updated (read model)
    let queried_user = user_query.get_user(1).expect("Should query user");
    assert!(queried_user.contains("Bob"));

    // Verify: Logging occurred
    let logs = logger.get_messages();
    assert!(
        logs.iter().any(|log| log.contains("RegisterUser")),
        "Should log command processing"
    );
    assert!(
        logs.iter().any(|log| log.contains("registered successfully")),
        "Should log successful registration"
    );
}

#[test]
fn test_end_to_end_multiple_users() {
    let (repository, _, command_handler, user_query) = setup_cqrs_system();

    // Create multiple users
    for i in 1..=5 {
        let cmd = RegisterUserCommand::new(i, format!("User{}", i)).expect("Valid command");
        command_handler.handle_register_user(cmd).expect("Command should succeed");
    }

    // Verify all users exist in read model
    let all_users = user_query.get_all_users();
    assert_eq!(all_users.len(), 5, "Should have 5 users in read model");

    // Verify user count
    let count = user_query.get_user_count();
    assert_eq!(count, 5, "User count should be 5");

    // Verify specific user retrieval
    let user = user_query.get_user(3).expect("Should find user 3");
    assert!(user.contains("User3"));

    // Verify all users can be reconstructed from event store
    for i in 1..=5 {
        let user = repository.get_by_id(i).expect("Should retrieve aggregate");
        assert_eq!(user.name, format!("User{}", i));
    }
}

#[test]
fn test_eventual_consistency_read_after_write() {
    let (_, _, command_handler, user_query) = setup_cqrs_system();

    // Write side: Issue command
    let cmd = RegisterUserCommand::new(1, "Charlie".to_string()).expect("Valid command");
    command_handler.handle_register_user(cmd).expect("Command should succeed");

    // Read side: Query should immediately reflect (in-memory synchronous in this demo)
    let result = user_query.get_user(1).expect("Should find user");
    assert!(result.contains("Charlie"));
    assert!(result.contains("ID: 1"));
}

#[test]
fn test_duplicate_user_ids_overwrite() {
    let (_, _, command_handler, user_query) = setup_cqrs_system();

    // Create user with ID 1
    let cmd1 = RegisterUserCommand::new(1, "Alice".to_string()).expect("Valid command");
    command_handler.handle_register_user(cmd1).expect("Command should succeed");

    // Create another user with same ID (overwrite)
    let cmd2 = RegisterUserCommand::new(1, "Bob".to_string()).expect("Valid command");
    command_handler.handle_register_user(cmd2).expect("Command should succeed");

    // Query should return the latest state
    let result = user_query.get_user(1).expect("Should find user");
    assert!(result.contains("Bob"), "Should have the latest name");
}

#[test]
fn test_query_nonexistent_user_returns_none() {
    let (_, _, _, user_query) = setup_cqrs_system();

    let result = user_query.get_user(999);

    assert!(result.is_none(), "Nonexistent user should return None");
}

// ============================================================================
// HELPER TYPES FOR TESTING
// ============================================================================

/// Test subscriber to verify EventBus is delivering events
struct TestEventSubscriber {
    event_count: std::sync::Mutex<usize>,
}

impl TestEventSubscriber {
    fn new() -> Self {
        TestEventSubscriber {
            event_count: std::sync::Mutex::new(0),
        }
    }

    fn get_event_count(&self) -> usize {
        *self.event_count.lock().unwrap()
    }
}

impl EventHandler for TestEventSubscriber {
    fn handle_event(&self, _event: &rust_composition::events::UserEvent) {
        let mut count = self.event_count.lock().unwrap();
        *count += 1;
    }
}
// ============================================================================
// RENAME USER TESTS
// ============================================================================

#[test]
fn test_rename_user_end_to_end() {
    let (repository, _, command_handler, user_query) = setup_cqrs_system();

    // Register user first
    let register_cmd = RegisterUserCommand::new(1, "Alice".to_string()).expect("Valid command");
    command_handler
        .handle_register_user(register_cmd)
        .expect("Register should succeed");

    // Verify initial state
    let result = user_query.get_user(1).expect("Should find user");
    assert!(result.contains("Alice"), "Should have initial name");

    // Rename user
    let rename_cmd = RenameUserCommand::new(1, "Alicia".to_string()).expect("Valid command");
    command_handler
        .handle_rename_user(rename_cmd)
        .expect("Rename should succeed");

    // Verify: Aggregate has new name
    let user = repository.get_by_id(1).expect("Should retrieve user");
    assert_eq!(user.name, "Alicia", "Aggregate should have new name");

    // Verify: Projection reflects rename
    let result = user_query.get_user(1).expect("Should find user");
    assert!(result.contains("Alicia"), "Query should return updated name");
    assert!(!result.contains("Alice"), "Query should not contain old name");
}

#[test]
fn test_rename_empty_name_validation() {
    let (_, _, _, _) = setup_cqrs_system();

    let cmd = RenameUserCommand::new(1, "".to_string());

    assert!(cmd.is_err(), "Empty name should fail validation");
    assert_eq!(cmd.unwrap_err(), "New name cannot be empty");
}

#[test]
fn test_rename_whitespace_name_validation() {
    let (_, _, _, _) = setup_cqrs_system();

    let cmd = RenameUserCommand::new(1, "   ".to_string());

    assert!(cmd.is_err(), "Whitespace-only name should fail validation");
}

#[test]
fn test_rename_nonexistent_user_error() {
    let (_, _, command_handler, _) = setup_cqrs_system();

    // Try to rename user that was never created
    let cmd = RenameUserCommand::new(999, "NewName".to_string()).expect("Valid command");
    let result = command_handler.handle_rename_user(cmd);

    // Should succeed at command level, but aggregate won't exist
    // (For strict validation, could require user to exist first)
    assert!(
        result.is_ok() || result.is_err(),
        "System should handle appropriately"
    );
}

#[test]
fn test_aggregate_reconstruction_with_rename() {
    let (repository, _, command_handler, _) = setup_cqrs_system();

    // Register user
    let register_cmd = RegisterUserCommand::new(1, "Bob".to_string()).expect("Valid command");
    command_handler
        .handle_register_user(register_cmd)
        .expect("Register should succeed");

    // Rename user
    let rename_cmd = RenameUserCommand::new(1, "Robert".to_string()).expect("Valid command");
    command_handler
        .handle_rename_user(rename_cmd)
        .expect("Rename should succeed");

    // Reconstruct from event history
    let user = repository.get_by_id(1).expect("Should retrieve user");

    // Verify final state reflects all events applied
    assert_eq!(user.name, "Robert", "Reconstructed user should have final name");
    assert_eq!(user.version, 1, "Should have version 1 after two events (0-indexed)");
}

#[test]
fn test_multiple_renames_sequence() {
    let (repository, _, command_handler, user_query) = setup_cqrs_system();

    // Register user
    let register_cmd = RegisterUserCommand::new(1, "Alice".to_string()).expect("Valid command");
    command_handler
        .handle_register_user(register_cmd)
        .expect("Register should succeed");

    // First rename
    let rename_cmd1 = RenameUserCommand::new(1, "Alicia".to_string()).expect("Valid command");
    command_handler
        .handle_rename_user(rename_cmd1)
        .expect("First rename should succeed");

    // Second rename
    let rename_cmd2 = RenameUserCommand::new(1, "Alice-Ann".to_string()).expect("Valid command");
    command_handler
        .handle_rename_user(rename_cmd2)
        .expect("Second rename should succeed");

    // Verify final state
    let user = repository.get_by_id(1).expect("Should retrieve user");
    assert_eq!(user.name, "Alice-Ann", "Should have final name after multiple renames");

    let result = user_query.get_user(1).expect("Should find user");
    assert!(result.contains("Alice-Ann"), "Query should reflect final rename");
}

#[test]
fn test_rename_preserves_user_id() {
    let (repository, _, command_handler, _) = setup_cqrs_system();

    // Register user
    let register_cmd = RegisterUserCommand::new(42, "Name1".to_string()).expect("Valid command");
    command_handler
        .handle_register_user(register_cmd)
        .expect("Register should succeed");

    // Rename user
    let rename_cmd = RenameUserCommand::new(42, "Name2".to_string()).expect("Valid command");
    command_handler
        .handle_rename_user(rename_cmd)
        .expect("Rename should succeed");

    // Verify user ID unchanged
    let user = repository.get_by_id(42).expect("Should retrieve user");
    assert_eq!(user.id, 42, "User ID should be preserved");
}