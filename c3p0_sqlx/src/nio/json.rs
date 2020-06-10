use async_trait::async_trait;
use c3p0_common::json::Queries;
use c3p0_common::*;
use crate::{build_pg_queries, SqlxC3p0PoolAsync, SqlxConnectionAsync};
use sqlx::{Connect, Row};

pub trait SqlxC3p0JsonAsyncBuilder<C: Connect + Send + Sync> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxC3p0JsonAsync<DATA, DefaultJsonCodec, C>;
    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxC3p0JsonAsync<DATA, CODEC, C>;
}

impl <C: Connect + Send + Sync> SqlxC3p0JsonAsyncBuilder<C> for C3p0JsonBuilder<SqlxC3p0PoolAsync<C>> {
    fn build<DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync>(
        self,
    ) -> SqlxC3p0JsonAsync<DATA, DefaultJsonCodec, C> {
        self.build_with_codec(DefaultJsonCodec {})
    }

    fn build_with_codec<
        DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
        CODEC: JsonCodec<DATA>,
    >(
        self,
        codec: CODEC,
    ) -> SqlxC3p0JsonAsync<DATA, CODEC, C> {
        SqlxC3p0JsonAsync {
            phantom_data: std::marker::PhantomData,
            phantom_c: std::marker::PhantomData,
            codec,
            queries: build_pg_queries(self),
        }
    }
}

#[derive(Clone)]
pub struct SqlxC3p0JsonAsync<DATA, CODEC: JsonCodec<DATA>, C: Connect + Send + Sync>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    phantom_data: std::marker::PhantomData<DATA>,
    phantom_c: std::marker::PhantomData<C>,

    codec: CODEC,
    queries: Queries,
}

impl<DATA, CODEC: JsonCodec<DATA>, C: Connect + Send + Sync> SqlxC3p0JsonAsync<DATA, CODEC, C>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    pub fn queries(&self) -> &Queries {
        &self.queries
    }
/*
    #[inline]
    pub fn to_model(&self, row: &Row) -> Result<Model<DATA>, Box<dyn std::error::Error>> {
        to_model(&self.codec, row, 0, 1, 2)
    }
*/
    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_optional_with_sql(
        &self,
        conn: &mut SqlxConnectionAsync<C>,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        unimplemented!()
    }
/*
    /// Allows the execution of a custom sql query and returns the first entry in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_one_with_sql(
        &self,
        conn: &mut SqlxConnectionAsync,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Model<DATA>, C3p0Error> {
        conn.fetch_one(sql, params, |row| self.to_model(row)).await
    }

    /// Allows the execution of a custom sql query and returns all the entries in the result set.
    /// For this to work, the sql query:
    /// - must be a SELECT
    /// - must declare the ID, VERSION and DATA fields in this exact order
    pub async fn fetch_all_with_sql(
        &self,
        conn: &mut SqlxConnectionAsync,
        sql: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        conn.fetch_all(sql, params, |row| self.to_model(row)).await
    }

 */
}


#[async_trait]
impl<DATA, CODEC: JsonCodec<DATA>, C: Connect + Send + Sync + Clone> C3p0JsonAsync<DATA, CODEC> for SqlxC3p0JsonAsync<DATA, CODEC, C>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Send + Sync,
{
    type Conn = SqlxConnectionAsync<C>;

    fn codec(&self) -> &CODEC {
        &self.codec
    }

    async fn create_table_if_not_exists(
        &self,
        conn: &mut Self::Conn,
    ) -> Result<(), C3p0Error> {
        unimplemented!()
        // conn.execute(&self.queries.create_table_sql_query, &[])
        //     .await?;
        // Ok(())
    }

    async fn drop_table_if_exists(
        &self,
        conn: &mut Self::Conn,
        cascade: bool,
    ) -> Result<(), C3p0Error> {
        unimplemented!()
        // let query = if cascade {
        //     &self.queries.drop_table_sql_query_cascade
        // } else {
        //     &self.queries.drop_table_sql_query
        // };
        // conn.execute(query, &[]).await?;
        // Ok(())
    }

    async fn count_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error> {
        unimplemented!()
        // conn.fetch_one_value(&self.queries.count_all_sql_query, &[])
        //     .await
        //     .map(|val: i64| val as u64)
    }

    async fn exists_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        unimplemented!()
        // conn.fetch_one_value(&self.queries.exists_by_id_sql_query, &[&id.into()])
        //     .await
    }

    async fn fetch_all(&self, conn: &mut Self::Conn) -> Result<Vec<Model<DATA>>, C3p0Error> {
        unimplemented!()
        // conn.fetch_all(&self.queries.find_all_sql_query, &[], |row| {
        //     self.to_model(row)
        // })
        // .await
    }

    async fn fetch_all_for_update(
        &self,
        conn: &mut Self::Conn,
        for_update: &ForUpdate,
    ) -> Result<Vec<Model<DATA>>, C3p0Error> {
        unimplemented!()
        // let sql = format!(
        //     "{}\n{}",
        //     &self.queries.find_all_sql_query,
        //     for_update.to_sql()
        // );
        // conn.fetch_all(&sql, &[], |row| self.to_model(row)).await
    }

    async fn fetch_one_optional_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        unimplemented!()
        // conn.fetch_one_optional(&self.queries.find_by_id_sql_query, &[&id.into()], |row| {
        //     self.to_model(row)
        // })
        // .await
    }

    async fn fetch_one_optional_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        unimplemented!()
        // let sql = format!(
        //     "{}\n{}",
        //     &self.queries.find_by_id_sql_query,
        //     for_update.to_sql()
        // );
        // conn.fetch_one_optional(&sql, &[&id.into()], |row| self.to_model(row))
        //     .await
    }

    async fn fetch_one_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
        // self.fetch_one_optional_by_id(conn, id)
        //     .await
        //     .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    async fn fetch_one_by_id_for_update<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
        for_update: &ForUpdate,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
        // self.fetch_one_optional_by_id_for_update(conn, id, for_update)
        //     .await
        //     .and_then(|result| result.ok_or_else(|| C3p0Error::ResultNotFoundError))
    }

    async fn delete(
        &self,
        conn: &mut Self::Conn,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
        // let result = conn
        //     .execute(&self.queries.delete_sql_query, &[&obj.id, &obj.version])
        //     .await?;
        //
        // if result == 0 {
        //     return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
        //                                                                 &self.queries.qualified_table_name, &obj.id, &obj.version
        //     )});
        // }
        //
        // Ok(obj)
    }

    async fn delete_all(&self, conn: &mut Self::Conn) -> Result<u64, C3p0Error> {
        unimplemented!()
        // conn.execute(&self.queries.delete_all_sql_query, &[]).await
    }

    async fn delete_by_id<'a, ID: Into<&'a IdType> + Send>(
        &'a self,
        conn: &mut Self::Conn,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        unimplemented!()
        // conn.execute(&self.queries.delete_by_id_sql_query, &[id.into()])
        //     .await
    }

    async fn save(
        &self,
        conn: &mut Self::Conn,
        obj: NewModel<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
        // let json_data = self.codec().to_value(&obj.data)?;
        // let id = conn
        //     .fetch_one_value(&self.queries.save_sql_query, &[&obj.version, &json_data])
        //     .await?;
        // Ok(Model {
        //     id,
        //     version: obj.version,
        //     data: obj.data,
        // })
    }

    async fn update(
        &self,
        conn: &mut Self::Conn,
        obj: Model<DATA>,
    ) -> Result<Model<DATA>, C3p0Error> {
        unimplemented!()
        // let json_data = self.codec().to_value(&obj.data)?;
        //
        // let updated_model = Model {
        //     id: obj.id,
        //     version: obj.version + 1,
        //     data: obj.data,
        // };
        //
        // let result = conn
        //     .execute(
        //         &self.queries.update_sql_query,
        //         &[
        //             &updated_model.version,
        //             &json_data,
        //             &updated_model.id,
        //             &obj.version,
        //         ],
        //     )
        //     .await?;
        //
        // if result == 0 {
        //     return Err(C3p0Error::OptimisticLockError{ message: format!("Cannot update data in table [{}] with id [{}], version [{}]: data was changed!",
        //                                                                 &self.queries.qualified_table_name, &updated_model.id, &obj.version
        //     )});
        // }
        //
        // Ok(updated_model)
    }
}
