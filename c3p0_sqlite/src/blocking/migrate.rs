use crate::blocking::*;
use c3p0_common::blocking::*;

pub trait SqliteC3p0MigrateBuilder {
    fn build(self) -> C3p0Migrate<SqliteConnection, SqliteC3p0Pool, SqliteMigrator>;
}

impl SqliteC3p0MigrateBuilder for C3p0MigrateBuilder<SqliteC3p0Pool> {
    fn build(self) -> C3p0Migrate<SqliteConnection, SqliteC3p0Pool, SqliteMigrator> {
        C3p0Migrate::new(
            self.table,
            self.schema,
            self.migrations,
            self.c3p0,
            SqliteMigrator {},
        )
    }
}

#[derive(Clone)]
pub struct SqliteMigrator {}

impl Migrator for SqliteMigrator {
    type Conn = SqliteConnection;
    type C3P0 = SqliteC3p0Pool;
    type C3P0Json = SqliteC3p0Json<MigrationData, DefaultJsonCodec>;

    fn build_cp30_json(
        &self,
        table: String,
        schema: Option<String>,
    ) -> SqliteC3p0Json<MigrationData, DefaultJsonCodec> {
        C3p0JsonBuilder::<SqliteC3p0Pool>::new(table)
            .with_schema_name(schema)
            .build()
    }

    fn lock_table(
        &self,
        _c3p0_json: &SqliteC3p0Json<MigrationData, DefaultJsonCodec>,
        _conn: &mut SqliteConnection,
    ) -> Result<(), C3p0Error> {
        Ok(())
    }

    fn lock_first_migration_row(
        &self,
        c3p0_json: &SqliteC3p0Json<MigrationData, DefaultJsonCodec>,
        conn: &mut SqliteConnection,
    ) -> Result<(), C3p0Error> {
        let lock_sql = format!(
            r#"select * from {} where JSON_EXTRACT({}, "$.migration_id") = ?"#,
            c3p0_json.queries().qualified_table_name,
            c3p0_json.queries().data_field_name
        );
        conn.fetch_one(&lock_sql, &[&C3P0_INIT_MIGRATION_ID], |_| Ok(()))
    }
}
