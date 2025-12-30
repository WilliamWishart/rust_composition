// ============================================================================
// APPLICATION ENTRY POINT (Main Binary)
// ============================================================================
//
// The main.rs file is kept thin and only responsible for:
// 1. Initializing the application
// 2. Wiring dependencies using the composition root
// 3. Running the application
//
// All business logic lives in lib.rs and its modules

use rust_composition::{AppBuilder, infrastructure::MockLogger};
use std::sync::Arc;

fn main() {
    println!("=== ENTERPRISE-SCALE APPLICATION STRUCTURE ===\n");

    // --- PRODUCTION COMPOSITION ---
    println!("--- PRODUCTION SETUP ---\n");
    let app = AppBuilder::new().build_user_service();

    println!("--- PRODUCTION EXECUTION ---\n");
    app.register_user(1, "Alice");
    app.register_user(2, "Bob");

    // --- TEST COMPOSITION ---
    println!("\n--- TEST SETUP ---\n");
    let mock_logger = Arc::new(MockLogger::new());
    let test_app = AppBuilder::new()
        .with_logger(mock_logger.clone())
        .build_user_service();

    println!("\n--- TEST EXECUTION ---\n");
    test_app.register_user(99, "TestUser");

    println!("\n=== ENTERPRISE STRUCTURE BENEFITS ===");
    println!("✓ Clear Module Hierarchy: infrastructure → domain → application → composition");
    println!("✓ Single Responsibility: Each module has one reason to change");
    println!("✓ Testability: Mock implementations, isolated layers");
    println!("✓ Scalability: Easy to add new features/services");
    println!("✓ Maintainability: Code is organized and discoverable");
    println!("✓ Reusability: lib.rs allows this to be used as a dependency");
}
