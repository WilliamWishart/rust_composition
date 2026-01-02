# Enterprise-Scale Rust Application - DDD/CQRS/Event Sourcing

> **📚 Learning Example** - This is a professional reference architecture demonstrating **Domain-Driven Design (DDD)**, **CQRS** (Command Query Responsibility Segregation), and **Event Sourcing** patterns in a layered Rust codebase. Ideal for learning enterprise application architecture.

## 🚀 Quick Start

```bash
# Build all crates
cargo build --all

# Run tests
cargo test --all

# Start REST server
cargo run -p api-rest
# Server runs on http://127.0.0.1:3000
```

### Test the API
```bash
# Register a user
curl -X POST http://127.0.0.1:3000/users \
  -H "Content-Type: application/json" \
  -d '{"user_id": 1, "name": "Alice"}'

# Rename user
curl -X PUT http://127.0.0.1:3000/users \
  -H "Content-Type: application/json" \
  -d '{"user_id": 1, "new_name": "Alice Smith"}'
```

## 📚 Documentation

This project includes comprehensive documentation:

| Document | Purpose | Audience |
|----------|---------|----------|
| **[ARCHITECTURE.md](ARCHITECTURE.md)** | Complete design guide with patterns, layers, and examples | Everyone (start here) |
| **[EXTENSION_GUIDE.md](EXTENSION_GUIDE.md)** | How to add new features with step-by-step examples | Developers |
| **[ROADMAP.md](ROADMAP.md)** | Planned improvements (15 weeks to v3.0) | Technical leads |
| **[REFACTORING_COMPLETE.md](REFACTORING_COMPLETE.md)** | Completion summary of layered crate migration | Project managers |

## 🏗️ Architecture Overview

**Layered Crate Architecture** with acyclic dependencies:

```
┌────────────────────────────────────────────┐
│            CQRS PATTERN                    │
```
crates/
├── domain/              (Pure business logic: User aggregate, events, commands)
├── infrastructure/      (Logger, metrics, cross-cutting concerns)
├── persistence/         (EventStore, Repository, projections)
├── application/         (Command handlers, event bus orchestration)
└── api-rest/            (HTTP API with Axum, running on :3000)
```

**Dependency Flow (Acyclic)**:
```
Domain ← Infrastructure ← Persistence ← Application ← API-REST
```

## ✨ Key Features

- **DDD**: User aggregate with business rule validation
- **CQRS**: Separate command and query paths
- **Event Sourcing**: Immutable event log with complete audit trail
- **Async/Await**: Modern async handlers and EventBus
- **Type Safety**: Unified AppError enum and DomainResult<T>
- **Testing**: 31 tests passing (2 unit + 29 integration)
- **REST API**: Full HTTP endpoints with error handling
- **Logging**: Abstracted Logger trait with multiple implementations

## 📖 Documentation Structure

1. **ARCHITECTURE.md** - Start here
   - Complete design overview
   - All patterns explained with diagrams
   - Data flow examples
   - Dependency rules

2. **EXTENSION_GUIDE.md** - Adding features
   - Step-by-step walkthrough (Product aggregate example)
   - Common patterns and templates
   - Testing approaches

3. **ROADMAP.md** - Future improvements
   - 7 phases of planned enhancements
   - 15-week timeline to v3.0
   - Success criteria for each phase

4. **REFACTORING_COMPLETE.md** - Project summary
   - Completion report
   - Metrics and deliverables
   - Before/after comparison

## 💡 Example: Register User

```bash
# Call the API
curl -X POST http://127.0.0.1:3000/users \
  -H "Content-Type: application/json" \
  -d '{"user_id": 1, "name": "Alice"}'

# Response
{"message": "User 1 registered successfully"}
```

**What happens**:
1. REST handler receives RegisterUserRequest
2. Creates RegisterUserCommand with validation
3. UserCommandHandler.handle_register_user() executes:
   - User::new_with_uniqueness_check() validates name
   - User applies Registered event internally
   - repository.save() appends event to EventStore
4. EventBus publishes event to subscribers
5. UserProjection updates read model (eventually consistent)
6. Returns 201 Created

## 🧪 Testing

```bash
# Run all tests
cargo test --all

# Run specific test
cargo test --lib event_envelope

# Run with output
cargo test -- --nocapture
```

## 🏗️ Code Organization

| File | Purpose |
|------|---------|
| `crates/domain/src/errors.rs` | AppError enum (8 variants) |
| `crates/domain/src/events/mod.rs` | UserEvent + EventEnvelope |
| `crates/domain/src/aggregates/mod.rs` | User aggregate with validation |
| `crates/domain/src/repository.rs` | IRepository trait |
| `crates/domain/src/commands/mod.rs` | RegisterUserCommand, RenameUserCommand |
| `crates/infrastructure/src/logger.rs` | Logger trait + ConsoleLogger, MockLogger |
| `crates/persistence/src/event_store.rs` | Append-only event log + DLQ |
| `crates/persistence/src/user_repository.rs` | Repository implementation |
| `crates/persistence/src/projections/mod.rs` | UserProjection (read model) |
| `crates/application/src/handlers/mod.rs` | UserCommandHandler |
| `crates/application/src/event_bus.rs` | EventBus with async handlers |
| `crates/api-rest/src/handlers.rs` | REST endpoints |
| `crates/api-rest/src/main.rs` | Server setup and routing |

## 🎯 Design Patterns

| Pattern | Used In | Purpose |
|---------|---------|---------|
| **DDD** | domain/ | Model complex business logic |
| **CQRS** | application/ | Separate reads from writes |
| **Event Sourcing** | persistence/ | Complete audit trail |
| **Repository** | domain/ | Abstract data access |
| **Dependency Injection** | api-rest/ | Loose coupling |
| **Adapter** | infrastructure/ | Abstract cross-cutting concerns |
| **Layered Architecture** | All crates | Separation of concerns |

## 🚀 Next Steps

1. **Review** [ARCHITECTURE.md](ARCHITECTURE.md) for design details
2. **Learn** [EXTENSION_GUIDE.md](EXTENSION_GUIDE.md) for adding features
3. **Build** your own command/event/aggregate following the pattern
4. **Plan** using [ROADMAP.md](ROADMAP.md) for future work

## 📊 Project Stats

- **Crates**: 5 independent crates
- **Lines of Code**: 1,200+ (domain, infrastructure, persistence, application, api-rest)
- **Documentation**: 2,582 lines across 5 markdown files
- **Tests**: 2 unit + 29 integration (all passing ✅)
- **Build Time**: ~1 second
- **Compiler Warnings**: 0

## 🔗 References

- [Domain-Driven Design by Eric Evans](https://www.domainlanguage.com/ddd/)
- [CQRS by Greg Young](https://cqrs.nu/)
- [Event Sourcing](https://martinfowler.com/eaaDev/EventSourcing.html)
- [Rust Async Programming](https://rust-lang.github.io/async-book/)

---

**For detailed information, see [ARCHITECTURE.md](ARCHITECTURE.md)** ✨

