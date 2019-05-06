#[cfg(feature = "mysql")]
mod mysql;

#[cfg(feature = "mysql")]
pub type C3p0Builder = mysql::pool::C3p0MySqlBuilder;
#[cfg(feature = "mysql")]
pub type JsonManager<'a, DATA, CODEC> = mysql::json::MySqlJsonManager<'a, DATA, CODEC>;
#[cfg(feature = "mysql")]
pub type JsonManagerBuilder<DATA, CODEC> = mysql::json::MySqlJsonManagerBuilder<DATA, CODEC>;
#[cfg(feature = "mysql")]
pub type ToSql = mysql::pool::ToSql;
#[cfg(feature = "mysql")]
pub type Row = mysql::pool::Row;
#[cfg(feature = "mysql")]
pub type Connection<'a> = mysql::pool::Connection<'a>;
#[cfg(feature = "mysql")]
pub type C3p0 = mysql::pool::C3p0MySql;

#[cfg(feature = "pg")]
mod pg;

#[cfg(feature = "pg")]
pub type C3p0Builder = pg::pool::C3p0PgBuilder;
#[cfg(feature = "pg")]
pub type JsonManager<'a, DATA, CODEC> = pg::json::PostgresJsonManager<'a, DATA, CODEC>;
#[cfg(feature = "pg")]
pub type JsonManagerBuilder<DATA, CODEC> = pg::json::PostgresJsonManagerBuilder<DATA, CODEC>;
#[cfg(feature = "pg")]
pub type ToSql = pg::pool::ToSql;
#[cfg(feature = "pg")]
pub type Row<'a> = pg::pool::Row<'a>;
#[cfg(feature = "pg")]
pub type Connection = pg::pool::Connection;
#[cfg(feature = "pg")]
pub type C3p0 = pg::pool::C3p0Pg;

pub const NO_PARAMS: &[&ToSql] = &[];
