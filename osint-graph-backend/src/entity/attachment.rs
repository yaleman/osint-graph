use chrono::Utc;
use sea_orm::{entity::prelude::*, FromQueryResult, JoinType, QuerySelect, SelectModel, Selector};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entity::project;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "attachment")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub node_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    #[sea_orm(column_type = "VarBinary(StringLen::Max)")]
    pub data: Vec<u8>,
    pub created: chrono::DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::node::Entity",
        from = "Column::NodeId",
        to = "super::node::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Node,
}

impl Related<super::node::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Node.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(FromQueryResult)]
pub struct ModelNoAttachment {
    pub id: Uuid,
    pub node_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub created: chrono::DateTime<Utc>,
}

pub fn attachment_list(project_id: Uuid) -> Selector<SelectModel<ModelNoAttachment>> {
    Entity::find()
        .join(
            JoinType::InnerJoin,
            Entity::belongs_to(super::node::Entity)
                .from(Column::NodeId)
                .to(super::node::Column::Id)
                .into(),
        )
        .join(
            JoinType::InnerJoin,
            super::node::Entity::belongs_to(project::Entity)
                .from(super::node::Column::ProjectId)
                .to(project::Column::Id)
                .into(),
        )
        .filter(project::Column::Id.eq(project_id))
        .columns([
            Column::Id,
            Column::NodeId,
            Column::Filename,
            Column::ContentType,
            Column::Size,
            Column::Created,
        ])
        .into_model::<ModelNoAttachment>()
}

impl From<ModelNoAttachment> for Model {
    fn from(no_attachment: ModelNoAttachment) -> Self {
        Self {
            id: no_attachment.id,
            node_id: no_attachment.node_id,
            filename: no_attachment.filename,
            content_type: no_attachment.content_type,
            size: no_attachment.size,
            data: Vec::new(), // Data is not included in ModelNoAttachment
            created: no_attachment.created,
        }
    }
}
