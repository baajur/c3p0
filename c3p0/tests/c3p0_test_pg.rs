#![cfg(feature = "pg")]

use c3p0::pg::r2d2::{Pool, PostgresConnectionManager};
use c3p0::pg::*;
use c3p0::*;
use lazy_static::lazy_static;
use maybe_single::{Data, MaybeSingle};
use testcontainers::*;

pub use c3p0::pg::driver::row::Row;
pub use c3p0::pg::driver::tls::NoTls;

pub type C3p0Impl = PgC3p0Pool;

mod tests;
mod tests_json;
pub mod utils;

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<MaybeType> = MaybeSingle::new(|| init());
}

pub type MaybeType = (
    C3p0Impl,
    Container<'static, clients::Cli, images::generic::GenericImage>,
);

fn init() -> MaybeType {
    let node = DOCKER.run(
        images::generic::GenericImage::new("postgres:11-alpine")
            .with_wait_for(images::generic::WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_DB", "postgres")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres"),
    );

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

pub fn data(serial: bool) -> Data<'static, MaybeType> {
    SINGLETON.data(serial)
}
