use serde::{Deserialize, Serialize};

/// User
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub name: String,
    pub email: String,
}
