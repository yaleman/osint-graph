use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create project table
        manager
            .create_table(
                Table::create()
                    .table(Project::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Project::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Project::Name).string().not_null())
                    .col(ColumnDef::new(Project::User).string().not_null())
                    .col(ColumnDef::new(Project::Creationdate).string().not_null())
                    .col(ColumnDef::new(Project::LastUpdated).string())
                    .col(ColumnDef::new(Project::Nodes).string())
                    .col(ColumnDef::new(Project::Description).string())
                    .col(ColumnDef::new(Project::Tags).string())
                    .to_owned(),
            )
            .await?;

        // Create node table
        manager
            .create_table(
                Table::create()
                    .table(Node::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Node::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Node::ProjectId).string().not_null())
                    .col(ColumnDef::new(Node::Type).string().not_null())
                    .col(ColumnDef::new(Node::Display).string().not_null())
                    .col(ColumnDef::new(Node::Value).string().not_null())
                    .col(ColumnDef::new(Node::Updated).string().not_null())
                    .col(ColumnDef::new(Node::Notes).string())
                    .col(ColumnDef::new(Node::PosX).integer())
                    .col(ColumnDef::new(Node::PosY).integer())
                    .col(ColumnDef::new(Node::Attachments).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_node_project")
                            .from(Node::Table, Node::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create nodelink table
        manager
            .create_table(
                Table::create()
                    .table(NodeLink::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NodeLink::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NodeLink::Left).string().not_null())
                    .col(ColumnDef::new(NodeLink::Right).string().not_null())
                    .col(ColumnDef::new(NodeLink::ProjectId).string().not_null())
                    .col(ColumnDef::new(NodeLink::Linktype).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodelink_project")
                            .from(NodeLink::Table, NodeLink::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodelink_left")
                            .from(NodeLink::Table, NodeLink::Left)
                            .to(Node::Table, Node::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodelink_right")
                            .from(NodeLink::Table, NodeLink::Right)
                            .to(Node::Table, Node::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create attachment table
        manager
            .create_table(
                Table::create()
                    .table(Attachment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Attachment::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Attachment::NodeId).string().not_null())
                    .col(ColumnDef::new(Attachment::Filename).string().not_null())
                    .col(ColumnDef::new(Attachment::ContentType).string().not_null())
                    .col(ColumnDef::new(Attachment::Size).big_integer().not_null())
                    .col(ColumnDef::new(Attachment::Data).binary().not_null())
                    .col(ColumnDef::new(Attachment::Created).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_attachment_node")
                            .from(Attachment::Table, Attachment::NodeId)
                            .to(Node::Table, Node::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Attachment::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(NodeLink::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Node::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Project::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Project {
    Table,
    Id,
    Name,
    User,
    Creationdate,
    LastUpdated,
    Nodes,
    Description,
    Tags,
}

#[derive(DeriveIden)]
enum Node {
    Table,
    Id,
    ProjectId,
    Type,
    Display,
    Value,
    Updated,
    Notes,
    PosX,
    PosY,
    Attachments,
}

#[derive(DeriveIden)]
enum NodeLink {
    Table,
    Id,
    Left,
    Right,
    ProjectId,
    Linktype,
}

#[derive(DeriveIden)]
enum Attachment {
    Table,
    Id,
    NodeId,
    Filename,
    ContentType,
    Size,
    Data,
    Created,
}
