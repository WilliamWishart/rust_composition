// Event Bus for pub/sub
use std::sync::{Arc, Mutex};
use domain::events::UserEvent;
use async_trait::async_trait;
use std::fmt;
use infrastructure::Logger;
use infrastructure::MetricsRegistry;

/// HandlerError
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

/// PublishError
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

/// HandlerPriority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandlerPriority {
    Critical = 3,
    High = 2,
    Normal = 1,
    Low = 0,
}

/// EventHandler
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

/// EventBus
#[derive(Clone)]
pub struct EventBus {
    subscribers: Arc<Mutex<Vec<Arc<dyn EventHandler>>>>,
    logger: Arc<dyn Logger>,
    metrics: MetricsRegistry,
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            subscribers: Arc::new(Mutex::new(Vec::new())),
            logger: Arc::new(infrastructure::ConsoleLogger::default()),
            metrics: MetricsRegistry::new(),
        }
    }
    
    pub fn with_logger(mut self, logger: Arc<dyn Logger>) -> Self {
        self.logger = logger;
        self
    }

    pub fn subscribe<H: EventHandler + 'static>(&self, handler: Arc<H>) {
        self.subscribers.lock().unwrap().push(handler as Arc<dyn EventHandler>);
    }

    pub async fn publish(&self, event: &UserEvent) -> Result<Vec<HandlerError>, PublishError> {
        self.logger.info(&format!("Publishing event: {:?}", event));
        
        let subscribers = {
            let subs = self.subscribers.lock()
                .map_err(|_| PublishError::LockPoisoned)?;
            subs.iter().map(Arc::clone).collect::<Vec<_>>()
        };
        
        let mut errors = Vec::new();
        
        for handler in subscribers {
            match tokio::time::timeout(
                std::time::Duration::from_secs(30),
                handler.handle_event(event)
            ).await {
                Ok(Ok(())) => {
                    self.metrics.record_success(handler.name(), 0);
                }
                Ok(Err(e)) => {
                    let err = HandlerError {
                        handler_name: handler.name().to_string(),
                        error_message: e.to_string(),
                        is_critical: handler.priority() == HandlerPriority::Critical,
                    };
                    
                    if err.is_critical {
                        self.logger.error(&format!("Critical handler failed: {}", err));
                        return Err(PublishError::CriticalHandlerFailed(err));
                    } else {
                        self.logger.warn(&format!("Non-critical handler failed: {}", err));
                        errors.push(err);
                    }
                }
                Err(_) => {
                    let err = HandlerError {
                        handler_name: handler.name().to_string(),
                        error_message: "Handler timeout".to_string(),
                        is_critical: handler.priority() == HandlerPriority::Critical,
                    };
                    
                    if err.is_critical {
                        return Err(PublishError::CriticalHandlerFailed(err));
                    } else {
                        errors.push(err);
                    }
                }
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
