use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Subject)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::DisplayName).string())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create pkce_states table
        manager
            .create_table(
                Table::create()
                    .table(PkceStates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PkceStates::State)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PkceStates::CodeVerifier).string().not_null())
                    .col(ColumnDef::new(PkceStates::Nonce).string().not_null())
                    .col(
                        ColumnDef::new(PkceStates::CodeChallenge)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PkceStates::RedirectUri).string().not_null())
                    .col(ColumnDef::new(PkceStates::ExpiresAt).timestamp().not_null())
                    .col(
                        ColumnDef::new(PkceStates::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on pkce_states.expires_at for cleanup
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-pkce-state-expires-at")
                    .table(PkceStates::Table)
                    .col(PkceStates::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PkceStates::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Subject,
    Email,
    DisplayName,
    CreatedAt,
    UpdatedAt,
}
#[derive(DeriveIden)]
enum PkceStates {
    Table,
    State,
    CodeVerifier,
    Nonce,
    CodeChallenge,
    RedirectUri,
    ExpiresAt,
    CreatedAt,
}
