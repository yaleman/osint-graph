use osint_graph_shared::nodelink::LinkType;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[sea_orm(table_name = "node_link")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub left: Uuid,
    pub right: Uuid,
    pub project_id: Uuid,
    #[sea_orm(column_type = "String(StringLen::N(15))")]
    pub linktype: LinkType,
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
    #[sea_orm(
        belongs_to = "super::node::Entity",
        from = "Column::Left",
        to = "super::node::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    LeftNode,
    #[sea_orm(
        belongs_to = "super::node::Entity",
        from = "Column::Right",
        to = "super::node::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    RightNode,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::node::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LeftNode.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
