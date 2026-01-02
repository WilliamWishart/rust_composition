// Domain events - pure data structures representing facts about what happened
use std::fmt;

/// UserEvent - Enum-based domain events for User aggregate
#[derive(Debug, Clone, PartialEq)]
pub enum UserEvent {
    Registered {
        user_id: u32,
        name: String,
        timestamp: i64,
    },
    Renamed {
        user_id: u32,
        new_name: String,
        timestamp: i64,
    },
}

impl UserEvent {
    pub fn aggregate_id(&self) -> u32 {
        match self {
            UserEvent::Registered { user_id, .. } => *user_id,
            UserEvent::Renamed { user_id, .. } => *user_id,
        }
    }

    pub fn event_type(&self) -> &str {
        match self {
            UserEvent::Registered { .. } => "UserRegistered",
            UserEvent::Renamed { .. } => "UserRenamed",
        }
    }

    pub fn timestamp(&self) -> i64 {
        match self {
            UserEvent::Registered { timestamp, .. } => *timestamp,
            UserEvent::Renamed { timestamp, .. } => *timestamp,
        }
    }
}

impl fmt::Display for UserEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserEvent::Registered {
                user_id,
                name,
                timestamp,
            } => {
                write!(
                    f,
                    "UserRegistered(id={}, name={}, timestamp={})",
                    user_id, name, timestamp
                )
            }
            UserEvent::Renamed {
                user_id,
                new_name,
                timestamp,
            } => {
                write!(
                    f,
                    "UserRenamed(id={}, new_name={}, timestamp={})",
                    user_id, new_name, timestamp
                )
            }
        }
    }
}

/// EventEnvelope - Wraps events with metadata for distributed tracing
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub aggregate_id: u32,
    pub aggregate_type: String,
    pub event: UserEvent,
    pub event_version: i32,
    pub timestamp: i64,
    pub correlation_id: String,
    pub causation_id: Option<String>,
}

impl EventEnvelope {
    pub fn new(
        aggregate_id: u32,
        event: UserEvent,
        event_version: i32,
        correlation_id: String,
    ) -> Self {
        EventEnvelope {
            aggregate_id,
            aggregate_type: "User".to_string(),
            event,
            event_version,
            timestamp: chrono::Utc::now().timestamp_millis(),
            correlation_id,
            causation_id: None,
        }
    }

    pub fn with_causation_id(mut self, causation_id: String) -> Self {
        self.causation_id = Some(causation_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_envelope_creation() {
        let event = UserEvent::Registered {
            user_id: 1,
            name: "Alice".to_string(),
            timestamp: 1000,
        };

        let envelope = EventEnvelope::new(1, event.clone(), 0, "corr_123".to_string());

        assert_eq!(envelope.aggregate_id, 1);
        assert_eq!(envelope.event, event);
        assert_eq!(envelope.correlation_id, "corr_123");
        assert_eq!(envelope.causation_id, None);
    }

    #[test]
    fn test_event_envelope_with_causation() {
        let event = UserEvent::Registered {
            user_id: 1,
            name: "Alice".to_string(),
            timestamp: 1000,
        };

        let envelope = EventEnvelope::new(1, event, 0, "corr_123".to_string())
            .with_causation_id("cmd_456".to_string());

        assert_eq!(envelope.causation_id, Some("cmd_456".to_string()));
    }
}
