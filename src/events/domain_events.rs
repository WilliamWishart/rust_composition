use std::fmt;

/// DomainEvent trait - Base for all domain events
/// Events are immutable facts about what happened in the domain
pub trait DomainEvent: Send + Sync {
    fn event_id(&self) -> String;
    fn aggregate_id(&self) -> String;
    fn event_type(&self) -> &str;
    fn timestamp(&self) -> i64;
}

/// UserRegisteredEvent - Event fired when a user is registered
#[derive(Debug, Clone)]
pub struct UserRegisteredEvent {
    pub event_id: String,
    pub user_id: u32,
    pub name: String,
    pub timestamp: i64,
}

impl UserRegisteredEvent {
    pub fn new(user_id: u32, name: String) -> Self {
        UserRegisteredEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            name,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}

impl DomainEvent for UserRegisteredEvent {
    fn event_id(&self) -> String {
        self.event_id.clone()
    }

    fn aggregate_id(&self) -> String {
        self.user_id.to_string()
    }

    fn event_type(&self) -> &str {
        "UserRegistered"
    }

    fn timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl fmt::Display for UserRegisteredEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UserRegistered(id={}, name={}, timestamp={})",
            self.user_id, self.name, self.timestamp
        )
    }
}
