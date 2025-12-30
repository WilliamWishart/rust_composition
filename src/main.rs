// ============================================================================
// APPLICATION ENTRY POINT (Main Binary)
// ============================================================================
//
// Demonstrates CQRS with Event Sourcing and Eventual Consistency
// Shows the complete flow: Command → Event → Projection → Query

use rust_composition::{
    infrastructure::MockLogger, commands::RegisterUserCommand,
    events::{EventStore, EventBus, UserRegisteredEvent},
    events::projections::UserProjection,
    events::projections::TypedUserProjectionHandler,
    queries::UserQuery, UserCommandHandler,
};
use std::sync::Arc;

fn main() {
    println!("=== ENTERPRISE DDD WITH CQRS + EVENT SOURCING ===\n");

    // --- SETUP: Create infrastructure ---
    println!("--- SETUP: Infrastructure ---\n");
    let logger = Arc::new(MockLogger::new());
    let event_store = EventStore::new();
    let event_bus = EventBus::new();

    // Setup projection (read model)
    let user_projection = UserProjection::new();
    let projection_handler = TypedUserProjectionHandler::new(user_projection.clone());

    // Create command handler (write side)
    let command_handler = UserCommandHandler::new(
        event_store.clone(),
        event_bus.clone(),
        logger.clone(),
    );

    // Create query handler (read side)
    let user_query = UserQuery::new(user_projection.clone());

    println!("✓ Event Store initialized");
    println!("✓ Event Bus initialized");
    println!("✓ User Projection (Read Model) initialized");
    println!("✓ Command Handler (Write Side) initialized");
    println!("✓ Query Handler (Read Side) initialized\n");

    // --- EXECUTION: Commands (Write Side) ---
    println!("--- COMMANDS (Write Side) ---\n");
    
    // Execute command to register user
    let cmd1 = RegisterUserCommand::new(1, "Alice".to_string())
        .expect("Command validation failed");
    println!("📝 Command: {}", cmd1.name);
    
    if let Err(e) = command_handler.handle_register_user(cmd1) {
        println!("❌ Command failed: {}", e);
    } else {
        // Manually update projection (in real system, event bus would do this async)
        let event1 = UserRegisteredEvent::new(1, "Alice".to_string());
        projection_handler.handle_user_registered(&event1);
        println!("✓ Command successful - Event stored & Projection updated (Eventual Consistency)\n");
    }

    let cmd2 = RegisterUserCommand::new(2, "Bob".to_string())
        .expect("Command validation failed");
    println!("📝 Command: {}", cmd2.name);
    
    if let Err(e) = command_handler.handle_register_user(cmd2) {
        println!("❌ Command failed: {}", e);
    } else {
        let event2 = UserRegisteredEvent::new(2, "Bob".to_string());
        projection_handler.handle_user_registered(&event2);
        println!("✓ Command successful - Event stored & Projection updated (Eventual Consistency)\n");
    }

    // Demonstrate command validation failure
    let cmd_invalid = RegisterUserCommand::new(3, "".to_string());
    if let Err(e) = cmd_invalid {
        println!("⚠️  Invalid command rejected: {}\n", e);
    }

    // --- STATE: Event Store ---
    println!("--- EVENT STORE (Source of Truth) ---\n");
    println!("Total events stored: {}", event_store.event_count());
    println!("Events are immutable facts about what happened\n");

    // --- QUERIES: Read Side (Eventually Consistent) ---
    println!("--- QUERIES (Read Side - Eventually Consistent) ---\n");
    
    if let Some(user) = user_query.get_user(1) {
        println!("✓ Query: Get User(1) → {}", user);
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

