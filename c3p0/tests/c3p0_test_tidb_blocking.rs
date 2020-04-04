#![cfg(feature = "mysql_blocking")]

pub use c3p0::mysql::blocking::mysql::{Opts, OptsBuilder, Row};
use c3p0::mysql::blocking::r2d2::{MysqlConnectionManager, Pool};
use c3p0::mysql::blocking::*;
use c3p0::blocking::*;
use lazy_static::lazy_static;
use maybe_single::{Data, MaybeSingle};
use testcontainers::*;

pub type C3p0Impl = MysqlC3p0Pool;

mod tests_blocking;
mod tests_blocking_json;
mod utils;

lazy_static! {
    static ref DOCKER: clients::Cli = clients::Cli::default();
    pub static ref SINGLETON: MaybeSingle<MaybeType> = MaybeSingle::new(|| init());
}

pub type MaybeType = (
    C3p0Impl,
    Container<'static, clients::Cli, images::generic::GenericImage>,
);

fn init() -> MaybeType {
    let mysql_version = "v3.0.3";
    let mysql_image = images::generic::GenericImage::new(format!("pingcap/tidb:{}", mysql_version))
        .with_wait_for(images::generic::WaitFor::message_on_stdout(
            r#"["server is running MySQL protocol"] [addr=0.0.0.0:4000]"#,
        ));
    let node = DOCKER.run(mysql_image);

    let db_url = format!(
        "mysql://root@127.0.0.1:{}/mysql",
        node.get_host_port(4000).unwrap()
    );

    let opts = Opts::from_url(&db_url).unwrap();
    let builder = OptsBuilder::from_opts(opts);

    let manager = MysqlConnectionManager::new(builder);

    let pool = Pool::builder().min_idle(Some(10)).build(manager).unwrap();

    let pool = MysqlC3p0Pool::new(pool);

    (pool, node)
}

pub fn data(serial: bool) -> Data<'static, MaybeType> {
    SINGLETON.data(serial)
}
