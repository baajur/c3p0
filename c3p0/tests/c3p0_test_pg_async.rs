#![cfg(feature = "pg_async")]

use c3p0::pg_async::bb8::{Pool, PostgresConnectionManager};
use c3p0::pg_async::*;
use c3p0::*;
use lazy_static::lazy_static;
use maybe_single_async::nio::MaybeSingleAsync;
use testcontainers::*;

pub use c3p0::pg_async::driver::row::Row;
pub use c3p0::pg_async::driver::tls::NoTls;
use futures::FutureExt;
use maybe_single_async::nio::Data;

pub type C3p0Impl = PgC3p0Pool;

//mod tests;
mod tests_json_async;

pub mod utils;

pub type MaybeType = (
    C3p0Impl,
    Container<'static, clients::Cli, images::generic::GenericImage>,
);

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingleAsync<MaybeType> =
        MaybeSingleAsync::new(|| init().boxed());
}

async fn init() -> MaybeType {
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
        NoTls,
    );

    let pool = Pool::builder()
        .min_idle(Some(10))
        .build(manager)
        .await
        .unwrap();
    let pool = PgC3p0Pool::new(pool);

    (pool, node)
}

pub async fn data(serial: bool) -> Data<'static, MaybeType> {
    SINGLETON.data(serial).await
}
