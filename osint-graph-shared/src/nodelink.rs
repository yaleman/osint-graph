use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, FromRow};
use uuid::Uuid;

#[derive(Copy, sqlx::Type, Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum LinkType {
    #[default]
    Omni,
    Directional,
}

#[derive(Encode, Decode, FromRow, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct NodeLink {
    pub id: Uuid,
    pub left: Uuid,
    pub right: Uuid,
    pub project_id: Uuid,
    pub linktype: LinkType,
}

impl NodeLink {
    pub fn new(left: Uuid, right: Uuid, project_id: Uuid, linktype: LinkType) -> Self {
        Self {
            id: Uuid::new_v4(),
            left,
            right,
            project_id,
            linktype,
        }
    }
}
