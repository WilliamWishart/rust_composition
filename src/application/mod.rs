// Application Layer: Services that orchestrate domain logic
// 
// NOTE: In CQRS architecture, application services are not needed.
// Instead use:
// - Commands → CommandHandler (write side)
// - Queries → UserQuery (read side)
//
// UserService was removed as it duplicated repository logic.

pub mod user_service;
