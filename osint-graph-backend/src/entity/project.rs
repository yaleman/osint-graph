use chrono::{DateTime, Utc};
use osint_graph_shared::StringVec;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "project")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub user: Uuid,
    pub creationdate: DateTime<Utc>,
    pub last_updated: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub tags: StringVec,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::node::Entity")]
    Nodes,
    #[sea_orm(has_many = "super::nodelink::Entity")]
    NodeLinks,
}

impl Related<super::node::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nodes.def()
    }
}

impl Related<super::nodelink::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NodeLinks.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
