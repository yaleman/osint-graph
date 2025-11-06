use chrono::{DateTime, Utc};
use osint_graph_shared::node::NodeType;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub project_id: Uuid,
    #[sea_orm(column_name = "type", column_type = "String(StringLen::N(15))")]
    pub node_type: NodeType,
    pub display: String,
    pub value: String,
    pub updated: DateTime<Utc>,
    pub notes: Option<String>,
    pub pos_x: Option<i32>,
    pub pos_y: Option<i32>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            node_type: NodeType::Document,
            display: String::new(),
            value: String::new(),
            updated: Utc::now(),
            notes: None,
            pos_x: None,
            pos_y: None,
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Project,
    #[sea_orm(has_many = "super::attachment::Entity")]
    Attachments,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::attachment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attachments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
