pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20241006_201324_create_count_summary_table;
mod m20250205_072723_add_status_column_to_count_summary;
mod m20250807_081018_rename_count_summary_to_status_history;
mod m20250807_100000_add_database_constraints;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20241006_201324_create_count_summary_table::Migration),
            Box::new(m20250205_072723_add_status_column_to_count_summary::Migration),
            Box::new(m20250807_081018_rename_count_summary_to_status_history::Migration),
            Box::new(m20250807_100000_add_database_constraints::Migration),
        ]
    }
}
