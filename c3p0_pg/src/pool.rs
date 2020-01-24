use crate::error::into_c3p0_error;
use crate::pg::driver::row::Row;
use crate::pg::driver::types::{FromSqlOwned, ToSql};
use crate::pg::r2d2::{Pool, PooledConnection};

use crate::r2d2::PostgresConnectionManager;
use c3p0_common::*;
use postgres::Transaction;

#[derive(Clone)]
pub struct PgC3p0Pool {
    pool: Pool<PostgresConnectionManager>,
}

impl PgC3p0Pool {
    pub fn new(pool: Pool<PostgresConnectionManager>) -> Self {
        PgC3p0Pool { pool }
    }
}

impl Into<PgC3p0Pool> for Pool<PostgresConnectionManager> {
    fn into(self) -> PgC3p0Pool {
        PgC3p0Pool::new(self)
    }
}

impl C3p0Pool for PgC3p0Pool {
    type CONN = PgConnection;

    fn connection(&self) -> Result<PgConnection, C3p0Error> {
        self.pool
            .get()
            .map_err(|err| C3p0Error::PoolError {
                cause: format!("{}", err),
            })
            .map(PgConnection::Conn)
    }

    fn transaction<T, E: From<C3p0Error>, F: FnOnce(&mut PgConnection) -> Result<T, E>>(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let conn = self.pool.get().map_err(|err| C3p0Error::PoolError {
            cause: format!("{}", err),
        })?;

        let transaction = new_simple_mut(conn)?;

        let (result, executor) = {
            let mut sql_executor = PgConnection::Tx(transaction);
            let result = (tx)(&mut sql_executor)?;
            (result, sql_executor)
        };

        match executor {
            PgConnection::Tx(mut tx) => {
                tx.rent_mut(|tref| {
                    let tref_some = tref.take();
                    tref_some.unwrap().commit()?;
                    Ok(())
                })
                .map_err(into_c3p0_error)?;
            }
            _ => panic!("It should have been a transaction"),
        };

        Ok(result)
    }
}

fn new_simple_mut(
    conn: PooledConnection<PostgresConnectionManager>,
) -> Result<rentals::SimpleMut, C3p0Error> {
    rentals::SimpleMut::try_new_or_drop(Box::new(conn), |c| {
        let tx = c.transaction().map_err(into_c3p0_error)?;
        Ok(Some(tx))
    })
}

rental! {
    mod rentals {
        use super::*;

        #[rental_mut]
        pub struct SimpleMut {
            conn: Box<PooledConnection<PostgresConnectionManager>>,
            tx: Option<Transaction<'conn>>,
        }
    }
}

pub enum PgConnection {
    Conn(PooledConnection<PostgresConnectionManager>),
    Tx(rentals::SimpleMut),
}

impl SqlConnection for PgConnection {
    fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            PgConnection::Conn(conn) => conn.batch_execute(sql).map_err(into_c3p0_error),
            PgConnection::Tx(tx) => tx.rent_mut(|tref| {
                let conn = tref.as_mut().unwrap();
                conn.batch_execute(sql).map_err(into_c3p0_error)
            }),
        }
    }
}

impl PgConnection {
    pub fn execute(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, C3p0Error> {
        match self {
            PgConnection::Conn(conn) => conn.execute(sql, params).map_err(into_c3p0_error),
            PgConnection::Tx(tx) => tx.rent_mut(|tref| {
                let conn = tref.as_mut().unwrap();
                conn.execute(sql, params).map_err(into_c3p0_error)
            }),
        }
    }

    pub fn fetch_one_value<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<T, C3p0Error> {
        self.fetch_one(sql, params, to_value_mapper)
    }

    pub fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        self.fetch_one_optional(sql, params, mapper)
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    pub fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        match self {
            PgConnection::Conn(conn) => {
                let stmt = conn.prepare(sql).map_err(into_c3p0_error)?;
                conn.query(&stmt, params)
                    .map_err(into_c3p0_error)?
                    .iter()
                    .next()
                    .map(|row| mapper(&row))
                    .transpose()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }
            PgConnection::Tx(tx) => tx.rent_mut(|tref| {
                let conn = tref.as_mut().unwrap();
                let stmt = conn.prepare(sql).map_err(into_c3p0_error)?;
                conn.query(&stmt, params)
                    .map_err(into_c3p0_error)?
                    .iter()
                    .next()
                    .map(|row| mapper(&row))
                    .transpose()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }),
        }
    }

    pub fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            PgConnection::Conn(conn) => {
                let stmt = conn.prepare(sql).map_err(into_c3p0_error)?;
                conn.query(&stmt, params)
                    .map_err(into_c3p0_error)?
                    .iter()
                    .map(|row| mapper(&row))
                    .collect::<Result<Vec<T>, Box<dyn std::error::Error>>>()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }
            PgConnection::Tx(tx) => tx.rent_mut(|tref| {
                let conn = tref.as_mut().unwrap();
                let stmt = conn.prepare(sql).map_err(into_c3p0_error)?;
                conn.query(&stmt, params)
                    .map_err(into_c3p0_error)?
                    .iter()
                    .map(|row| mapper(&row))
                    .collect::<Result<Vec<T>, Box<dyn std::error::Error>>>()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }),
        }
    }

    pub fn fetch_all_values<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper)
    }
}

fn to_value_mapper<T: FromSqlOwned>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    Ok(row.try_get(0).map_err(|_| C3p0Error::ResultNotFoundError)?)
}
