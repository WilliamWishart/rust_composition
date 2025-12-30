/// User Aggregate - Domain Model
/// Represents a user in the domain
#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
}

impl User {
    pub fn new(id: u32, name: String) -> Self {
        User { id, name }
    }
}
