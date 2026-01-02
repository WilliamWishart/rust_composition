# Enterprise-Scale Rust Application - DDD/CQRS/Event Sourcing

> **📚 Learning Example** - This is an educational project to support learning of **Dependency Injection (DI)**, **Domain-Driven Design (DDD)**, **CQRS** (Command Query Responsibility Segregation), and **Event Sourcing** patterns in Rust. It aims to act as a potential reference for how to structure large-scale, enterprise applications.

## 📁 Project Structure

```
rust_composition/
├── Cargo.toml                          # Project manifest
├── src/
│   ├── lib.rs                          # Library root - exports all modules
│   ├── main.rs                         # Application entry point (thin wrapper)
│   ├── infrastructure/                 # External dependencies & adapters
│   │   ├── mod.rs                      # Module definition & re-exports
│   │   └── logger.rs                   # Logger trait & implementations
│   ├── domain/                         # Business logic (core domain)
│   │   ├── mod.rs                      # Module definition & re-exports
│   │   ├── user_repository.rs          # Data access abstraction
│   │   └── user_aggregate.rs           # Domain models (User aggregate)
│   ├── events/                         # Event Sourcing (immutable event log)
│   │   ├── mod.rs                      # Module definition & re-exports
│   │   ├── event_store.rs              # Event store (source of truth)
│   │   ├── event_bus.rs                # Event bus (pub/sub)
│   │   └── projections.rs              # Read models (eventual consistency)
│   ├── commands/                       # CQRS Write Side
│   │   ├── mod.rs                      # Module definition & re-exports
│   │   ├── register_user_command.rs    # Command definitions
│   │   └── command_handler.rs          # Command processors
│   ├── queries/                        # CQRS Read Side
│   │   ├── mod.rs                      # Module definition & re-exports
│   │   └── user_queries.rs             # Query handlers
│   ├── application/                    # Use cases & services
│   │   ├── mod.rs                      # Module definition & re-exports
│   │   └── user_service.rs             # Business logic orchestration
│   └── composition/                    # Dependency wiring (Composition Root)
│       ├── mod.rs                      # Module definition & re-exports
│       └── app_builder.rs              # Builder pattern for DI
├── tests/                              # Integration tests
└── README.md                           # This file
```

## 🏗️ Architecture Overview

This project follows a **Layered Architecture** with **CQRS** and **Event Sourcing**:

```
┌────────────────────────────────────────────┐
│            CQRS PATTERN                    │
├────────────────────────────────────────────┤
│  Commands (Write Side)  │  Queries (Read)  │
│  - RegisterUserCommand  │  - UserQuery     │
│  - Command Handlers     │  - Projections   │
│     ↓                   │        ↑          │
│  EVENT STORE (Event Sourcing - Source of Truth)
│     ↓                   │        ↑          │
│  Write Model            │  Read Model      │
│  (Commands emit)        │  (Eventual       │
│  - Domain Events        │   Consistency)   │
│  - Immutable Log        │  - User Proj.    │
└────────────────────────────────────────────┘
                    ↓
        Application & Domain Logic
```

## 📚 Layer Responsibilities

**Files:**
- `logger.rs` - Logger trait with `ConsoleLogger` and `MockLogger` implementations

**Key Principle:** 
- Contains **traits** that define "what" (contracts)
- Contains **implementations** that define "how" (adaptability)
- No business logic - purely technical infrastructure
- Used by all other layers through trait abstraction

```rust
// Usage example:
pub trait Logger: Send + Sync {
    fn log(&self, message: &str);
}

pub struct ConsoleLogger;  // Production implementation
pub struct MockLogger;     // Test implementation
```

### 2. **Domain Layer** (`src/domain/`)

Contains the core business logic and domain models - the "heart" of your application.

**Files:**
- `user_repository.rs` - Data access abstraction (Repository pattern)
- `user_aggregate.rs` - Domain models (User aggregate root)

**Key Principle:**
- **No external dependencies** - Only depends on infrastructure traits, not implementations
- Pure business logic
- Testable in isolation
- Represents your business rules and entities

```rust
// In CQRS, repositories use event sourcing:
pub struct Repository {
    event_store: EventStore,
}

impl IRepository for Repository {
    fn save(&self, aggregate: &User, expected_version: i32) -> Result<Vec<UserEvent>, String> {
        // Save events, not snapshots
    }
    fn get_by_id(&self, id: u32) -> Result<User, String> {
        // Load from event history
    }
}
```

### 3. **Commands Layer** (`src/commands/`)

CQRS write-side handlers that process commands through aggregates.

**Files:**
- `register_user_command.rs` - Command definitions with validation
- `command_handler.rs` - Handlers that apply commands to aggregates

**Key Principle:**
- Commands represent intentions to change state
- Commands are validated before processing
- Processing produces domain events
- Events are published via EventBus

```rust
pub struct UserCommandHandler {
    repository: Arc<Repository>,
    event_bus: EventBus,
    logger: Arc<dyn Logger>,
}

impl UserCommandHandler {
    pub fn handle_register_user(&self, command: RegisterUserCommand) -> Result<(), String> {
        let user = User::new(command.user_id, command.name);
        self.repository.save(&user, -1)?;
        // Events published via EventBus
    }
}
```

### 4. **Queries Layer** (`src/queries/`)

CQRS read-side handlers that query from projections (read models).

**Files:**
- `user_queries.rs` - Query handlers for read-side

**Key Principle:**
- Queries read from eventually-consistent projections
- Never modify state
- Multiple projections can exist for different use cases
- Queries are fast because read models are optimized for reading

### 5. **Composition Root Layer** (`src/composition/`)

The **only place that knows about concrete implementations**. Responsible for wiring all dependencies.

**Files:**
- `app_builder.rs` - Builder pattern for dependency injection

**Key Principle:**
- Centralizes all dependency instantiation
- Makes dependency configuration explicit
- Easy to swap implementations (e.g., production vs. test)
- Uses the Builder pattern for fluent API

```rust
let logger = Arc::new(MockLogger::new());
let event_store = EventStore::new();
let event_bus = EventBus::new();
let repository = Arc::new(Repository::new(event_store));
let command_handler = UserCommandHandler::new(repository, event_bus, logger);
```

### 6. **Application Entry Point** (`src/main.rs`)

Kept intentionally thin. Only responsible for:
1. Initializing the composition root
2. Running the application

**Key Principle:**
- Delegates all logic to other layers
- Uses builder to wire dependencies
- No business logic here

## 🔄 Data Flow (CQRS + Event Sourcing)

**Write Side (Commands):**
```
Command → CommandHandler → Aggregate → Event → Repository → EventStore → EventBus
```

**Read Side (Queries):**
```
EventBus → Projections → Query → Response
```

**Dependency Direction:**
All dependencies point toward the domain layer. Layers depend on abstractions (traits), not concrete implementations.

## 🧪 Testing Strategy

The architecture enables different testing approaches:

### Unit Testing
```rust
// Test the domain layer in isolation
let mock_logger = Arc::new(MockLogger::new());
let repository = UserRepository::new(mock_logger.clone(), ...);
// No concrete implementations needed in domain
```

### Integration Testing
```rust
// Compose with test implementations
let app = AppBuilder::new()
    .with_logger(Arc::new(MockLogger::new()))
    .build_user_service();

app.register_user(1, "Alice");
// Verify behavior end-to-end with test doubles
```

## 🚀 How to Extend

### Adding a New Command

Example: Add "DeactivateUserCommand"

**Step 1:** Create the command
```rust
// src/commands/deactivate_user_command.rs
pub struct DeactivateUserCommand {
    pub user_id: u32,
    pub reason: String,
}

impl DeactivateUserCommand {
    pub fn new(user_id: u32, reason: String) -> Result<Self, String> {
        if user_id == 0 { return Err("Invalid ID".into()); }
        Ok(DeactivateUserCommand { user_id, reason })
    }
}
```

**Step 2:** Add event variant
```rust
// src/events/user_events.rs
pub enum UserEvent {
    Registered { user_id: u32, name: String, timestamp: i64 },
    Renamed { user_id: u32, new_name: String, timestamp: i64 },
    Deactivated { user_id: u32, reason: String, timestamp: i64 },  // NEW
}
```

**Step 3:** Add aggregate method
```rust
// src/domain/user_aggregate.rs
impl User {
    pub fn deactivate(&mut self, reason: String) {
        let event = UserEvent::Deactivated {
            user_id: self.id,
            reason,
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        self.apply_event(&event);
        self.uncommitted_changes.push(event);
    }
}
```

**Step 4:** Add handler
```rust
// src/commands/command_handler.rs
impl UserCommandHandler {
    pub fn handle_deactivate_user(&self, cmd: DeactivateUserCommand) -> Result<(), String> {
        let mut user = self.repository.get_by_id(cmd.user_id)?;
        user.deactivate(cmd.reason);
        self.repository.save(&user, user.version)?;
        Ok(())
    }
}
```

**Step 5:** Update projection
```rust
// src/events/projections.rs
impl TypedUserProjectionHandler {
    fn handle_user_deactivated(&self, user_id: u32) {
        // Update or remove from projection
    }
}
```

The new command flows through the entire CQRS pipeline automatically!

## 🎯 CQRS + Event Sourcing Pattern

This architecture implements **CQRS** (Command Query Responsibility Segregation) combined with **Event Sourcing**:

### Write Side (Commands)
- `commands/register_user_command.rs` - Command definitions
- `commands/command_handler.rs` - Command processors
- Commands are validated and can fail
- Successful commands emit domain events
- Events are stored in the **Event Store** (immutable, append-only log)

### Read Side (Queries)
- `queries/user_queries.rs` - Query handlers
- Queries read from **Projections** (read models)
- Projections are eventually consistent with events
- Multiple projections can exist for different use cases
- Queries never modify state

### Event Sourcing
- `events/event_store.rs` - Immutable event log (source of truth)
- `events/event_bus.rs` - Pub/sub for event distribution
- `events/projections.rs` - Read models built from events

**Flow:**
```
Command → Validation → Event → Event Store → Event Bus → Projections → Queries
```

### Eventual Consistency
- Commands write to the Event Store immediately
- Projections (read models) update asynchronously
- The read model **eventually** becomes consistent with the write model
- Enables horizontal scaling - reads and writes scale independently

## 🔑 Key Principles

### 1. **Dependency Inversion**
- High-level modules (services) don't depend on low-level modules (implementations)
- Both depend on abstractions (traits)

### 2. **Command-Query Separation**
- Commands modify state (write model)
- Queries retrieve data (read models)
- Different optimization strategies for each

### 3. **Event Sourcing**
- Events are facts - they can't be false
- Complete audit trail of all changes
- Can reconstruct state at any point in time
- Enables temporal queries and debugging

### 4. **Eventual Consistency**
- Projections lag behind the event store
- Acceptable latency for most use cases
- Scales better than immediate consistency
- Real-world systems embrace eventual consistency

### 5. **Single Responsibility**
- Each module has one reason to change
- Infrastructure handles cross-cutting concerns
- Domain contains business logic
- Commands handle write side
- Queries handle read side
- Events represent what happened
- Composition wires dependencies

### 6. **Testability**
- Mock implementations allow testing without external dependencies
- Layers can be tested in isolation
- Commands produce predictable events
- Projections can be verified against event sequences

### 7. **Flexibility**
- Swap implementations at composition time
- Same code works with different configurations
- Easy to support multiple environments (dev, test, prod)

## 🎯 Why This Architecture?

| Aspect | Benefit |
|--------|---------|
| **Separation of Concerns** | Each layer has a single, clear responsibility |
| **Testability** | Mock implementations enable comprehensive testing |
| **Maintainability** | Code is organized, discoverable, and easy to modify |
| **Flexibility** | Implementations can be swapped without code changes |
| **Scalability** | Easy to add new features and services |
| **Reusability** | `lib.rs` makes this usable as a dependency in other projects |
| **Explicit Dependencies** | No magic - all dependencies are visible and intentional |

## 🏃 Running the Application

```bash
# Run the application
cargo run

# Run tests
cargo test

# Build release version
cargo build --release
```

## 📖 DI/IoC Patterns Used

1. **Constructor Injection** - Dependencies passed via constructors
2. **Builder Pattern** - Fluent API for complex composition
3. **Composition Root** - Centralized wiring in `AppBuilder`
4. **Trait Objects** - Dynamic dispatch via `Arc<dyn Trait>`
5. **Service Locator** - AppBuilder acts as a service locator

## 🔗 References

- [Dependency Injection in Rust](https://docs.rs/shaku/latest/shaku/)
- [Domain-Driven Design](https://en.wikipedia.org/wiki/Domain-driven_design)
- [The Dependency Inversion Principle](https://en.wikipedia.org/wiki/Dependency_inversion_principle)
- [Clean Architecture in Rust]()

## 💡 Best Practices

1. **Keep main.rs thin** - It's just an entry point
2. **Domain layer is pure** - No external dependencies, just business logic
3. **Infrastructure at the bottom** - Contains cross-cutting concerns
4. **Composition in one place** - All wiring in AppBuilder
5. **Traits over structs** - Depend on abstractions, not implementations
6. **Use Arc for sharing** - Thread-safe shared ownership of singletons
7. **Test with mocks** - Replace implementations in tests

