use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use sea_orm::{DeriveValueType, EnumIter};
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, FromRow};

use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Encode, Decode, FromRow, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct NodePosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, sqlx::Type, FromRow, Deserialize, Serialize)]
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

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, EnumIter, Serialize, Deserialize, ToSchema, DeriveValueType,
)]
#[sea_orm(value_type = "String")]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Person,
    Domain,
    Ip,
    Phone,
    Email,
    Url,
    Image,
    Location,
    Organisation,
    Document,
    Currency,
}

impl AsRef<str> for NodeType {
    fn as_ref(&self) -> &str {
        match self {
            NodeType::Person => "person",
            NodeType::Domain => "domain",
            NodeType::Ip => "ip",
            NodeType::Phone => "phone",
            NodeType::Email => "email",
            NodeType::Url => "url",
            NodeType::Image => "image",
            NodeType::Location => "location",
            NodeType::Organisation => "organisation",
            NodeType::Document => "document",
            NodeType::Currency => "currency",
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl TryFrom<&str> for NodeType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "person" => Ok(NodeType::Person),
            "domain" => Ok(NodeType::Domain),
            "ip" => Ok(NodeType::Ip),
            "phone" => Ok(NodeType::Phone),
            "email" => Ok(NodeType::Email),
            "url" => Ok(NodeType::Url),
            "image" => Ok(NodeType::Image),
            "location" => Ok(NodeType::Location),
            "organisation" => Ok(NodeType::Organisation),
            "document" => Ok(NodeType::Document),
            "currency" => Ok(NodeType::Currency),
            _ => Err(format!("Unknown NodeType: {}", value)),
        }
    }
}

impl FromStr for NodeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.try_into()
    }
}

impl TryFrom<String> for NodeType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_node_position_creation() {
        let pos = NodePosition { x: 100, y: 200 };
        assert_eq!(pos.x, 100);
        assert_eq!(pos.y, 200);
    }

    #[test]
    fn test_node_update_list_new() {
        let list = NodeUpdateList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_node_update_list_default() {
        let list = NodeUpdateList::default();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_node_update_list_insert_and_get() {
        let mut list = NodeUpdateList::new();
        let id = Uuid::new_v4();
        let time = Utc::now();

        list.insert(id, time);

        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
        assert_eq!(list.get(&id), Some(&time));

        let non_existent_id = Uuid::new_v4();
        assert_eq!(list.get(&non_existent_id), None);
    }

    #[test]
    fn test_node_update_list_iter() {
        let mut list = NodeUpdateList::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let time1 = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2023, 1, 2, 12, 0, 0).unwrap();

        list.insert(id1, time1);
        list.insert(id2, time2);

        let items: Vec<_> = list.iter().collect();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&(&id1, &time1)));
        assert!(items.contains(&(&id2, &time2)));
    }

    #[test]
    fn test_get_newer_from_empty_lists() {
        let list1 = NodeUpdateList::new();
        let list2 = NodeUpdateList::new();

        let result = list1.get_newer_from(&list2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_newer_from_with_new_items() {
        let mut list1 = NodeUpdateList::new();
        let mut list2 = NodeUpdateList::new();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let time1 = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2023, 1, 2, 12, 0, 0).unwrap();

        list1.insert(id1, time1);
        list2.insert(id1, time1);
        list2.insert(id2, time2); // This is new in list2

        let result = list1.get_newer_from(&list2);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&id2), Some(&time2));
    }

    #[test]
    fn test_get_newer_from_with_updated_items() {
        let mut list1 = NodeUpdateList::new();
        let mut list2 = NodeUpdateList::new();

        let id = Uuid::new_v4();
        let old_time = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let new_time = Utc.with_ymd_and_hms(2023, 1, 2, 12, 0, 0).unwrap();

        list1.insert(id, old_time);
        list2.insert(id, new_time); // This is newer in list2

        let result = list1.get_newer_from(&list2);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&id), Some(&new_time));
    }

    #[test]
    fn test_get_newer_from_with_older_items() {
        let mut list1 = NodeUpdateList::new();
        let mut list2 = NodeUpdateList::new();

        let id = Uuid::new_v4();
        let old_time = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let new_time = Utc.with_ymd_and_hms(2023, 1, 2, 12, 0, 0).unwrap();

        list1.insert(id, new_time);
        list2.insert(id, old_time); // This is older in list2

        let result = list1.get_newer_from(&list2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_newer_than_empty_lists() {
        let list1 = NodeUpdateList::new();
        let list2 = NodeUpdateList::new();

        let result = list1.get_newer_than(&list2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_newer_than_with_new_items() {
        let mut list1 = NodeUpdateList::new();
        let mut list2 = NodeUpdateList::new();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let time1 = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2023, 1, 2, 12, 0, 0).unwrap();

        list1.insert(id1, time1);
        list1.insert(id2, time2); // This is new in list1
        list2.insert(id1, time1);

        let result = list1.get_newer_than(&list2);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&id2), Some(&time2));
    }

    #[test]
    fn test_get_newer_than_with_updated_items() {
        let mut list1 = NodeUpdateList::new();
        let mut list2 = NodeUpdateList::new();

        let id = Uuid::new_v4();
        let old_time = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let new_time = Utc.with_ymd_and_hms(2023, 1, 2, 12, 0, 0).unwrap();

        list1.insert(id, new_time); // This is newer in list1
        list2.insert(id, old_time);

        let result = list1.get_newer_than(&list2);
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&id), Some(&new_time));
    }

    #[test]
    fn test_get_newer_than_with_older_items() {
        let mut list1 = NodeUpdateList::new();
        let mut list2 = NodeUpdateList::new();

        let id = Uuid::new_v4();
        let old_time = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let new_time = Utc.with_ymd_and_hms(2023, 1, 2, 12, 0, 0).unwrap();

        list1.insert(id, old_time); // This is older in list1
        list2.insert(id, new_time);

        let result = list1.get_newer_than(&list2);
        assert!(result.is_empty());
    }

    #[test]
    fn test_complex_scenario() {
        let mut list1 = NodeUpdateList::new();
        let mut list2 = NodeUpdateList::new();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();
        let id4 = Uuid::new_v4();

        let time1 = Utc.with_ymd_and_hms(2023, 1, 1, 12, 0, 0).unwrap();
        let time2 = Utc.with_ymd_and_hms(2023, 1, 2, 12, 0, 0).unwrap();
        let time3 = Utc.with_ymd_and_hms(2023, 1, 3, 12, 0, 0).unwrap();
        let time4 = Utc.with_ymd_and_hms(2023, 1, 4, 12, 0, 0).unwrap();

        // list1 has: id1(old), id2(new), id3(only in list1)
        list1.insert(id1, time1);
        list1.insert(id2, time3);
        list1.insert(id3, time2);

        // list2 has: id1(new), id2(old), id4(only in list2)
        list2.insert(id1, time2);
        list2.insert(id2, time1);
        list2.insert(id4, time4);

        let newer_from_list2 = list1.get_newer_from(&list2);
        assert_eq!(newer_from_list2.len(), 2); // id1 (updated) and id4 (new)
        assert_eq!(newer_from_list2.get(&id1), Some(&time2));
        assert_eq!(newer_from_list2.get(&id4), Some(&time4));

        let newer_than_list2 = list1.get_newer_than(&list2);
        assert_eq!(newer_than_list2.len(), 2); // id2 (updated) and id3 (new)
        assert_eq!(newer_than_list2.get(&id2), Some(&time3));
        assert_eq!(newer_than_list2.get(&id3), Some(&time2));
    }
}
