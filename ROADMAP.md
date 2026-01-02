# Roadmap: Future Improvements & Enhancements

This document outlines planned improvements to make the application more complete, production-ready, and easier to extend.

## 🎯 Current Status (v2.0)

**Completed**:
- ✅ Layered crate architecture (5 independent crates)
- ✅ DDD with aggregate roots (User)
- ✅ CQRS pattern (separate command/query paths)
- ✅ Event Sourcing (immutable event log)
- ✅ Optimistic locking (concurrency control)
- ✅ Event bus with async handlers
- ✅ REST API with Axum
- ✅ Comprehensive logging infrastructure
- ✅ Error handling with AppError enum
- ✅ Full test coverage (2 unit + 29 integration tests)

**Known Limitations**:
- ⚠️ Single-aggregate commands only (no sagas for multi-aggregate transactions)
- ⚠️ In-memory event store (no persistent storage)
- ⚠️ Single error per validation (should collect all errors)
- ⚠️ Event handler failure isn't persistent (no dead letter storage)
- ⚠️ No event versioning/migration framework
- ⚠️ No snapshots for large aggregates
- ⚠️ No distributed tracing beyond correlation IDs
- ⚠️ Time not injectable (clock trait missing)

## 📋 Roadmap by Priority

### Phase 1: Validation Improvements (Weeks 1-2)

#### 1.1 Multi-Error Validation
**Issue**: Only first validation error is returned. Should collect all errors.

**Example**:
```rust
// Current: Returns first error only
Product::new("", -10.0)  // Returns: "Name cannot be empty"

// Desired: Returns all errors
Product::new("", -10.0)  // Returns: ["Name cannot be empty", "Price must be positive"]
```

**Implementation**:
- Create `ValidationError { errors: Vec<String> }` variant in AppError
- Update all `new_with_*()` methods to collect errors instead of early-return
- Update error handling in REST layer to display all errors
- Estimated effort: 2-3 hours
- Impact: Better UX, easier to fix input validation issues

**Files to change**:
- `crates/domain/src/errors.rs` - Add ValidationError variant
- `crates/domain/src/aggregates/mod.rs` - Collect validation errors
- `crates/api-rest/src/handlers.rs` - Display all errors in response

#### 1.2 Extract Constraint Magic Strings
**Issue**: Validation limits hardcoded as magic numbers in code.

**Example**:
```rust
// Current
if name.len() > 100 { /* error */ }

// Desired
const MAX_USER_NAME_LENGTH: usize = 100;
pub struct UserConstraints {
    const MAX_NAME_LENGTH: usize = 100;
    const MIN_NAME_LENGTH: usize = 1;
}
```

**Implementation**:
- Create `crates/domain/src/constraints.rs`
- Define `UserConstraints` struct with const values
- Update all validation code to reference constants
- Update tests to use constants
- Estimated effort: 1 hour
- Impact: Single source of truth for business rules

### Phase 2: Testability Improvements (Weeks 2-3)

#### 2.1 Clock Trait for Testable Timestamps
**Issue**: Can't test time-dependent behavior (expiry, scheduled events).

**Current**: `chrono::Utc::now()` is called directly (not injectable).

**Desired Design**:
```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;  // Production: uses real time
pub struct MockClock {   // Testing: returns fixed time
    current_time: RefCell<DateTime<Utc>>,
}
```

**Implementation**:
- Add `Clock` trait to `infrastructure` crate
- Add `SystemClock` and `MockClock` implementations
- Update `EventEnvelope` to accept `clock` parameter
- Pass clock through all layers
- Update tests to use `MockClock`
- Estimated effort: 3-4 hours
- Impact: Full control over time in tests, testable scheduled logic

**Files to change**:
- `crates/infrastructure/src/lib.rs` - Add Clock trait
- `crates/domain/src/events/mod.rs` - Use Clock in EventEnvelope
- `crates/application/src/handlers/mod.rs` - Accept clock parameter

#### 2.2 Mock Repository Implementation
**Issue**: Testing command handlers requires EventStore setup.

**Desired**:
```rust
pub struct MockRepository {
    saved_events: RefCell<Vec<(u32, Vec<UserEvent>)>>,
    get_returns: RefCell<HashMap<u32, User>>,
}

impl IRepository for MockRepository {
    fn save(&self, user: &User, version: i32) -> DomainResult<Vec<UserEvent>> {
        self.saved_events.borrow_mut().push((user.id, user.events.clone()));
        Ok(user.events.clone())
    }
}
```

**Implementation**:
- Create `crates/persistence/src/testing.rs` with `MockRepository`
- Export in `crates/persistence/src/lib.rs`
- Update application tests to use `MockRepository`
- Estimated effort: 2 hours
- Impact: Faster unit tests, isolated command handler testing

### Phase 3: Event Handling Robustness (Weeks 3-4)

#### 3.1 Event Handler Dead Letter Queue
**Issue**: If an event handler fails, the error is logged but the event is lost (no retry).

**Desired**:
```rust
pub struct DeadLetterQueue {
    failed_events: Vec<(UserEvent, String)>,  // event + error reason
}

impl DeadLetterQueue {
    pub fn publish_failed_event(&mut self, event: UserEvent, error: String) {
        self.failed_events.push((event, error));
    }

    pub fn retry_all(&self) -> Result<(), String> {
        // Attempt to reprocess failed events
    }
}
```

**Implementation**:
- Add `DeadLetterQueue` to `EventBus` struct
- Catch errors in event handler subscriptions
- Store failed events with error details
- Provide `retry()` method for manual replay
- Add metrics for dead letter queue size
- Estimated effort: 3 hours
- Impact: No lost events, manual error recovery, better visibility

**Files to change**:
- `crates/application/src/event_bus.rs` - Add DeadLetterQueue
- `crates/application/src/handlers/mod.rs` - Push to DLQ on error

#### 3.2 Event Handler Persistence
**Issue**: Event handlers are in-memory. If server restarts, subscriptions are lost.

**Desired**:
```rust
pub struct PersistentEventBus {
    subscriptions: HashMap<String, Vec<Arc<dyn EventHandler>>>,
}

impl PersistentEventBus {
    pub async fn save_subscription(&self, handler_id: &str, handler: Arc<dyn EventHandler>) {
        // Save to persistent storage
    }

    pub async fn restore_subscriptions(&mut self) {
        // Load from persistent storage
    }
}
```

**Implementation**:
- Add `handler_id` field to `EventHandler` trait
- Create `subscription_store` module in persistence crate
- Save subscriptions to file/database on app startup
- Restore subscriptions from storage on startup
- Estimated effort: 4-5 hours
- Impact: Handlers survive server restarts

### Phase 4: Event Versioning & Migration (Weeks 4-6)

#### 4.1 Event Versioning
**Issue**: Can't add fields to existing event types without breaking event history.

**Desired**:
```rust
#[derive(Serialize, Deserialize)]
pub enum UserEvent {
    #[serde(rename = "UserRegisteredV1")]
    RegisteredV1 { id: u32, name: String },
    
    #[serde(rename = "UserRegisteredV2")]
    RegisteredV2 { id: u32, name: String, email: String },  // NEW: email field
}

impl UserEvent {
    pub fn upgrade_to_latest(&self) -> UserEvent {
        match self {
            UserEvent::RegisteredV1 { id, name } => {
                UserEvent::RegisteredV2 {
                    id: *id,
                    name: name.clone(),
                    email: "unknown@example.com".to_string(),  // Default for old events
                }
            }
            other => other.clone(),
        }
    }
}
```

**Implementation**:
- Add version suffix to all event variants
- Create migration framework in domain crate
- Implement `upgrade_to_latest()` for each event type
- Apply migrations when loading from EventStore
- Estimated effort: 5-6 hours
- Impact: Can evolve event schema over time

**Files to change**:
- `crates/domain/src/events/mod.rs` - Add versioning to events
- `crates/domain/src/lib.rs` - Export migration functions
- `crates/persistence/src/event_store.rs` - Apply migrations on load

#### 4.2 Aggregate Snapshots
**Issue**: Reconstructing large aggregates from hundreds of events is slow.

**Desired**:
```rust
pub struct Snapshot {
    pub aggregate_id: u32,
    pub version: i32,
    pub state: Vec<u8>,  // Serialized aggregate
    pub created_at: DateTime<Utc>,
}

impl EventStore {
    pub async fn load_aggregate(&self, id: u32) -> Result<User> {
        // Try to load snapshot first
        if let Ok(snapshot) = self.load_snapshot(id) {
            if let Ok(mut user) = bincode::deserialize(&snapshot.state) {
                // Load only events after snapshot
                let events = self.load_events_after(id, snapshot.version)?;
                for event in events {
                    user.apply_event(&event);
                }
                return Ok(user);
            }
        }

        // Fallback: reconstruct from all events
        self.load_and_reconstruct(id)
    }

    pub async fn create_snapshot(&self, user: &User) -> Result<()> {
        let state = bincode::serialize(user)?;
        // Save snapshot...
    }
}
```

**Implementation**:
- Add `Snapshot` struct to persistence crate
- Add snapshot creation to repository on periodic basis
- Modify `load_aggregate` to check snapshots first
- Track snapshot version metadata
- Estimated effort: 5 hours
- Impact: Fast loading of large aggregates

**Files to change**:
- `crates/persistence/src/event_store.rs` - Add snapshot support
- `crates/persistence/src/user_repository.rs` - Use snapshots

### Phase 5: Distributed Tracing & Observability (Weeks 6-8)

#### 5.1 OpenTelemetry Integration
**Issue**: Hard to debug issues in distributed systems (no trace context across services).

**Desired**:
```rust
use opentelemetry::trace::Tracer;
use tracing_opentelemetry::OpenTelemetryLayer;

pub struct TracedCommandHandler {
    inner: UserCommandHandler,
    tracer: Tracer,
}

impl TracedCommandHandler {
    pub async fn handle_register_user(&self, command: RegisterUserCommand) -> DomainResult<()> {
        let span = self.tracer.start("handle_register_user");
        let result = self.inner.handle_register_user(command).await;
        span.end();
        result
    }
}
```

**Implementation**:
- Add `opentelemetry` dependency to all crates
- Create instrumented versions of handlers
- Export correlation_id as trace context
- Use structured logging with tracing crate
- Estimated effort: 6-8 hours
- Impact: Full request tracing across services

**Files to change**:
- All `Cargo.toml` files - Add tracing dependencies
- `crates/application/src/handlers/mod.rs` - Add spans
- `crates/api-rest/src/main.rs` - Initialize tracing

#### 5.2 Metrics with Prometheus
**Issue**: No metrics exported for monitoring (latency, error rates, event bus depth).

**Desired**:
```rust
pub struct MetricsCollector {
    command_latency: prometheus::Histogram,
    command_errors: prometheus::IntCounter,
    event_bus_queue_depth: prometheus::Gauge,
}

impl MetricsCollector {
    pub fn record_command_latency(&self, name: &str, duration_ms: f64) {
        self.command_latency.observe(duration_ms);
    }

    pub fn record_event_bus_depth(&self, depth: usize) {
        self.event_bus_queue_depth.set(depth as i64);
    }
}

// Expose at GET /metrics
```

**Implementation**:
- Add prometheus dependency to infrastructure crate
- Create metrics endpoints in REST API
- Record metrics in command handlers
- Create Grafana dashboard (separate project)
- Estimated effort: 4-5 hours
- Impact: Production observability

### Phase 6: Advanced Patterns (Weeks 8-10)

#### 6.1 Saga Pattern for Multi-Aggregate Transactions
**Issue**: Can't coordinate changes across multiple aggregates atomically.

**Desired**:
```rust
pub trait Saga {
    async fn start(&self, command: &Command) -> Result<SagaId>;
    async fn handle_event(&self, event: &Event) -> Result<Option<Command>>;
    async fn compensate(&self) -> Result<()>;  // Undo on failure
}

// Example: Transfer funds (affects two accounts)
pub struct TransferSaga {
    account_repo: Arc<dyn IRepository<Account>>,
}

impl Saga for TransferSaga {
    async fn start(&self, command: &TransferCommand) -> Result<SagaId> {
        let saga = SagaInstance::new(generate_saga_id());
        saga.emit_step(DebitAccountCommand::new(command.from, command.amount));
        Ok(saga.id)
    }

    async fn handle_event(&self, event: &Event) -> Result<Option<Command>> {
        match event {
            AccountDebited { saga_id, account } => {
                // Emit credit command for other account
                Some(CreditAccountCommand::new(saga_id, ...))
            }
            AccountCredited { saga_id, .. } => {
                // Saga complete
                None
            }
        }
    }
}
```

**Implementation**:
- Create `crates/domain/src/sagas.rs` module
- Define `Saga` trait with lifecycle methods
- Implement `TransferSaga` as example
- Create saga state machine in persistence layer
- Estimated effort: 8-10 hours
- Impact: Support complex, distributed workflows

#### 6.2 Query DSL for Complex Projections
**Issue**: Complex queries require custom projection code.

**Desired**:
```rust
pub struct QueryBuilder {
    filters: Vec<Filter>,
    sort: Vec<Sort>,
    limit: Option<usize>,
}

impl QueryBuilder {
    pub fn new() -> Self { /* ... */ }

    pub fn filter(mut self, field: &str, op: FilterOp, value: &str) -> Self {
        self.filters.push(Filter { field, op, value });
        self
    }

    pub fn sort(mut self, field: &str, order: SortOrder) -> Self {
        self.sort.push(Sort { field, order });
        self
    }

    pub fn execute(&self, projection: &UserProjection) -> Result<Vec<User>> {
        // Apply filters, sorting, limits
    }
}

// Usage
let results = QueryBuilder::new()
    .filter("name", FilterOp::Contains, "Alice")
    .filter("created_at", FilterOp::GreaterThan, "2024-01-01")
    .sort("created_at", SortOrder::Desc)
    .limit(10)
    .execute(&projection)?;
```

**Implementation**:
- Create `crates/persistence/src/query_dsl.rs`
- Define `Filter`, `Sort`, `FilterOp` types
- Implement filter/sort logic for projection
- Add to REST API as query parameters
- Estimated effort: 4-5 hours
- Impact: Dynamic queries without custom projection code

### Phase 7: CLI & Data Management (Weeks 10-12)

#### 7.1 Command-Line Interface
**Issue**: No way to manage data without REST API (seed, export, migrate).

**Desired**:
```bash
cargo run --bin rust-cli -- seed --users 100  # Generate test data
cargo run --bin rust-cli -- export --format json --output users.json
cargo run --bin rust-cli -- migrate --from-version 1 --to-version 2
cargo run --bin rust-cli -- replay-events --from-version 1
```

**Implementation**:
- Create `crates/cli/` crate with clap
- Implement subcommands for common operations
- Add database seeding utilities
- Add event export/import functionality
- Estimated effort: 6-8 hours
- Impact: Operations and testing tools

#### 7.2 Event Store to SQL Database Sync
**Issue**: Events only in memory; lost on restart. Should persist to disk/database.

**Desired**:
```rust
pub struct SqlEventStore {
    pool: PgPool,
    in_memory_cache: Vec<StoredEvent>,
}

impl SqlEventStore {
    pub async fn append(&mut self, events: Vec<UserEvent>) -> Result<()> {
        // Save to database
        sqlx::query(
            "INSERT INTO events (aggregate_id, data, version) VALUES ($1, $2, $3)"
        ).execute(&self.pool).await?;

        // Update cache
        self.in_memory_cache.extend(events);
        Ok(())
    }

    pub async fn load(&mut self, id: u32) -> Result<Vec<UserEvent>> {
        sqlx::query_as::<_, UserEvent>(
            "SELECT data FROM events WHERE aggregate_id = $1 ORDER BY version"
        ).fetch_all(&self.pool).await
    }
}
```

**Implementation**:
- Add SQLx and PostgreSQL driver to dependencies
- Implement `SqlEventStore` in persistence crate
- Create database migrations (events table schema)
- Add configuration for database connection string
- Estimated effort: 8-10 hours
- Impact: Event sourcing survives restarts

## 📊 Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| 1: Validation | 2 weeks | Multi-error validation, constraint extraction |
| 2: Testability | 2 weeks | Clock trait, Mock repository |
| 3: Robustness | 2 weeks | Event handler DLQ, persistent handlers |
| 4: Event Evolution | 3 weeks | Event versioning, snapshots |
| 5: Observability | 2 weeks | OpenTelemetry, Prometheus metrics |
| 6: Advanced Patterns | 2 weeks | Saga pattern, Query DSL |
| 7: CLI & Persistence | 2 weeks | CLI tools, SQL database |
| **Total** | **~15 weeks** | Production-grade event sourcing system |

## 🎯 Success Criteria

### By End of Phase 1
- [ ] All validation errors collected in single response
- [ ] Business rule constraints centralized in single module
- [ ] No magic numbers in validation code

### By End of Phase 2
- [ ] Can mock time in tests (`MockClock`)
- [ ] Command handlers tested without EventStore setup
- [ ] 100% unit test coverage

### By End of Phase 3
- [ ] No lost events (dead letter queue)
- [ ] Event handler failures tracked and reported
- [ ] Manual retry mechanism for failed events

### By End of Phase 4
- [ ] Event schema can evolve without breaking history
- [ ] Large aggregates load in <100ms (snapshots)
- [ ] Tested with 10,000+ events per aggregate

### By End of Phase 5
- [ ] Request traces visible across service boundaries
- [ ] Metrics dashboards in Grafana
- [ ] Performance regression detection

### By End of Phase 6
- [ ] Multi-step workflows coordinated via sagas
- [ ] Complex queries without custom projection code
- [ ] Can handle 1000+ events/sec

### By End of Phase 7
- [ ] Event data persists to PostgreSQL
- [ ] CLI tools for operations (seed, export, migrate)
- [ ] Tested 1M+ events in event store

## 🚀 Getting Started with Phase 1

To start implementing, pick one improvement:

1. **Multi-Error Validation** (easiest, ~2 hours):
   - PR checklist in EXTENSION_GUIDE.md
   - Tests in `crates/domain/src/aggregates/user.rs`

2. **Clock Trait** (intermediate, ~4 hours):
   - Similar pattern to existing Logger trait
   - Start with `infrastructure/clock.rs`
   - Thread through all layers

3. **Event Handler DLQ** (intermediate, ~3 hours):
   - Extend EventBus struct in `application/event_bus.rs`
   - Add error tracking and retry logic
   - Test with intentionally failing handlers

## 📚 References

- **Event Sourcing Migration**: Spiegler, S. (2019). "Practical Event Sourcing"
- **Sagas**: Young, G. "A Saga on Sagas" (whitepaper)
- **OpenTelemetry**: https://opentelemetry.io/
- **Prometheus**: https://prometheus.io/docs/

---

**Document Status**: Active Planning  
**Last Updated**: January 2025  
**Priority**: Phases 1-2 (essential), Phases 3-4 (important), Phases 5-7 (nice-to-have)

