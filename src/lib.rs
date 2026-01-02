// ============================================================================
// ENTERPRISE-SCALE RUST APPLICATION STRUCTURE
// ============================================================================
//
// Layered Crate Architecture (Primary):
// The application is now organized as a layered workspace:
// - crates/domain:         Pure business logic (User aggregate, commands, errors, events)
// - crates/infrastructure: Cross-cutting concerns (Logger, Metrics)
// - crates/persistence:    Data access layer (EventStore, Repository, Projections)
// - crates/application:    Command handlers and event bus orchestration
// - crates/api-rest:       HTTP API layer (Axum routes)
//
// Dependency Flow (Acyclic):
// Domain → Infrastructure → Persistence → Application → API-REST
//
// Legacy src/ Content:
// The modules below are the legacy monolithic structure and are kept for
// backward compatibility with existing tests. New code should use the layered crates.

// Old src/ modules (legacy - kept for backward compatibility with tests)
pub mod events;
pub mod commands;
pub mod queries;
pub mod composition;
pub mod infrastructure;
pub mod domain;
pub mod application;
