// Infrastructure Layer: Cross-cutting concerns and external adapters
// This module contains traits and implementations for logging, etc.

pub mod logger;

pub use logger::{Logger, ConsoleLogger, MockLogger};
