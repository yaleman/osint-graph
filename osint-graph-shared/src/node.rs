use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, FromRow};

use uuid::Uuid;

#[derive(Encode, Decode, FromRow, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct NodePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Encode, Decode, FromRow, Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default)]
pub struct Node {
    pub project_id: Uuid,
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub value: String,
    pub updated: DateTime<Utc>,
    pub notes: Option<String>,
    // TODO: ownership
    // pub position: NodePosition,
    pub pos_x: Option<i32>,
    pub pos_y: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, FromRow)]
pub struct NodeUpdateList(HashMap<Uuid, DateTime<Utc>>);

impl Default for NodeUpdateList {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeUpdateList {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Based on another list, find any items in that list that are newer than this one
    pub fn get_newer_from(&self, other: &NodeUpdateList) -> NodeUpdateList {
        let mut new_updates = NodeUpdateList::new();
        for (id, time) in other.iter() {
            if let Some(my_time) = self.get(id) {
                if time > my_time {
                    new_updates.insert(id.to_owned(), time.to_owned());
                }
            } else {
                new_updates.insert(id.to_owned(), time.to_owned());
            }
        }
        new_updates
    }

    /// Based on another list, find any items in self that are newer
    pub fn get_newer_than(&self, other: &NodeUpdateList) -> NodeUpdateList {
        let mut new_updates = NodeUpdateList::new();
        for (id, time) in self.iter() {
            if let Some(their_time) = other.get(id) {
                if time > their_time {
                    new_updates.insert(id.to_owned(), time.to_owned());
                }
            } else {
                new_updates.insert(id.to_owned(), time.to_owned());
            }
        }
        new_updates
    }

    pub fn insert(&mut self, value: Uuid, time: DateTime<Utc>) {
        self.0.insert(value, time);
    }
    pub fn get(&self, value: &Uuid) -> Option<&DateTime<Utc>> {
        self.0.get(value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &DateTime<Utc>)> {
        self.0.iter()
    }
}
