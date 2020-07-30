use async_trait::async_trait;
use c3p0_common::*;
use futures::Future;

use crate::into_c3p0_error;
use sqlx::{Any, AnyPool, Transaction};

#[derive(Clone)]
pub struct SqlxC3p0PoolAsync {
    pool: AnyPool,
}

impl SqlxC3p0PoolAsync {
    pub fn new(pool: AnyPool) -> Self {
        SqlxC3p0PoolAsync { pool }
    }
}

impl Into<SqlxC3p0PoolAsync> for AnyPool {
    fn into(self) -> SqlxC3p0PoolAsync {
        SqlxC3p0PoolAsync::new(self)
    }
}

#[async_trait]
impl C3p0PoolAsync for SqlxC3p0PoolAsync {
    type Conn = SqlxConnectionAsync;

    async fn transaction<
        T: Send,
        E: Send + From<C3p0Error>,
        F: Send + FnOnce(Self::Conn) -> Fut,
        Fut: Send + Future<Output = Result<T, E>>,
    >(
        &self,
        tx: F,
    ) -> Result<T, E> {
        let mut native_transaction = self.pool.begin().await.map_err(into_c3p0_error)?;

        // ToDo: To avoid this unsafe we need GAT
        let transaction =
            SqlxConnectionAsync::Tx(unsafe { ::std::mem::transmute(&mut native_transaction) });

        let result = { (tx)(transaction).await? };

        native_transaction.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub enum SqlxConnectionAsync {
    Tx(&'static mut Transaction<'static, Any>),
}

impl SqlxConnectionAsync {
    pub fn get_conn(&mut self) -> &mut Transaction<'static, Any> {
        match self {
            SqlxConnectionAsync::Tx(tx) => tx,
        }
    }
}

#[async_trait]
impl SqlConnectionAsync for SqlxConnectionAsync {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        let query = sqlx::query(sql);
        query
            .execute(self.get_conn())
            .await
            .map_err(into_c3p0_error)
            .map(|_| ())
    }
}
