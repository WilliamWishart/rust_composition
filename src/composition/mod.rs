// Composition Root: Dependency Injection and wiring
// This layer is responsible for creating and wiring all dependencies
// It's the only place that knows about concrete implementations

pub mod app_builder;

pub use app_builder::AppBuilder;
