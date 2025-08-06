use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // count_summaryテーブルをstatus_historyにリネーム
        manager
            .rename_table(
                Table::rename()
                    .table(CountSummary::Table, StatusHistory::Table)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // status_historyテーブルをcount_summaryに戻す
        manager
            .rename_table(
                Table::rename()
                    .table(StatusHistory::Table, CountSummary::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum CountSummary {
    Table,
}

#[derive(DeriveIden)]
enum StatusHistory {
    Table,
}