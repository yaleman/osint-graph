use sea_orm::{DeriveActiveEnum, EnumIter};
use sea_query::table::StringLen;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(
    Copy,
    sqlx::Type,
    Default,
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Eq,
    PartialEq,
    EnumIter,
    DeriveActiveEnum,
    ToSchema,
)]
#[sea_orm(
    rs_type = "String",
    db_type = "String(StringLen::N(12))",
    rename_all = "camelCase"
)]
pub enum LinkType {
    #[default]
    Omni,
    Directional,
}
