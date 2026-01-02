//! # Rust Composition - Enterprise Architecture
//!
//! Re-exports from the layered crate architecture for backward-compatible test access.

// Module re-exports for backward compatibility with existing tests
pub mod infrastructure {
    pub use ::infrastructure::*;
    pub use ::domain::DomainError;
}

pub mod domain {
    pub use ::domain::*;
    pub use ::persistence::Repository;
}

pub mod commands {
    pub use ::domain::commands::*;
    pub use ::application::handlers::UserCommandHandler;
}

pub mod events {
    pub use ::domain::events::*;
    pub use ::persistence::event_store::EventStore;
    pub use ::application::event_bus::EventBus;
    pub use ::application::EventHandler;
    
    pub mod event_bus {
        pub use ::application::event_bus::{EventBus, HandlerPriority, PublishError, HandlerError};
    }
    
    pub mod projections {
        use ::domain::events::UserEvent;
        use ::application::EventHandler;
        
        pub use ::persistence::projections::*;
        
        /// Adapter to make TypedUserProjectionHandler work with EventBus
        pub struct TypedUserProjectionHandlerAdapter {
            inner: ::persistence::projections::TypedUserProjectionHandler,
        }
        
        impl TypedUserProjectionHandlerAdapter {
            pub fn new(handler: ::persistence::projections::TypedUserProjectionHandler) -> Self {
                TypedUserProjectionHandlerAdapter { inner: handler }
            }
        }
        
        #[async_trait::async_trait]
        impl EventHandler for TypedUserProjectionHandlerAdapter {
            async fn handle_event(&self, event: &UserEvent) -> Result<(), Box<dyn std::error::Error>> {
                use ::persistence::projections::Handles;
                self.inner.handle(event);
                Ok(())
            }
            
            fn name(&self) -> &str {
                "UserProjectionAdapter"
            }
        }
    }
}

pub mod queries {
    use ::persistence::{UserProjection, projections::UserReadModel};
    
    /// UserQuery - Query interface for the read model
    /// Wraps UserProjection to provide a simplified query API for tests
    pub struct UserQuery {
        projection: UserProjection,
    }
    
    impl UserQuery {
        pub fn new(projection: UserProjection) -> Self {
            UserQuery { projection }
        }
        
        /// Get a user by ID, returning formatted user info as a string
        pub fn get_user(&self, user_id: u32) -> Option<String> {
            self.projection.get_user(user_id).map(|user| {
                format!("ID: {} Name: {}", user.id, user.name)
            })
        }
        
        /// Get all users
        pub fn get_all_users(&self) -> Vec<UserReadModel> {
            self.projection.get_all_users()
        }
        
        /// Get count of users
        pub fn get_user_count(&self) -> usize {
            self.projection.get_all_users().len()
        }
    }
}
