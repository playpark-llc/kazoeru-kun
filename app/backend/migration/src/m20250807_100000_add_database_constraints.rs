use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add unique constraint for session_counter (session_id is already unique)
        // Add index for better query performance
        manager
            .create_index(
                Index::create()
                    .name("idx_session_counter_created_at")
                    .table(SessionCounter::Table)
                    .col(SessionCounter::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_session_counter_updated_at")
                    .table(SessionCounter::Table)
                    .col(SessionCounter::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        // Add unique constraint for status_history to prevent duplicate entries for the same period
        manager
            .create_index(
                Index::create()
                    .name("idx_status_history_unique_period")
                    .table(StatusHistory::Table)
                    .col(StatusHistory::StartDate)
                    .col(StatusHistory::EndDate)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add indexes for status_history queries
        manager
            .create_index(
                Index::create()
                    .name("idx_status_history_dates")
                    .table(StatusHistory::Table)
                    .col(StatusHistory::StartDate)
                    .col(StatusHistory::EndDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_status_history_created_at")
                    .table(StatusHistory::Table)
                    .col(StatusHistory::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Add check constraint to ensure end_date >= start_date
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE `status_history` ADD CONSTRAINT `chk_status_history_dates` CHECK (`end_date` >= `start_date`)"
            )
            .await?;

        // Add check constraint to ensure count >= 0
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE `session_counter` ADD CONSTRAINT `chk_session_counter_count` CHECK (`count` >= 0)"
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE `status_history` ADD CONSTRAINT `chk_status_history_count` CHECK (`count` >= 0)"
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop check constraints
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE `status_history` DROP CONSTRAINT IF EXISTS `chk_status_history_dates`"
            )
            .await?;
        
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE `session_counter` DROP CONSTRAINT IF EXISTS `chk_session_counter_count`"
            )
            .await?;
        
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE `status_history` DROP CONSTRAINT IF EXISTS `chk_status_history_count`"
            )
            .await?;

        // Drop indexes
        manager
            .drop_index(
                Index::drop()
                    .name("idx_session_counter_created_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_session_counter_updated_at")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_status_history_unique_period")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_status_history_dates")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_status_history_created_at")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum SessionCounter {
    Table,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum StatusHistory {
    Table,
    StartDate,
    EndDate,
    CreatedAt,
}