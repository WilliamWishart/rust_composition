# Enterprise-Scale Rust Application - DDD/CQRS/Event Sourcing

> **📚 Learning Example** - This is an educational project designed to teach and demonstrate **Dependency Injection (DI)**, **Domain-Driven Design (DDD)**, **CQRS** (Command Query Responsibility Segregation), and **Event Sourcing** patterns in Rust. It serves as a reference for how to structure large-scale, enterprise applications.

A comprehensive example demonstrating best practices for organizing large Rust applications using **Domain-Driven Design** combined with **CQRS** and **Event Sourcing** to achieve eventual consistency and scalability.

## 📁 Project Structure

```
rust_composition/
├── Cargo.toml                          # Project manifest
├── src/
│   ├── lib.rs                          # Library root - exports all modules
│   ├── main.rs                         # Application entry point (thin wrapper)
│   ├── infrastructure/                 # External dependencies & adapters
│   │   ├── mod.rs                      # Module definition & re-exports
│   │   ├── logger.rs                   # Logger trait & implementations
│   │   └── database.rs                 # Database trait & implementations
│   ├── domain/                         # Business logic (core domain)
│   │   ├── mod.rs                      # Module definition & re-exports
│   │   ├── user_repository.rs          # Data access abstraction
│   │   └── user_aggregate.rs           # Domain models (User aggregate)
│   ├── events/                         # Event Sourcing (immutable event log)
│   │   ├── mod.rs                      # Module definition & re-exports
│   │   ├── domain_events.rs            # Domain event definitions
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
- `database.rs` - Database trait with `MockDatabase` implementation

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
// Domain knows about Logger trait, not ConsoleLogger
pub struct UserRepository {
    logger: Arc<dyn Logger>,  // Injected abstraction
    db: Arc<dyn Database>,
}
```

### 3. **Application Layer** (`src/application/`)

Orchestrates the domain logic and implements use cases.

**Files:**
- `user_service.rs` - Application service with use case methods

**Key Principle:**
- Coordinates between domain and infrastructure
- Implements specific business use cases
- Thin logic layer - delegates to domain
- Depends on domain and infrastructure traits

```rust
pub struct UserService {
    repository: Arc<UserRepository>,
    logger: Arc<dyn Logger>,
}

impl UserService {
    pub fn register_user(&self, id: u32, name: &str) {
        // Orchestrates domain and infrastructure
    }
}
```

### 4. **Composition Root Layer** (`src/composition/`)

The **only place that knows about concrete implementations**. Responsible for wiring all dependencies.

**Files:**
- `app_builder.rs` - Builder pattern for dependency injection

**Key Principle:**
- Centralizes all dependency instantiation
- Makes dependency configuration explicit
- Easy to swap implementations (e.g., production vs. test)
- Uses the Builder pattern for fluent API

```rust
let app = AppBuilder::new()
    .with_logger(Arc::new(ConsoleLogger))
    .with_database(Arc::new(MockDatabase))
    .build_user_service();
```

### 5. **Application Entry Point** (`src/main.rs`)

Kept intentionally thin. Only responsible for:
1. Initializing the composition root
2. Running the application

**Key Principle:**
- Delegates all logic to other layers
- Uses builder to wire dependencies
- No business logic here

## 🔄 Dependency Flow

Dependencies flow **inward**, never outward:

```
main.rs
   ↓
AppBuilder (Composition Root)
   ↓
UserService (Application)
   ↓
UserRepository (Domain) + Logger/Database (Infrastructure)
```

Each layer depends only on layers below it through **trait abstractions**, never on concrete implementations above it.

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

### Adding a New Feature

Example: Add email notification capability

**Step 1:** Define abstraction in Infrastructure
```rust
// src/infrastructure/email.rs
pub trait EmailService: Send + Sync {
    fn send(&self, email: &str, message: &str);
}

pub struct MockEmailService;
impl EmailService for MockEmailService { ... }
```

**Step 2:** Update Infrastructure exports
```rust
// src/infrastructure/mod.rs
pub mod email;
pub use email::{EmailService, MockEmailService};
```

**Step 3:** Inject into Domain/Application
```rust
// src/application/user_service.rs
pub struct UserService {
    repository: Arc<UserRepository>,
    logger: Arc<dyn Logger>,
    email_service: Arc<dyn EmailService>,  // NEW
}

impl UserService {
    pub fn register_user(&self, id: u32, name: &str, email: &str) {
        self.repository.save_user(id, name);
        self.email_service.send(email, "Welcome!");
    }
}
```

**Step 4:** Update Composition Root
```rust
// src/composition/app_builder.rs
pub struct AppBuilder {
    logger: Arc<dyn Logger>,
    database: Arc<dyn Database>,
    email_service: Arc<dyn EmailService>,  // NEW
}

impl AppBuilder {
    pub fn with_email_service(mut self, svc: Arc<dyn EmailService>) -> Self {
        self.email_service = svc;
        self
    }
    
    pub fn build_user_service(self) -> UserService {
        let repository = Arc::new(UserRepository::new(...));
        UserService::new(repository, self.logger, self.email_service)
    }
}
```

**Step 5:** Use in main.rs
```rust
let app = AppBuilder::new()
    .with_email_service(Arc::new(MockEmailService))
    .build_user_service();
```

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
- `events/domain_events.rs` - Domain event definitions
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

