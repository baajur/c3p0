use crate::nio::*;
use crate::nio::deadpool::postgres::Pool;
use crate::nio::tokio_postgres::Transaction;
use crate::nio::tokio_postgres::row::Row;
use crate::nio::tokio_postgres::types::{FromSqlOwned, ToSql};

use async_trait::async_trait;
use c3p0_common::*;
use futures::Future;

pub enum PgC3p0ConnectionManager{
    DeadPool
}

impl PgC3p0ConnectionManager {

}

#[derive(Clone)]
pub struct PgC3p0PoolAsync {
    pool: Pool,
}

impl PgC3p0PoolAsync {
    pub fn new(pool: Pool) -> Self {
        PgC3p0PoolAsync { pool }
    }
}

impl Into<PgC3p0PoolAsync> for Pool {
    fn into(self) -> PgC3p0PoolAsync {
        PgC3p0PoolAsync::new(self)
    }
}

#[async_trait]
impl C3p0PoolAsync for PgC3p0PoolAsync {
    type CONN = PgConnectionAsync;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(Self::CONN) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut conn = self.pool.get().await.map_err(deadpool_into_c3p0_error)?;

        let native_transaction = conn.transaction().await.map_err(into_c3p0_error)?;

        // ToDo: To avoid this unsafe we need GAT
            let transaction =
                PgConnectionAsync::Tx(unsafe { ::std::mem::transmute(&native_transaction) });

        let result = {
            (tx)(transaction).await?
        };

        native_transaction.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub enum PgConnectionAsync {
    Tx(&'static Transaction<'static>),
}

#[async_trait]
impl SqlConnectionAsync for PgConnectionAsync {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        match self {
            PgConnectionAsync::Tx(tx) => tx.batch_execute(sql).await.map_err(into_c3p0_error),
        }
    }
}

impl PgConnectionAsync {
    pub async fn execute(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, C3p0Error> {
        match self {
            PgConnectionAsync::Tx(tx) => tx.execute(sql, params).await.map_err(into_c3p0_error),
        }
    }

    pub async fn fetch_one_value<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<T, C3p0Error> {
        self.fetch_one(sql, params, to_value_mapper).await
    }

    pub async fn fetch_one<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<T, C3p0Error> {
        self.fetch_one_optional(sql, params, mapper)
            .await
            .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    pub async fn fetch_one_optional<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Option<T>, C3p0Error> {
        match self {
            PgConnectionAsync::Tx(tx) => {
                let stmt = tx.prepare(sql).await.map_err(into_c3p0_error)?;
                tx.query(&stmt, params)
                    .await
                    .map_err(into_c3p0_error)?
                    .iter()
                    .next()
                    .map(|row| mapper(&row))
                    .transpose()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }
        }
    }

    pub async fn fetch_all<T, F: Fn(&Row) -> Result<T, Box<dyn std::error::Error>>>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
        mapper: F,
    ) -> Result<Vec<T>, C3p0Error> {
        match self {
            PgConnectionAsync::Tx(tx) => {
                let stmt = tx.prepare(sql).await.map_err(into_c3p0_error)?;
                tx.query(&stmt, params)
                    .await
                    .map_err(into_c3p0_error)?
                    .iter()
                    .map(|row| mapper(&row))
                    .collect::<Result<Vec<T>, Box<dyn std::error::Error>>>()
                    .map_err(|err| C3p0Error::RowMapperError {
                        cause: format!("{}", err),
                    })
            }
        }
    }

    pub async fn fetch_all_values<T: FromSqlOwned>(
        &mut self,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<T>, C3p0Error> {
        self.fetch_all(sql, params, to_value_mapper).await
    }
}

fn to_value_mapper<T: FromSqlOwned>(row: &Row) -> Result<T, Box<dyn std::error::Error>> {
    Ok(row.try_get(0).map_err(|_| C3p0Error::ResultNotFoundError)?)
}
