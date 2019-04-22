use crate::client::ToSql;
use crate::error::C3p0Error;

pub trait C3p0 {
    type Connection: Connection;

    fn connection(&self) -> Result<Self::Connection, C3p0Error>;

    fn transaction<T, F: Fn(&Connection) -> Result<T, C3p0Error>>(
        &self,
        tx: F,
    ) -> Result<T, C3p0Error>;
}

pub trait Connection {

    fn execute(&self, sql: &str, params: &[&ToSql]) -> Result<u64, C3p0Error>;

    fn batch_execute(&self, sql: &str) -> Result<(), C3p0Error>;

    //fetch_one
    //fetch_one_option
    //fetch_all

    //count_all_from_table

    //lock_table
}
