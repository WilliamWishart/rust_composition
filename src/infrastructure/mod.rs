// Infrastructure Layer: Cross-cutting concerns and external adapters
// This module contains traits and implementations for logging, database access, etc.

pub mod logger;
pub mod database;

pub use logger::{Logger, ConsoleLogger, MockLogger};
pub use database::{Database, MockDatabase};
