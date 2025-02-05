use sea_orm::Statement;
use sea_orm_migration::prelude::*;
use sea_query::Alias;

#[derive(Iden)]
enum StatusEnumType {
    #[iden = "status_enum"]
    Type,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // マイグレーション適用時の処理
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // count_summaryテーブルに status カラムを追加
        manager
            .alter_table(
                Table::alter()
                    .table(CountSummary::Table)
                    .add_column(
                        ColumnDef::new(CountSummary::Status)
                            .enumeration(
                                StatusEnumType::Type,
                                vec![
                                    Alias::new("DIRTY"),
                                    Alias::new("UNCLEAN"),
                                    Alias::new("CLEAN"),
                                    Alias::new("IMMACULATE"),
                                    Alias::new("SPOTLESS"),
                                ],
                            )
                            .not_null(), // 必要に応じて default 値も設定可能
                                         // .default("CLEAN")
                    )
                    .to_owned(),
            )
            .await
    }

    // マイグレーションロールバック時の処理
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // count_summaryテーブルから status カラムを削除
        manager
            .alter_table(
                Table::alter()
                    .table(CountSummary::Table)
                    .drop_column(CountSummary::Status)
                    .to_owned(),
            )
            .await?;

        // ENUM型 status_enum の削除
        manager
            .get_connection()
            .execute(Statement::from_string(
                manager.get_database_backend(),
                "DROP TYPE status_enum".to_owned(),
            ))
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum CountSummary {
    Table,
    Status,
}

