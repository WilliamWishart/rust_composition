// Infrastructure Layer: Cross-cutting concerns and external adapters
// This module contains traits and implementations for logging, error handling, etc.

pub mod logger;
pub mod errors;
pub mod metrics;

pub use logger::{Logger, ConsoleLogger, MockLogger};
pub use errors::{DomainError, DomainResult};
pub use metrics::{HandlerMetrics, MetricsRegistry, MetricsSummary};
