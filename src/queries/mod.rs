// Queries Module: CQRS read side
// Queries retrieve data from read models (projections)
// Read models are eventually consistent with the write model

pub mod user_queries;

pub use user_queries::UserQuery;
