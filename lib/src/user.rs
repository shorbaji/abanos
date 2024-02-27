use serde::{Deserialize, Serialize};

/// User
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub name: String,
}
