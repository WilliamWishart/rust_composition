/// Database trait - represents data persistence
pub trait Database: Send + Sync {
    fn query(&self, sql: &str) -> String;
}

/// Production-like database (for demo purposes)
pub struct MockDatabase;

impl Database for MockDatabase {
    fn query(&self, sql: &str) -> String {
        format!("[MOCK] Executed: {}", sql)
    }
}
