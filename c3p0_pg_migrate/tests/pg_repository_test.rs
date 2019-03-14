use c3p0_pg_migrate::migration::Migration;
use c3p0_pg_migrate::{PgMigrateBuilder, C3P0_MIGRATE_TABLE_DEFAULT};
use testcontainers::clients;

mod shared;

#[test]
fn should_create_the_c3p0_migrate_table_with_default_name() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0;

    let pg_migrate = PgMigrateBuilder::new().with_migrations(vec![]).build();

    pg_migrate.migrate(&conn).unwrap();

    assert!(conn
        .execute(
            &format!("select * from {}", C3P0_MIGRATE_TABLE_DEFAULT),
            &[]
        )
        .is_ok());

    Ok(())
}

#[test]
fn should_create_the_c3p0_migrate_table_with_custom_name() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0;

    let custom_name = "c3p0_custom_name";

    let pg_migrate = PgMigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![])
        .build();

    pg_migrate.migrate(&conn).unwrap();

    assert!(conn
        .execute(&format!("select * from {}", custom_name), &[])
        .is_ok());

    Ok(())
}

#[test]
fn should_execute_migrations() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0;

    let custom_name = "c3p0_custom_name";

    let pg_migrate = PgMigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![
            Migration {
                id: "first".to_owned(),
                up: "create table FIRST_TABLE (id int)".to_owned(),
                down: "".to_owned(),
            },
            Migration {
                id: "second".to_owned(),
                up: "create table SECOND_TABLE (id int)".to_owned(),
                down: "".to_owned(),
            },
        ])
        .build();

    pg_migrate.migrate(&conn).unwrap();

    assert!(conn
        .execute(&format!("select * from {}", custom_name), &[])
        .is_ok());
    assert!(conn
        .execute(&format!("select * from FIRST_TABLE"), &[])
        .is_ok());
    assert!(conn
        .execute(&format!("select * from SECOND_TABLE"), &[])
        .is_ok());

    let status = pg_migrate.fetch_migrations_history(&conn).unwrap();
    assert_eq!(2, status.len());
    assert_eq!("first", status.get(0).unwrap().data.migration_id);

    Ok(())
}

#[test]
fn should_not_execute_same_migrations_twice() -> Result<(), Box<std::error::Error>> {
    let docker = clients::Cli::default();
    let postgres_node = shared::new_connection(&docker);
    let conn = postgres_node.0;

    let custom_name = "c3p0_custom_name";

    let pg_migrate = PgMigrateBuilder::new()
        .with_table_name(custom_name)
        .with_migrations(vec![Migration {
            id: "first".to_owned(),
            up: "create table FIRST_TABLE (id int)".to_owned(),
            down: "".to_owned(),
        }])
        .build();

    pg_migrate.migrate(&conn).unwrap();
    pg_migrate.migrate(&conn).unwrap();

    assert!(conn
        .execute(&format!("select * from {}", custom_name), &[])
        .is_ok());
    assert!(conn
        .execute(&format!("select * from FIRST_TABLE"), &[])
        .is_ok());

    let status = pg_migrate.fetch_migrations_history(&conn).unwrap();
    assert_eq!(1, status.len());
    assert_eq!("first", status.get(0).unwrap().data.migration_id);

    Ok(())
}
