#![cfg(feature = "pg")]

use c3p0_json::*;
use lazy_static::lazy_static;
use maybe_single::MaybeSingle;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use serde_derive::{Deserialize, Serialize};
use testcontainers::*;

pub use c3p0_json::C3p0Pg as C3p0;
pub use c3p0_json::C3p0PgBuilder as C3p0Builder;
pub use c3p0_json::C3p0PgJson as C3p0Json;
pub use c3p0_json::C3p0PgJsonBuilder as C3p0JsonBuilder;

pub use postgres::rows::Row;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct TestData {
    pub first_name: String,
    pub last_name: String,
}

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<(
        C3p0Pg,
        Container<'static, clients::Cli, images::postgres::Postgres>
    )> = MaybeSingle::new(|| init());
}

fn init() -> (
    C3p0Pg,
    Container<'static, clients::Cli, images::postgres::Postgres>,
) {
    let node = DOCKER.run(images::postgres::Postgres::default());

    let manager = PostgresConnectionManager::new(
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port(5432).unwrap()
        ),
        TlsMode::None,
    )
    .unwrap();
    let pool = r2d2::Pool::builder()
        .min_idle(Some(10))
        .build(manager)
        .unwrap();

    let pool = C3p0PgBuilder::build(pool);

    (pool, node)
}
