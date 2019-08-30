#![cfg(feature = "sqlite")]

use c3p0_sqlite::sqlite::r2d2::{Pool, SqliteConnectionManager};
use c3p0_sqlite::sqlite::*;
use c3p0_sqlite::*;
use testcontainers::*;

mod tests;

pub fn new_connection(_docker: &clients::Cli) -> (C3p0PoolSqlite, String) {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder().build(manager).unwrap();

    let pool = C3p0PoolSqlite::new(pool);

    (pool, "".to_owned())
}
