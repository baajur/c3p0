#![cfg(feature = "mysql")]

use c3p0_json::mysql::mysql::{Opts, OptsBuilder};
use c3p0_json::mysql::r2d2::{Pool, MysqlConnectionManager};
use testcontainers::*;

pub use c3p0_json::mysql::C3p0Mysql as C3p0Impl;
pub use c3p0_json::mysql::C3p0MysqlBuilder as C3p0BuilderImpl;

pub fn new_connection(
    docker: &clients::Cli,
) -> (C3p0Impl, Container<clients::Cli, images::generic::GenericImage>) {
    let mysql_version = "5.7.25";
    let mysql_image = images::generic::GenericImage::new(format!("mysql:{}", mysql_version))
        .with_wait_for(images::generic::WaitFor::message_on_stderr(
            format!("Version: '{}'  socket: '/var/run/mysqld/mysqld.sock'  port: 3306  MySQL Community Server (GPL)", mysql_version),
        ))
        .with_env_var("MYSQL_DATABASE", "mysql")
        .with_env_var("MYSQL_USER", "mysql")
        .with_env_var("MYSQL_PASSWORD", "mysql")
        .with_env_var("MYSQL_ROOT_PASSWORD", "mysql");
    let node = docker.run(mysql_image);

    let db_url = format!(
        "mysql://mysql:mysql@127.0.0.1:{}/mysql",
        node.get_host_port(3306).unwrap()
    );

    let opts = Opts::from_url(&db_url).unwrap();
    let builder = OptsBuilder::from_opts(opts);

    let manager = MysqlConnectionManager::new(builder);

    let pool = Pool::builder()
        .min_idle(Some(10))
        .build(manager)
        .unwrap();

    let pool = C3p0BuilderImpl::build(pool);

    (pool, node)
}
