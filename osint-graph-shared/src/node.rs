use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub project_id: Uuid,
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: String,
    pub updated: DateTime<Utc>,
    pub notes: Option<String>,
    // maybe related nodes?
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeUpdateList(pub BTreeMap<Uuid, DateTime<Utc>>);

impl Default for NodeUpdateList {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeUpdateList {
    pub fn new() -> Self {
        NodeUpdateList(BTreeMap::new())
    }

    /// Based on another list, find any items in that list that are newer than this one
    pub fn get_newer_from(&self, other: &NodeUpdateList) -> NodeUpdateList {
        let mut new_updates = BTreeMap::new();
        for (id, time) in other.0.iter() {
            if let Some(my_time) = self.0.get(id) {
                if time > my_time {
                    new_updates.insert(id.to_owned(), time.to_owned());
                }
            } else {
                new_updates.insert(id.to_owned(), time.to_owned());
            }
        }
        NodeUpdateList(new_updates)
    }

    /// Based on another list, find any items in self that are newer
    pub fn get_newer_than(&self, other: &NodeUpdateList) -> NodeUpdateList {
        let mut new_updates = BTreeMap::new();
        for (id, time) in self.0.iter() {
            if let Some(their_time) = other.0.get(id) {
                if time > their_time {
                    new_updates.insert(id.to_owned(), time.to_owned());
                }
            } else {
                new_updates.insert(id.to_owned(), time.to_owned());
            }
        }
        NodeUpdateList(new_updates)
    }
}
