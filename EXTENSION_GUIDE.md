# Extension Guide: Adding Features to the Application

This guide demonstrates how to add new features to the application following the layered crate architecture and CQRS/Event Sourcing patterns.

## 📋 Quick Reference

- **New Aggregate Type**: Add to `domain/aggregates/`
- **New Command**: Add to `domain/commands/`, implement in `application/handlers/`
- **New Event**: Add variant to event enum in `domain/events/`
- **New REST Endpoint**: Add handler in `api-rest/handlers/`, route in `main.rs`
- **New Projection**: Add to `persistence/projections/`, subscribe in event bus

## 🎯 Example: Add a New Aggregate (Product)

### Step 1: Define the Product Aggregate in Domain

Create `crates/domain/src/aggregates/product.rs`:

```rust
use serde::{Deserialize, Serialize};
use crate::errors::{AppError, DomainResult};
use crate::IRepository;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub version: i32,
    pub events: Vec<ProductEvent>,
}

impl Product {
    /// Create a new product with validation
    pub fn new(id: u32, name: String, price: f64) -> DomainResult<Self> {
        // Validate product name
        if name.is_empty() || name.len() > 200 {
            return Err(AppError::Validation(
                "Product name must be 1-200 characters".to_string()
            ));
        }

        // Validate price
        if price <= 0.0 {
            return Err(AppError::Validation(
                "Product price must be positive".to_string()
            ));
        }

        let event = ProductEvent::Created {
            id,
            name: name.clone(),
            price,
        };

        let mut product = Product {
            id,
            name,
            price,
            version: 0,
            events: vec![],
        };

        product.apply_event(&event);
        product.events.push(event);

        Ok(product)
    }

    /// Apply event to aggregate state
    fn apply_event(&mut self, event: &ProductEvent) {
        match event {
            ProductEvent::Created { id, name, price } => {
                self.id = *id;
                self.name = name.clone();
                self.price = *price;
                self.version = 0;
            }
            ProductEvent::PriceChanged { new_price } => {
                self.price = *new_price;
                self.version += 1;
            }
        }
    }

    /// Update product price with validation
    pub fn update_price(&mut self, new_price: f64) -> DomainResult<()> {
        if new_price <= 0.0 {
            return Err(AppError::Validation(
                "Price must be positive".to_string()
            ));
        }

        let event = ProductEvent::PriceChanged {
            new_price,
        };

        self.apply_event(&event);
        self.events.push(event);

        Ok(())
    }

    /// Reconstruct product from event history
    pub fn load_from_history(mut events: Vec<ProductEvent>) -> DomainResult<Self> {
        if events.is_empty() {
            return Err(AppError::AggregateNotFound(0));
        }

        let mut product = Product {
            id: 0,
            name: String::new(),
            price: 0.0,
            version: -1,
            events: vec![],
        };

        for event in events {
            product.apply_event(&event);
            product.version += 1;
        }

        Ok(product)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProductEvent {
    Created { id: u32, name: String, price: f64 },
    PriceChanged { new_price: f64 },
}
```

### Step 2: Update Domain Module Exports

Edit `crates/domain/src/aggregates/mod.rs`:

```rust
pub mod user;
pub mod product;  // Add this line

pub use user::User;
pub use product::{Product, ProductEvent};
```

### Step 3: Add Commands in Domain

Create `crates/domain/src/commands/product_commands.rs`:

```rust
use serde::{Deserialize, Serialize};
use crate::errors::DomainResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateProductCommand {
    pub product_id: u32,
    pub name: String,
    pub price: f64,
}

impl CreateProductCommand {
    pub fn new(product_id: u32, name: String, price: f64) -> DomainResult<Self> {
        Ok(CreateProductCommand {
            product_id,
            name,
            price,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateProductPriceCommand {
    pub product_id: u32,
    pub new_price: f64,
}
```

Edit `crates/domain/src/commands/mod.rs`:

```rust
pub mod register_user;
pub mod rename_user;
pub mod product_commands;  // Add this line

pub use register_user::RegisterUserCommand;
pub use rename_user::RenameUserCommand;
pub use product_commands::{CreateProductCommand, UpdateProductPriceCommand};
```

### Step 4: Update Domain lib.rs

Edit `crates/domain/src/lib.rs`:

```rust
pub mod aggregates;
pub mod commands;
pub mod events;
pub mod errors;
pub mod repository;

pub use aggregates::{User, Product, UserEvent, ProductEvent};
pub use commands::{RegisterUserCommand, RenameUserCommand, CreateProductCommand, UpdateProductPriceCommand};
pub use errors::{AppError, DomainResult};
pub use events::EventEnvelope;
pub use repository::IRepository;
```

### Step 5: Implement Command Handler in Application

Edit `crates/application/src/handlers/mod.rs`:

```rust
use domain::Product;

impl UserCommandHandler {
    // ... existing user handlers ...

    pub async fn handle_create_product(&self, command: CreateProductCommand) -> DomainResult<()> {
        let correlation_id = generate_correlation_id();
        
        self.logger.info(&format!(
            "Processing command: CreateProduct(id={}, name={}) [corr_id={}]",
            command.product_id, command.name, correlation_id
        ));

        let product = Product::new(
            command.product_id,
            command.name.clone(),
            command.price,
        )?;

        // Save to repository - NOTE: You'd need to extend Repository to handle Product
        // For now, this is pseudocode showing the pattern
        
        let saved_events = product.events.clone();
        
        for event in saved_events.iter() {
            match self.event_bus.publish(event).await {
                Ok(errors) if errors.is_empty() => {},
                Ok(errors) => {
                    for err in errors {
                        self.logger.warn(&format!("Non-critical handler error: {}", err));
                    }
                }
                Err(e) => {
                    self.logger.error(&format!("Critical error publishing event: {}", e));
                    return Err(AppError::PublishError(format!("Failed to publish event: {}", e)));
                }
            }
        }

        self.logger.info(&format!("Product {} created successfully", command.product_id));
        Ok(())
    }
}
```

### Step 6: Add REST Endpoint

Edit `crates/api-rest/src/handlers.rs`:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub product_id: u32,
    pub name: String,
    pub price: f64,
}

pub async fn create_product(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> impl IntoResponse {
    state.logger.info(&format!(
        "REST API: POST /products - create product {}",
        payload.product_id
    ));

    let command = CreateProductCommand {
        product_id: payload.product_id,
        name: payload.name.clone(),
        price: payload.price,
    };

    match state.command_handler.handle_create_product(command).await {
        Ok(_) => {
            state.logger.info(&format!(
                "REST API: Product {} created successfully",
                payload.product_id
            ));
            (
                StatusCode::CREATED,
                Json(SuccessResponse {
                    message: format!("Product {} created successfully", payload.product_id),
                }),
            )
                .into_response()
        }
        Err(err) => {
            let error_msg = format!("{:?}", err);
            state.logger.error(&format!("REST API: Failed to create product: {}", error_msg));
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse { error: error_msg }),
            )
                .into_response()
        }
    }
}
```

Edit `crates/api-rest/src/main.rs`:

```rust
use api_rest::{handlers::{register_user, rename_user, create_product}, AppState};

let app = Router::new()
    .route("/users", post(register_user))
    .route("/users", put(rename_user))
    .route("/products", post(create_product))  // Add this line
    .layer(CorsLayer::permissive())
    .with_state(state);
```

### Step 7: Add Projection (Optional)

Create `crates/persistence/src/projections/product_projection.rs`:

```rust
use domain::ProductEvent;
use crate::projections::Handles;

pub struct ProductProjection {
    products: std::collections::HashMap<u32, ProductData>,
}

#[derive(Clone)]
pub struct ProductData {
    pub id: u32,
    pub name: String,
    pub price: f64,
}

impl ProductProjection {
    pub fn new() -> Self {
        ProductProjection {
            products: std::collections::HashMap::new(),
        }
    }

    pub fn get_product(&self, id: u32) -> Option<ProductData> {
        self.products.get(&id).cloned()
    }
}

impl Handles<ProductEvent> for ProductProjection {
    fn handle(&mut self, event: &ProductEvent) {
        match event {
            ProductEvent::Created { id, name, price } => {
                self.products.insert(*id, ProductData {
                    id: *id,
                    name: name.clone(),
                    price: *price,
                });
            }
            ProductEvent::PriceChanged { new_price } => {
                // Update price for the last product (simplified example)
                // In real code, you'd track which product this event applies to
            }
        }
    }
}
```

## 🔄 Workflow Template for New Features

When adding a new feature, follow this checklist:

```
1. ✅ Domain Layer (crates/domain/)
   [ ] Define aggregate struct with fields and business logic
   [ ] Create event enum for all state changes
   [ ] Implement validation in constructor (new_with_*)
   [ ] Implement event application (apply_event)
   [ ] Implement reconstruction from history
   [ ] Define command structs with validation

2. ✅ Persistence Layer (crates/persistence/)
   [ ] Extend Repository to handle new aggregate type
   [ ] Add EventStore support for new event types
   [ ] Create optional Projection for read model

3. ✅ Application Layer (crates/application/)
   [ ] Add command handler methods
   [ ] Publish events to event bus
   [ ] Include correlation ID for tracing
   [ ] Add proper logging at each step

4. ✅ REST API Layer (crates/api-rest/)
   [ ] Define request DTOs
   [ ] Create handler functions
   [ ] Add routes to router
   [ ] Map errors to HTTP status codes

5. ✅ Testing
   [ ] Add unit tests to domain crate
   [ ] Add integration tests for command handling
   [ ] Test REST endpoints with curl or integration tests
   [ ] Verify error cases

6. ✅ Documentation
   [ ] Update ARCHITECTURE.md if needed
   [ ] Update README.md if user-facing
   [ ] Add code comments for complex business logic
```

## 📚 Pattern Examples

### Adding a Query Handler

In `crates/application/`, create a query handler:

```rust
pub struct GetProductQuery;

impl GetProductQuery {
    pub async fn execute(&self, product_id: u32, projection: &ProductProjection) -> Option<ProductData> {
        projection.get_product(product_id)
    }
}
```

### Adding Multi-Value Validation

```rust
pub fn validate_product(name: &str, price: f64) -> Result<(), Vec<AppError>> {
    let mut errors = vec![];
    
    if name.is_empty() {
        errors.push(AppError::Validation("Name cannot be empty".to_string()));
    }
    
    if price <= 0.0 {
        errors.push(AppError::Validation("Price must be positive".to_string()));
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

### Adding an Event Handler with Priority

```rust
pub struct NotificationHandler;

#[async_trait]
impl EventHandler for NotificationHandler {
    async fn handle(&self, event: &UserEvent) -> Result<(), String> {
        match event {
            UserEvent::Registered { name, .. } => {
                // Send welcome email
                println!("Sending welcome email to {}", name);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn priority(&self) -> HandlerPriority {
        HandlerPriority::High  // Execute before normal handlers
    }
}
```

## ⚠️ Common Pitfalls

| Issue | Solution |
|-------|----------|
| Circular dependencies between crates | Check dependency flow is strictly downward |
| Domain logic in REST layer | Keep DTOs separate, map to domain objects in handlers |
| Mutable state without versioning | Always include version in repository.save() |
| Missing error handling | Use `DomainResult<T>` consistently |
| Events not serializable | Ensure all event types derive `Serialize, Deserialize` |
| Projection out of sync | Subscribe projection to all relevant events |

## 🧪 Testing New Features

### Unit Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_creation_with_valid_input() {
        let product = Product::new(1, "Widget".to_string(), 29.99);
        assert!(product.is_ok());
        let p = product.unwrap();
        assert_eq!(p.name, "Widget");
        assert_eq!(p.price, 29.99);
    }

    #[test]
    fn test_product_creation_with_negative_price() {
        let product = Product::new(1, "Widget".to_string(), -10.0);
        assert!(product.is_err());
    }

    #[test]
    fn test_product_reconstruction_from_history() {
        let events = vec![
            ProductEvent::Created {
                id: 1,
                name: "Widget".to_string(),
                price: 29.99,
            }
        ];
        let product = Product::load_from_history(events);
        assert!(product.is_ok());
    }
}
```

### Integration Test Template

```rust
#[tokio::test]
async fn test_create_product_command_flow() {
    // Setup
    let logger = Arc::new(MockLogger::new());
    let event_store = Arc::new(EventStore::new());
    let handler = UserCommandHandler::new(
        Arc::new(Repository::new((*event_store).clone())),
        EventBus::new(),
        logger.clone(),
    );

    // Execute
    let command = CreateProductCommand::new(1, "Widget".to_string(), 29.99);
    let result = handler.handle_create_product(command).await;

    // Verify
    assert!(result.is_ok());
    // Add assertions about event store state
}
```

## 🎓 Learning Resources

1. **Review existing implementations**:
   - `User` aggregate in `crates/domain/src/aggregates/`
   - `UserCommandHandler` in `crates/application/src/handlers/`
   - REST handlers in `crates/api-rest/src/handlers.rs`

2. **Understand the patterns**:
   - Read ARCHITECTURE.md for design overview
   - Study CQRS and Event Sourcing references
   - Follow domain-driven design principles

3. **Test your changes**:
   - `cargo build` - Verify compilation
   - `cargo test` - Run all tests
   - `cargo test --lib` - Run unit tests only

4. **Ask questions**:
   - Check if similar features already exist
   - Review error handling in similar features
   - Look at test examples for patterns

---

**Tips for Success**:
- Start with the domain layer (pure business logic)
- Work downward through layers (infrastructure → persistence → application → REST)
- Keep aggregates small and focused
- Make events immutable and serializable
- Test at each layer independently
- Use type system to enforce constraints

