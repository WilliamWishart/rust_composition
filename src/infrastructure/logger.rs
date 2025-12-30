use std::sync::{Arc, Mutex};

/// Logger trait - defines the logging capability
/// Implemented by different loggers depending on the environment
pub trait Logger: Send + Sync {
    fn log(&self, message: &str);
}

/// Production logger - logs to console
pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("📝 [CONSOLE] {}", message);
    }
}

/// Test logger - collects messages for verification
#[derive(Clone)]
pub struct MockLogger {
    messages: Arc<Mutex<Vec<String>>>,
}

impl MockLogger {
    pub fn new() -> Self {
        MockLogger {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_messages(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }
}

impl Logger for MockLogger {
    fn log(&self, message: &str) {
        self.messages.lock().unwrap().push(message.to_string());
    }
}
