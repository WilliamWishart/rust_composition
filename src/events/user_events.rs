use std::fmt;

/// UserEvent - Enum-based domain events for User aggregate
/// Using a concrete enum instead of trait objects gives us:
/// - Compile-time exhaustiveness checking
/// - Zero runtime overhead (no vtable, no Arc)
/// - Type safety without downcasting
/// - Pattern matching instead of string comparisons
#[derive(Debug, Clone)]
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
    /// Get the aggregate ID that this event belongs to
    pub fn aggregate_id(&self) -> u32 {
        match self {
            UserEvent::Registered { user_id, .. } => *user_id,
            UserEvent::Renamed { user_id, .. } => *user_id,
        }
    }

    /// Get event type name for logging/debugging
    pub fn event_type(&self) -> &str {
        match self {
            UserEvent::Registered { .. } => "UserRegistered",
            UserEvent::Renamed { .. } => "UserRenamed",
        }
    }

    /// Get timestamp when event occurred
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
