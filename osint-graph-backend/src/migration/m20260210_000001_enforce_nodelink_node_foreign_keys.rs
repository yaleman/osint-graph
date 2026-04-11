use sea_orm::ConnectionTrait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite cannot reliably add foreign keys to an existing table, so rebuild `node_link`.
        manager
            .create_table(
                Table::create()
                    .table(NodeLinkWithNodeFks::Table)
                    .col(
                        ColumnDef::new(NodeLinkWithNodeFks::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NodeLinkWithNodeFks::Left)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NodeLinkWithNodeFks::Right)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NodeLinkWithNodeFks::ProjectId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NodeLinkWithNodeFks::Linktype)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodelink_project_enforced")
                            .from(NodeLinkWithNodeFks::Table, NodeLinkWithNodeFks::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodelink_left_enforced")
                            .from(NodeLinkWithNodeFks::Table, NodeLinkWithNodeFks::Left)
                            .to(Node::Table, Node::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodelink_right_enforced")
                            .from(NodeLinkWithNodeFks::Table, NodeLinkWithNodeFks::Right)
                            .to(Node::Table, Node::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO node_link_with_node_fks (id, left, right, project_id, linktype)
                 SELECT id, left, right, project_id, linktype FROM node_link;",
            )
            .await?;

        manager
            .drop_table(Table::drop().table(NodeLink::Table).to_owned())
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(NodeLinkWithNodeFks::Table, NodeLink::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NodeLinkWithoutNodeFks::Table)
                    .col(
                        ColumnDef::new(NodeLinkWithoutNodeFks::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NodeLinkWithoutNodeFks::Left)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NodeLinkWithoutNodeFks::Right)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NodeLinkWithoutNodeFks::ProjectId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NodeLinkWithoutNodeFks::Linktype)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodelink_project_only")
                            .from(
                                NodeLinkWithoutNodeFks::Table,
                                NodeLinkWithoutNodeFks::ProjectId,
                            )
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "INSERT INTO node_link_without_node_fks (id, left, right, project_id, linktype)
                 SELECT id, left, right, project_id, linktype FROM node_link;",
            )
            .await?;

        manager
            .drop_table(Table::drop().table(NodeLink::Table).to_owned())
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(NodeLinkWithoutNodeFks::Table, NodeLink::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Project {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Node {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum NodeLink {
    #[sea_orm(iden = "node_link")]
    Table,
}

#[derive(DeriveIden)]
enum NodeLinkWithNodeFks {
    #[sea_orm(iden = "node_link_with_node_fks")]
    Table,
    Id,
    Left,
    Right,
    ProjectId,
    Linktype,
}

#[derive(DeriveIden)]
enum NodeLinkWithoutNodeFks {
    #[sea_orm(iden = "node_link_without_node_fks")]
    Table,
    Id,
    Left,
    Right,
    ProjectId,
    Linktype,
}
