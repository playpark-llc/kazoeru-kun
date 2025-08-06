use sea_orm::{DatabaseBackend, Statement};
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
        // PostgreSQLの場合は先にENUM型を作成する必要がある
        // MySQLの場合は列定義内で暗黙的に作成される
        let db_backend = manager.get_database_backend();
        if db_backend == DatabaseBackend::Postgres {
            manager
                .get_connection()
                .execute(Statement::from_string(
                    db_backend,
                    "CREATE TYPE status_enum AS ENUM ('DIRTY', 'UNCLEAN', 'CLEAN', 'IMMACULATE', 'SPOTLESS')".to_owned(),
                ))
                .await?;
        }

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
                            .not_null()
                            .default("CLEAN") // 既存データのためデフォルト値を設定
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

        // PostgreSQLの場合のみENUM型 status_enum を削除
        // MySQLではENUM型は列と共に自動的に削除される
        let db_backend = manager.get_database_backend();
        if db_backend == DatabaseBackend::Postgres {
            manager
                .get_connection()
                .execute(Statement::from_string(
                    db_backend,
                    "DROP TYPE IF EXISTS status_enum".to_owned(),
                ))
                .await?;
        }

        Ok(())
    }
}

#[derive(Iden)]
pub enum CountSummary {
    Table,
    Status,
}

