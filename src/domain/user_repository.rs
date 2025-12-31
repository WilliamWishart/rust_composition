// UserRepository - REMOVED
// 
// This was a legacy data access pattern that bypassed event sourcing.
// It violated CQRS principles by mixing write model (save_user) with read model (get_user).
//
// Proper replacement:
// - Write operations: Use Repository with event sourcing (src/domain/repository.rs)
// - Read operations: Use UserQuery with projections (src/queries/user_queries.rs)
//
// This ensures:
// - Commands flow through aggregates to events (CQRS write model)
// - Queries read from projections (CQRS read model)
// - Event sourcing provides a complete audit trail
// - No direct database access bypassing the event stream
