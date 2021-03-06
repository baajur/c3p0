use async_trait::async_trait;
use c3p0_common::*;
use futures::Future;

use crate::common::executor::batch_execute;
use crate::error::into_c3p0_error;
use crate::mysql::Db;
use sqlx::{Pool, Transaction};

#[derive(Clone)]
pub struct SqlxMySqlC3p0Pool {
    pool: Pool<Db>,
}

impl SqlxMySqlC3p0Pool {
    pub fn new(pool: Pool<Db>) -> Self {
        SqlxMySqlC3p0Pool { pool }
    }
}

impl Into<SqlxMySqlC3p0Pool> for Pool<Db> {
    fn into(self) -> SqlxMySqlC3p0Pool {
        SqlxMySqlC3p0Pool::new(self)
    }
}

#[async_trait]
impl C3p0Pool for SqlxMySqlC3p0Pool {
    type Conn = SqlxMySqlConnection;

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
            SqlxMySqlConnection::Tx(unsafe { ::std::mem::transmute(&mut native_transaction) });

        let result = { (tx)(transaction).await? };

        native_transaction.commit().await.map_err(into_c3p0_error)?;

        Ok(result)
    }
}

pub enum SqlxMySqlConnection {
    Tx(&'static mut Transaction<'static, Db>),
}

impl SqlxMySqlConnection {
    pub fn get_conn(&mut self) -> &mut Transaction<'static, Db> {
        match self {
            SqlxMySqlConnection::Tx(tx) => tx,
        }
    }
}

#[async_trait]
impl SqlConnection for SqlxMySqlConnection {
    async fn batch_execute(&mut self, sql: &str) -> Result<(), C3p0Error> {
        batch_execute(sql, self.get_conn()).await
    }
}
