#![cfg(feature = "sqlite_blocking")]

pub use c3p0::sqlite::blocking::rusqlite::Row;
use c3p0::sqlite::blocking::r2d2::{Pool, SqliteConnectionManager};
use c3p0::sqlite::blocking::*;
use c3p0::blocking::*;
use lazy_static::lazy_static;
use maybe_single::{Data, MaybeSingle};

pub type C3p0Impl = SqliteC3p0Pool;

mod tests_blocking;
mod tests_blocking_json;
mod utils;

lazy_static! {
    pub static ref SINGLETON: MaybeSingle<MaybeType> = MaybeSingle::new(|| init());
}

pub type MaybeType = (C3p0Impl, String);

fn init() -> MaybeType {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder().build(manager).unwrap();

    let pool = SqliteC3p0Pool::new(pool);

    (pool, "".to_owned())
}

pub fn data(serial: bool) -> Data<'static, MaybeType> {
    SINGLETON.data(serial)
}
