use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "node")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub project_id: Uuid,
    #[sea_orm(column_name = "type")]
    pub node_type: String,
    pub display: String,
    pub value: String,
    pub updated: DateTime<Utc>,
    pub notes: Option<String>,
    pub pos_x: Option<i32>,
    pub pos_y: Option<i32>,
    pub attachments: serde_json::Value,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            node_type: String::new(),
            display: String::new(),
            value: String::new(),
            updated: Utc::now(),
            notes: None,
            pos_x: None,
            pos_y: None,
            attachments: serde_json::Value::Array(Vec::new()),
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
