use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    pub identifier: String,
    pub description: String,
}
