use crate::error::C3p0Error;
use crate::json::codec::JsonCodec;
use crate::json::model::*;

pub mod builder;
pub mod codec;
pub mod model;

#[derive(Clone)]
pub struct Queries {
    pub id_field_name: String,
    pub version_field_name: String,
    pub data_field_name: String,

    pub table_name: String,
    pub schema_name: Option<String>,
    pub qualified_table_name: String,

    pub count_all_sql_query: String,
    pub exists_by_id_sql_query: String,

    pub find_all_sql_query: String,
    pub find_by_id_sql_query: String,

    pub delete_sql_query: String,
    pub delete_all_sql_query: String,
    pub delete_by_id_sql_query: String,

    pub save_sql_query: String,

    pub update_sql_query: String,

    pub create_table_sql_query: String,
    pub drop_table_sql_query: String,
    pub drop_table_sql_query_cascade: String,
    pub lock_table_sql_query: Option<String>,
}

pub trait C3p0Json<DATA, CODEC>: Clone
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
    CODEC: JsonCodec<DATA>,
{
    type CONN;

    fn codec(&self) -> &CODEC;

    fn create_table_if_not_exists(&self, conn: &Self::CONN) -> Result<(), C3p0Error>;

    fn drop_table_if_exists(&self, conn: &Self::CONN, cascade: bool) -> Result<(), C3p0Error>;

    fn count_all(&self, conn: &Self::CONN) -> Result<u64, C3p0Error>;

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<bool, C3p0Error>;

    fn fetch_all(&self, conn: &Self::CONN) -> Result<Vec<Model<DATA>>, C3p0Error>;

    fn fetch_one_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error>;

    fn delete(&self, conn: &Self::CONN, obj: &Model<DATA>) -> Result<u64, C3p0Error>;

    fn delete_all(&self, conn: &Self::CONN) -> Result<u64, C3p0Error>;

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Self::CONN,
        id: ID,
    ) -> Result<u64, C3p0Error>;

    fn save(&self, conn: &Self::CONN, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error>;

    fn update(&self, conn: &Self::CONN, obj: Model<DATA>) -> Result<Model<DATA>, C3p0Error>;
}
