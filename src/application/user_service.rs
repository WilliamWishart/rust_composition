// UserService - REMOVED
// 
// This was redundant in a CQRS architecture.
// Use instead:
// - Commands: UserCommandHandler handles all write operations
// - Queries: UserQuery handles all read operations
//
// The proper flow is:
// Commands → CommandHandler → Repository → EventStore
// EventStore → Projections → Queries
//
// Application services are not needed when using CQRS with event sourcing.
