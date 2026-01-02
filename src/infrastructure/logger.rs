use std::sync::{Arc, Mutex};

/// LogLevel - Severity levels for structured logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

/// Logger trait - defines the structured logging capability
/// Implemented by different loggers depending on the environment
pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
    
    /// Convenience methods for different log levels
    fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }
    
    fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }
    
    fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }
    
    fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }
}

/// Production logger - logs to console with level prefix
pub struct ConsoleLogger {
    min_level: LogLevel,
}

impl ConsoleLogger {
    pub fn new(min_level: LogLevel) -> Self {
        ConsoleLogger { min_level }
    }
}

impl Default for ConsoleLogger {
    fn default() -> Self {
        ConsoleLogger::new(LogLevel::Info)
    }
}

impl Logger for ConsoleLogger {
    fn log(&self, level: LogLevel, message: &str) {
        if level >= self.min_level {
            println!("[{}] {}", level, message);
        }
    }
}

/// Test logger - collects messages with levels for verification
#[derive(Clone)]
pub struct MockLogger {
    messages: Arc<Mutex<Vec<(LogLevel, String)>>>,
}

impl MockLogger {
    pub fn new() -> Self {
        MockLogger {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_messages(&self) -> Vec<(LogLevel, String)> {
        self.messages.lock().unwrap().clone()
    }
    
    pub fn get_messages_as_strings(&self) -> Vec<String> {
        self.messages
            .lock()
            .unwrap()
            .iter()
            .map(|(level, msg)| format!("[{}] {}", level, msg))
            .collect()
    }
    
    pub fn clear(&self) {
        self.messages.lock().unwrap().clear();
    }
}

impl Logger for MockLogger {
    fn log(&self, level: LogLevel, message: &str) {
        self.messages.lock().unwrap().push((level, message.to_string()));
    }
}
