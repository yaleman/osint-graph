use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Node {
    project_id: Uuid,
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,

    value: String,

    notes: Option<String>,
    // maybe related nodes?
}
