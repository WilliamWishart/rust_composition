use std::sync::{Arc, Mutex};
use crate::events::UserEvent;
use async_trait::async_trait;
use std::fmt;

/// HandlerError - Error information from a failed event handler
#[derive(Debug, Clone)]
pub struct HandlerError {
    pub handler_name: String,
    pub error_message: String,
    pub is_critical: bool,
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Handler '{}' failed: {}", self.handler_name, self.error_message)
    }
}

impl std::error::Error for HandlerError {}

/// PublishError - Errors that can occur during event publishing
#[derive(Debug, Clone)]
pub enum PublishError {
    LockPoisoned,
    CriticalHandlerFailed(HandlerError),
    SomeHandlersFailed(Vec<HandlerError>),
}

impl fmt::Display for PublishError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PublishError::LockPoisoned => write!(f, "Subscriber list lock was poisoned"),
            PublishError::CriticalHandlerFailed(err) => write!(f, "Critical handler failed: {}", err),
            PublishError::SomeHandlersFailed(errs) => write!(f, "{} handler(s) failed", errs.len()),
        }
    }
}

impl std::error::Error for PublishError {}

/// HandlerPriority - Controls execution ordering and guarantees
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandlerPriority {
    Critical = 3,
    High = 2,
    Normal = 1,
    Low = 0,
}

/// EventHandler - Trait for components that handle domain events asynchronously
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_event(&self, event: &UserEvent) -> Result<(), Box<dyn std::error::Error>>;
    
    fn priority(&self) -> HandlerPriority {
        HandlerPriority::Normal
    }
    
    fn name(&self) -> &str {
        "UnnamedHandler"
    }
}

/// EventBus - Implements pub/sub for domain events with async support
/// Publishes events to subscribers asynchronously for eventual consistency
/// Non-blocking: handlers execute concurrently without blocking the publisher
#[derive(Clone)]
pub struct EventBus {
    subscribers: Arc<Mutex<Vec<Arc<dyn EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a subscriber to receive events
    pub fn subscribe<H: EventHandler + 'static>(&self, handler: Arc<H>) {
        self.subscribers.lock().unwrap().push(handler as Arc<dyn EventHandler>);
    }

    /// Publish a UserEvent asynchronously with proper error handling and priority-based execution
    /// 
    /// Execution order:
    /// 1. Critical handlers execute synchronously (blocking), in order, must all succeed
    /// 2. High priority handlers spawn concurrently, awaited for completion
    /// 3. Normal priority handlers spawn concurrently (eventual consistency)
    /// 4. Low priority handlers spawn as background tasks
    /// 
    /// Returns Ok(errors) if no critical handlers failed, Err if critical handler fails
    pub async fn publish(&self, event: &UserEvent) -> Result<Vec<HandlerError>, PublishError> {
        // Acquire lock and collect Arc clones (cheap - just increments refcount)
        // Release lock immediately to prevent blocking other subscribers
        let subscribers = {
            let subs = self.subscribers.lock()
                .map_err(|_| PublishError::LockPoisoned)?;
            subs.iter().map(Arc::clone).collect::<Vec<_>>()
        };
        
        // Partition handlers by priority
        let mut critical_handlers = Vec::new();
        let mut high_handlers = Vec::new();
        let mut normal_handlers = Vec::new();
        let mut low_handlers = Vec::new();
        
        for handler in subscribers {
            match handler.priority() {
                HandlerPriority::Critical => critical_handlers.push(handler),
                HandlerPriority::High => high_handlers.push(handler),
                HandlerPriority::Normal => normal_handlers.push(handler),
                HandlerPriority::Low => low_handlers.push(handler),
            }
        }
        
        // Phase 1: Execute critical handlers synchronously (must all succeed)
        for handler in critical_handlers {
            match handler.handle_event(event).await {
                Ok(()) => {},
                Err(e) => {
                    let error = HandlerError {
                        handler_name: handler.name().to_string(),
                        error_message: e.to_string(),
                        is_critical: true,
                    };
                    return Err(PublishError::CriticalHandlerFailed(error));
                }
            }
        }
        
        // Phase 2 & 3: Spawn high and normal priority handlers concurrently
        let handles: Vec<_> = high_handlers
            .into_iter()
            .chain(normal_handlers.into_iter())
            .map(|handler| {
                let event = event.clone();
                let handler_name = handler.name().to_string();
                tokio::spawn(async move {
                    match handler.handle_event(&event).await {
                        Ok(()) => None,
                        Err(e) => Some(HandlerError {
                            handler_name,
                            error_message: e.to_string(),
                            is_critical: false,
                        }),
                    }
                })
            })
            .collect();
        
        // Phase 4: Spawn low priority handlers (fire-and-forget)
        for handler in low_handlers {
            let event = event.clone();
            tokio::spawn(async move {
                let _ = handler.handle_event(&event).await;
            });
        }
        
        // Collect errors from spawned handlers
        let mut errors = Vec::new();
        for handle in handles {
            if let Ok(Some(err)) = handle.await {
                errors.push(err);
            }
        }
        
        Ok(errors)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
