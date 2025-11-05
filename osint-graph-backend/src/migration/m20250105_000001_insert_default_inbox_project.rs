use sea_orm_migration::prelude::*;
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Insert default "Inbox" project with all-zeroes UUID
        let insert = Query::insert()
            .into_table(Project::Table)
            .columns([
                Project::Id,
                Project::Name,
                Project::User,
                Project::Creationdate,
                Project::Tags,
            ])
            .values_panic([
                Uuid::nil().into(),
                "Inbox".into(),
                Uuid::nil().into(),
                chrono::Utc::now().to_rfc3339().into(),
                "[]".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Delete the default Inbox project
        let delete = Query::delete()
            .from_table(Project::Table)
            .and_where(Expr::col(Project::Id).eq(Uuid::nil().to_string()))
            .to_owned();

        manager.exec_stmt(delete).await?;

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
    Tags,
}
