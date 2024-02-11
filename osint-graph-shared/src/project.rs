//! Project-related schema
//!

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    #[serde(default = "uuid::Uuid::new_v4")]
    pub id: uuid::Uuid,
    /// Project name
    pub name: String,
    /// Owner/creator of the project
    pub user: uuid::Uuid,
    /// UTC timestamp of the project creation
    #[serde(with = "chrono::serde::ts_milliseconds", default = "Utc::now")]
    pub creationdate: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// UTC timestamp of the last update time
    pub last_updated: Option<DateTime<Utc>>,
}

impl Project {
    pub fn updated(&mut self) -> Self {
        self.last_updated = Some(Utc::now());
        self.clone()
    }

    /// Set the name
    pub fn name(&mut self, name: String) {
        self.name = name
    }
}
