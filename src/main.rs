// ============================================================================
// APPLICATION ENTRY POINT (Main Binary)
// ============================================================================
//
// Demonstrates CQRS with Event Sourcing and Eventual Consistency following
// Gregory Young's m-r pattern:
// - Aggregates apply events and accumulate changes
// - Repository reconstructs aggregates from event history
// - Optimistic locking prevents concurrency violations
// - Commands → Aggregates → Events → Projections → Queries

use rust_composition::{
    infrastructure::MockLogger, commands::{RegisterUserCommand, RenameUserCommand},
    events::{EventStore, EventBus},
    events::projections::{UserProjection, TypedUserProjectionHandler},
    queries::UserQuery, commands::UserCommandHandler,
    domain::{Repository, IRepository},
};
use std::sync::Arc;

fn main() {
    println!("=== ENTERPRISE DDD WITH CQRS + EVENT SOURCING (m-r pattern) ===\n");

    // --- SETUP: Create infrastructure ---
    println!("--- SETUP: Infrastructure ---\n");
    let logger = Arc::new(MockLogger::new());
    let event_store = EventStore::new();
    let event_bus = EventBus::new();
    let repository = Arc::new(Repository::new(event_store.clone()));

    // Setup projection (read model)
    let user_projection = UserProjection::new();
    let projection_handler = Arc::new(TypedUserProjectionHandler::new(user_projection.clone()));

    // Register projection handler as an EventBus subscriber
    event_bus.subscribe(projection_handler.clone());

    // Create command handler (write side) - uses repository pattern
    let command_handler = UserCommandHandler::new(
        repository.clone(),
        event_bus.clone(),
        logger.clone(),
    );

    // Create query handler (read side)
    let user_query = UserQuery::new(user_projection.clone());

    println!("✓ Event Store initialized (source of truth)");
    println!("✓ Event Bus initialized (pub/sub)");
    println!("✓ Repository initialized (aggregate persistence)");
    println!("✓ User Projection (Read Model) initialized");
    println!("✓ Command Handler (Write Side) initialized");
    println!("✓ Query Handler (Read Side) initialized");
    println!("✓ Projection Handler subscribed to EventBus\n");

    // --- EXECUTION: Commands (Write Side) ---
    println!("--- COMMANDS (Write Side) ---\n");
    
    // Execute command to register user
    let cmd1 = RegisterUserCommand::new(1, "Alice".to_string())
        .expect("Command validation failed");
    println!("📝 Command: Create User '{}'", cmd1.name);
    
    match command_handler.handle_register_user(cmd1) {
        Ok(()) => {
            println!("✓ Command processed");
            println!("  - Aggregate created from command");
            println!("  - Event appended to EventStore");
            println!("  - EventBus published to all subscribers");
            println!("  - Projection updated (Eventual Consistency)\n");
        }
        Err(e) => println!("❌ Command failed: {}\n", e),
    }

    let cmd2 = RegisterUserCommand::new(2, "Bob".to_string())
        .expect("Command validation failed");
    println!("📝 Command: Create User '{}'", cmd2.name);
    
    match command_handler.handle_register_user(cmd2) {
        Ok(()) => {
            println!("✓ Command processed");
            println!("  - Aggregate created from command");
            println!("  - Event appended to EventStore");
            println!("  - Projection updated (Eventual Consistency)\n");
        }
        Err(e) => println!("❌ Command failed: {}\n", e),
    }

    // Demonstrate command validation failure
    let cmd_invalid = RegisterUserCommand::new(3, "".to_string());
    if let Err(e) = cmd_invalid {
        println!("⚠️  Invalid command rejected: {}\n", e);
    }

    // --- RENAME COMMANDS (Write Side) ---
    println!("--- RENAME COMMANDS (Write Side) ---\n");

    let rename_cmd = RenameUserCommand::new(1, "Alicia".to_string())
        .expect("Command validation failed");
    println!("📝 Command: Rename User 1 to '{}'", rename_cmd.new_name);
    
    match command_handler.handle_rename_user(rename_cmd) {
        Ok(()) => {
            println!("✓ Command processed");
            println!("  - Aggregate loaded from event history");
            println!("  - New event appended to EventStore");
            println!("  - EventBus published to all subscribers");
            println!("  - Projection updated (Eventual Consistency)\n");
        }
        Err(e) => println!("❌ Command failed: {}\n", e),
    }

    // Demonstrate rename validation failure
    let invalid_rename = RenameUserCommand::new(2, "".to_string());
    if let Err(e) = invalid_rename {
        println!("⚠️  Invalid rename command rejected: {}\n", e);
    }

    // --- STATE: Event Store ---
    println!("--- EVENT STORE (Source of Truth) ---\n");
    println!("Total events stored: {}", event_store.event_count());
    println!("Events are immutable, append-only log of all domain changes\n");

    // --- RECONSTRUCTION: Load aggregate from history ---
    println!("--- AGGREGATE RECONSTRUCTION (Event Sourcing) ---\n");
    match repository.get_by_id(1) {
        Ok(user) => {
            println!("✓ Loaded User(1) from event history:");
            println!("  - ID: {}", user.id);
            println!("  - Name: {}", user.name);
            println!("  - Version: {} (incremented per event)", user.version + 1);
            println!("  - Events in history: {}", user.get_uncommitted_changes().len());
        }
        Err(e) => println!("❌ Failed to load user: {}", e),
    }
    println!();

    // --- QUERIES: Read Side (Eventually Consistent) ---
    println!("--- QUERIES (Read Side - Eventually Consistent) ---\n");
    
    if let Some(user) = user_query.get_user(1) {
        println!("✓ Query: Get User(1) → {} (After Rename)", user);
    }
    
    if let Some(user) = user_query.get_user(2) {
        println!("✓ Query: Get User(2) → {}", user);
    }
    
    let all_users = user_query.get_all_users();
    println!("\n✓ Query: Get All Users");
    for user in all_users {
        println!("  - {}", user);
    }
    
    println!("\nTotal users in read model: {}", user_query.get_user_count());

    // --- DEMONSTRATE CQRS + EVENT SOURCING BENEFITS ---
    println!("\n=== CQRS + EVENT SOURCING BENEFITS ===");
    println!("✓ Command-Query Separation: Different models for reads/writes");
    println!("✓ Event Sourcing: Complete audit trail of all changes");
    println!("✓ Eventual Consistency: Read model eventually matches write model");
    println!("✓ Temporal Queries: Can reconstruct state at any point in time");
    println!("✓ Scalability: Read and write models can scale independently");
    println!("✓ Testability: Commands produce predictable events");
}

