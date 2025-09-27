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
                    .table(CountSummary::Table)
                    .if_not_exists()
                    .col(pk_auto(CountSummary::Id))
                    .col(date_time(CountSummary::StartDate))
                    .col(date_time(CountSummary::EndDate))
                    .col(decimal(CountSummary::Count))
                    .col(
                        timestamp(CountSummary::CreatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(
                        timestamp(CountSummary::UpdatedAt)
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // todo!();

        manager
            .drop_table(Table::drop().table(CountSummary::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CountSummary {
    Table,
    Id,
    StartDate,
    EndDate,
    Count,
    CreatedAt,
    UpdatedAt,
}
