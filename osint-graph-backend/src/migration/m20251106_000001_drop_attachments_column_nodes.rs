use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                TableAlterStatement::new()
                    .table(Node::Table)
                    .drop_column(Node::Attachments)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Re-add the attachments column to the nodes table
        let add_attachments = TableAlterStatement::new()
            .table(Node::Table)
            .add_column(ColumnDef::new(Node::Attachments).string().null())
            .to_owned();

        manager.exec_stmt(add_attachments).await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Node {
    Table,
    Attachments,
}
