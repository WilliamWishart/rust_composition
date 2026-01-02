use std::sync::{Arc, Mutex};
use crate::events::UserEvent;
use async_trait::async_trait;

/// EventHandler - Trait for components that handle domain events asynchronously
/// Handlers can now perform async operations without blocking the event bus
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_event(&self, event: &UserEvent);
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

    /// Publish a UserEvent asynchronously - notify all registered subscribers (eventually consistent)
    /// Spawns concurrent tasks for each subscriber - non-blocking
    pub async fn publish(&self, event: &UserEvent) {
        let subscribers = self.subscribers.lock().unwrap().clone();
        
        // Spawn concurrent tasks for each handler
        let handles: Vec<_> = subscribers
            .iter()
            .map(|subscriber| {
                let event = event.clone();
                let sub = subscriber.clone();
                tokio::spawn(async move {
                    sub.handle_event(&event).await
                })
            })
            .collect();

        // Wait for all handlers to complete
        for handle in handles {
            let _ = handle.await;
        }
    }

    /// Publish synchronously (blocking) - for backward compatibility or when async context not available
    /// Use this sparingly; prefer publish() for non-blocking behavior
    pub fn publish_blocking(&self, event: &UserEvent) {
        // Create a runtime to execute async code
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");
        
        rt.block_on(self.publish(event));
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
