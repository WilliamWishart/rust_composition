use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// HandlerMetrics - Performance metrics for a single event handler
#[derive(Debug, Clone)]
pub struct HandlerMetrics {
    pub handler_name: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_retries: u64,
    pub successful_retries: u64,
    pub failed_after_retries: u64,
    pub total_execution_time_ms: u64,
    pub min_execution_time_ms: u64,
    pub max_execution_time_ms: u64,
    pub timeout_count: u64,
}

impl HandlerMetrics {
    pub fn new(handler_name: String) -> Self {
        HandlerMetrics {
            handler_name,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_retries: 0,
            successful_retries: 0,
            failed_after_retries: 0,
            total_execution_time_ms: 0,
            min_execution_time_ms: u64::MAX,
            max_execution_time_ms: 0,
            timeout_count: 0,
        }
    }

    /// Calculate average execution time in milliseconds
    pub fn avg_execution_time_ms(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.total_execution_time_ms as f64 / self.total_executions as f64
        }
    }

    /// Calculate success rate as percentage (0-100)
    pub fn success_rate_percent(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.successful_executions as f64 / self.total_executions as f64) * 100.0
        }
    }

    /// Calculate retry rate (retries per 100 executions)
    pub fn retry_rate_percent(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.total_retries as f64 / self.total_executions as f64) * 100.0
        }
    }

    /// Calculate failure rate after retries exhausted
    pub fn failure_after_retries_rate_percent(&self) -> f64 {
        if self.total_retries == 0 {
            0.0
        } else {
            (self.failed_after_retries as f64 / self.total_retries as f64) * 100.0
        }
    }
}

/// MetricsRegistry - Thread-safe registry of handler metrics
#[derive(Clone)]
pub struct MetricsRegistry {
    metrics: Arc<Mutex<HashMap<String, HandlerMetrics>>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        MetricsRegistry {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Record a successful handler execution
    pub fn record_success(&self, handler_name: &str, duration_ms: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        let stats = metrics
            .entry(handler_name.to_string())
            .or_insert_with(|| HandlerMetrics::new(handler_name.to_string()));

        stats.total_executions += 1;
        stats.successful_executions += 1;
        stats.total_execution_time_ms += duration_ms;
        stats.min_execution_time_ms = stats.min_execution_time_ms.min(duration_ms);
        stats.max_execution_time_ms = stats.max_execution_time_ms.max(duration_ms);
    }

    /// Record a failed handler execution
    pub fn record_failure(&self, handler_name: &str, duration_ms: u64) {
        let mut metrics = self.metrics.lock().unwrap();
        let stats = metrics
            .entry(handler_name.to_string())
            .or_insert_with(|| HandlerMetrics::new(handler_name.to_string()));

        stats.total_executions += 1;
        stats.failed_executions += 1;
        stats.total_execution_time_ms += duration_ms;
        stats.min_execution_time_ms = stats.min_execution_time_ms.min(duration_ms);
        stats.max_execution_time_ms = stats.max_execution_time_ms.max(duration_ms);
    }

    /// Record a retry attempt
    pub fn record_retry(&self, handler_name: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        let stats = metrics
            .entry(handler_name.to_string())
            .or_insert_with(|| HandlerMetrics::new(handler_name.to_string()));

        stats.total_retries += 1;
    }

    /// Record a successful retry (handler eventually succeeded)
    pub fn record_retry_success(&self, handler_name: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        let stats = metrics
            .entry(handler_name.to_string())
            .or_insert_with(|| HandlerMetrics::new(handler_name.to_string()));

        stats.successful_retries += 1;
    }

    /// Record a failed retry (handler failed after all retries)
    pub fn record_retry_failure(&self, handler_name: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        let stats = metrics
            .entry(handler_name.to_string())
            .or_insert_with(|| HandlerMetrics::new(handler_name.to_string()));

        stats.failed_after_retries += 1;
    }

    /// Record a timeout
    pub fn record_timeout(&self, handler_name: &str) {
        let mut metrics = self.metrics.lock().unwrap();
        let stats = metrics
            .entry(handler_name.to_string())
            .or_insert_with(|| HandlerMetrics::new(handler_name.to_string()));

        stats.timeout_count += 1;
    }

    /// Get metrics for a specific handler
    pub fn get_handler_metrics(&self, handler_name: &str) -> Option<HandlerMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.get(handler_name).cloned()
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> Vec<HandlerMetrics> {
        let metrics = self.metrics.lock().unwrap();
        metrics.values().cloned().collect()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.metrics.lock().unwrap().clear();
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> MetricsSummary {
        let metrics = self.metrics.lock().unwrap();
        let handlers = metrics.values().cloned().collect::<Vec<_>>();

        if handlers.is_empty() {
            return MetricsSummary::default();
        }

        let total_executions: u64 = handlers.iter().map(|m| m.total_executions).sum();
        let total_successful: u64 = handlers.iter().map(|m| m.successful_executions).sum();
        let total_failures: u64 = handlers.iter().map(|m| m.failed_executions).sum();
        let total_timeouts: u64 = handlers.iter().map(|m| m.timeout_count).sum();
        let total_time_ms: u64 = handlers.iter().map(|m| m.total_execution_time_ms).sum();
        let avg_time_ms = if total_executions > 0 {
            total_time_ms as f64 / total_executions as f64
        } else {
            0.0
        };

        MetricsSummary {
            total_handlers: handlers.len() as u32,
            total_executions,
            total_successful,
            total_failures,
            total_timeouts,
            avg_execution_time_ms: avg_time_ms,
            slowest_handler: handlers.iter().max_by_key(|m| m.max_execution_time_ms).cloned(),
            highest_error_rate_handler: handlers
                .iter()
                .max_by(|a, b| {
                    a.success_rate_percent()
                        .partial_cmp(&b.success_rate_percent())
                        .unwrap()
                })
                .cloned(),
        }
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of all metrics
#[derive(Debug, Clone, Default)]
pub struct MetricsSummary {
    pub total_handlers: u32,
    pub total_executions: u64,
    pub total_successful: u64,
    pub total_failures: u64,
    pub total_timeouts: u64,
    pub avg_execution_time_ms: f64,
    pub slowest_handler: Option<HandlerMetrics>,
    pub highest_error_rate_handler: Option<HandlerMetrics>,
}

impl MetricsSummary {
    pub fn overall_success_rate_percent(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.total_successful as f64 / self.total_executions as f64) * 100.0
        }
    }
}
