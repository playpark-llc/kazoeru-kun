use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // todo!();

        manager
            .create_table(
                Table::create()
                    .table(SessionCounter::Table)
                    .if_not_exists()
                    .col(pk_auto(SessionCounter::Id))
                    .col(string(SessionCounter::SessionId).unique_key())
                    .col(integer(SessionCounter::Count))
                    .col(
                        timestamp(SessionCounter::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(SessionCounter::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .extra("on update CURRENT_TIMESTAMP"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // todo!();

        manager
            .drop_table(Table::drop().table(SessionCounter::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SessionCounter {
    Table,
    Id,
    SessionId,
    Count,
    CreatedAt,
    UpdatedAt,
}
