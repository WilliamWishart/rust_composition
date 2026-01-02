use crate::events::UserEvent;
use std::fmt;

/// EventEnvelope - Wraps domain events with metadata for event sourcing
/// 
/// Enables:
/// - Event correlation across systems (correlation_id)
/// - Distributed tracing support (trace tracking)
/// - Event schema versioning (event_version)
/// - Replay and dead letter queue operations
/// - Audit trails with exact timestamps
#[derive(Debug, Clone, PartialEq)]
pub struct EventEnvelope {
    /// Unique identifier for this event
    pub event_id: String,
    
    /// ID of the aggregate this event belongs to
    pub aggregate_id: u32,
    
    /// Type name of the aggregate (e.g., "User")
    pub aggregate_type: String,
    
    /// The actual domain event
    pub event: UserEvent,
    
    /// Version of this event in the aggregate's history (0-indexed)
    pub event_version: i32,
    
    /// Timestamp when the event was created (milliseconds since epoch)
    pub timestamp: i64,
    
    /// Correlation ID for tracking related events across system boundaries
    /// Useful for distributed tracing and multi-step workflows
    pub correlation_id: String,
    
    /// Causation ID - the ID of the command that caused this event
    pub causation_id: Option<String>,
}

impl EventEnvelope {
    /// Create a new event envelope with auto-generated IDs
    pub fn new(
        aggregate_id: u32,
        event: UserEvent,
        event_version: i32,
        correlation_id: String,
    ) -> Self {
        EventEnvelope {
            event_id: Self::generate_event_id(),
            aggregate_id,
            aggregate_type: "User".to_string(),
            event,
            event_version,
            timestamp: chrono::Utc::now().timestamp_millis(),
            correlation_id,
            causation_id: None,
        }
    }

    /// Create a new event envelope with causation tracking
    pub fn with_causation(
        aggregate_id: u32,
        event: UserEvent,
        event_version: i32,
        correlation_id: String,
        causation_id: String,
    ) -> Self {
        EventEnvelope {
            event_id: Self::generate_event_id(),
            aggregate_id,
            aggregate_type: "User".to_string(),
            event,
            event_version,
            timestamp: chrono::Utc::now().timestamp_millis(),
            correlation_id,
            causation_id: Some(causation_id),
        }
    }

    /// Generate a unique event ID (simple UUID-like string)
    fn generate_event_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("event_{}", nanos)
    }

    /// Get the event type name
    pub fn event_type(&self) -> &str {
        self.event.event_type()
    }
}

impl fmt::Display for EventEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EventEnvelope {{ id={}, agg_id={}, type={}, event={}, version={}, corr_id={} }}",
            self.event_id, self.aggregate_id, self.aggregate_type, self.event, self.event_version, self.correlation_id
        )
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
            timestamp: 1234567890,
        };

        let envelope = EventEnvelope::new(1, event, 0, "corr-123".to_string());
        
        assert_eq!(envelope.aggregate_id, 1);
        assert_eq!(envelope.event_version, 0);
        assert_eq!(envelope.aggregate_type, "User");
        assert_eq!(envelope.correlation_id, "corr-123");
        assert!(envelope.event_id.starts_with("event_"));
    }

    #[test]
    fn test_event_envelope_with_causation() {
        let event = UserEvent::Renamed {
            user_id: 1,
            new_name: "Bob".to_string(),
            timestamp: 1234567890,
        };

        let envelope = EventEnvelope::with_causation(
            1,
            event,
            1,
            "corr-123".to_string(),
            "cmd-456".to_string(),
        );

        assert_eq!(envelope.causation_id, Some("cmd-456".to_string()));
    }
}
