#![cfg(feature = "pg_blocking")]

pub use c3p0::blocking::*;
use c3p0::pg::blocking::postgres::NoTls;
use c3p0::pg::blocking::r2d2::{Pool, PostgresConnectionManager};
pub use c3p0::pg::blocking::*;
use testcontainers::*;

mod tests_blocking;
pub mod utils;

pub fn new_connection(
    docker: &clients::Cli,
) -> (
    PgC3p0Pool,
    Container<clients::Cli, images::postgres::Postgres>,
) {
    let node = docker.run(images::postgres::Postgres::default());
    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        )
        .parse()
        .unwrap(),
        Box::new(move |config| config.connect(NoTls)),
    );

    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();

    let pool = PgC3p0Pool::new(pool);

    (pool, node)
}

pub mod db_specific {
    use super::*;

    pub fn db_type() -> utils::DbType {
        utils::DbType::Pg
    }
}
