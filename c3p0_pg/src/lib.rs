use crate::error::into_c3p0_error;
use c3p0::codec::Codec;
use c3p0::error::C3p0Error;
use c3p0::types::OptString;
use c3p0::{IdType, Model, NewModel};
use postgres::rows::Row;
use postgres::Connection;

pub mod error;

#[derive(Clone)]
pub struct Config<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub codec: Codec<DATA>,

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

    pub delete_all_sql_query: String,
    pub delete_by_id_sql_query: String,

    pub save_sql_query: String,

    pub create_table_sql_query: String,
    pub drop_table_sql_query: String,
}

#[derive(Clone)]
pub struct ConfigBuilder<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    codec: Codec<DATA>,
    id_field_name: String,
    version_field_name: String,
    data_field_name: String,
    table_name: String,
    schema_name: Option<String>,
}

impl<DATA> ConfigBuilder<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new<T: Into<String>>(table_name: T) -> Self {
        let table_name = table_name.into();
        ConfigBuilder {
            codec: Default::default(),
            table_name: table_name.clone(),
            id_field_name: "id".to_owned(),
            version_field_name: "version".to_owned(),
            data_field_name: "data".to_owned(),
            schema_name: None,
        }
    }

    pub fn with_codec(mut self, codec: Codec<DATA>) -> ConfigBuilder<DATA> {
        self.codec = codec;
        self
    }

    pub fn with_id_field_name<T: Into<String>>(mut self, id_field_name: T) -> ConfigBuilder<DATA> {
        self.id_field_name = id_field_name.into();
        self
    }

    pub fn with_version_field_name<T: Into<String>>(
        mut self,
        version_field_name: T,
    ) -> ConfigBuilder<DATA> {
        self.version_field_name = version_field_name.into();
        self
    }

    pub fn with_data_field_name<T: Into<String>>(
        mut self,
        data_field_name: T,
    ) -> ConfigBuilder<DATA> {
        self.data_field_name = data_field_name.into();
        self
    }

    pub fn with_schema_name<O: Into<OptString>>(mut self, schema_name: O) -> ConfigBuilder<DATA> {
        self.schema_name = schema_name.into().value;
        self
    }

    pub fn build(self) -> Config<DATA> {
        let qualified_table_name = match &self.schema_name {
            Some(schema_name) => format!(r#"{}."{}""#, schema_name, self.table_name),
            None => self.table_name.clone(),
        };

        Config {
            count_all_sql_query: format!("SELECT COUNT(*) FROM {}", qualified_table_name,),

            exists_by_id_sql_query: format!(
                "SELECT EXISTS (SELECT 1 FROM {} WHERE {} = $1)",
                qualified_table_name, self.id_field_name,
            ),

            find_all_sql_query: format!(
                "SELECT {}, {}, {} FROM {} ORDER BY {} ASC",
                self.id_field_name,
                self.version_field_name,
                self.data_field_name,
                qualified_table_name,
                self.id_field_name,
            ),

            find_by_id_sql_query: format!(
                "SELECT {}, {}, {} FROM {} WHERE {} = $1 LIMIT 1",
                self.id_field_name,
                self.version_field_name,
                self.data_field_name,
                qualified_table_name,
                self.id_field_name,
            ),

            delete_all_sql_query: format!("DELETE FROM {}", qualified_table_name,),

            delete_by_id_sql_query: format!(
                "DELETE FROM {} WHERE {} = $1",
                qualified_table_name, self.id_field_name,
            ),

            save_sql_query: format!(
                "INSERT INTO {} ({}, {}) VALUES ($1, $2) RETURNING {}",
                qualified_table_name,
                self.version_field_name,
                self.data_field_name,
                self.id_field_name
            ),

            create_table_sql_query: format!(
                r#"
                CREATE TABLE IF NOT EXISTS {} (
                    {} bigserial primary key,
                    {} int not null,
                    {} JSONB
                )
                "#,
                qualified_table_name,
                self.id_field_name,
                self.version_field_name,
                self.data_field_name
            ),

            drop_table_sql_query: format!("DROP TABLE IF EXISTS {}", qualified_table_name),

            codec: self.codec,
            qualified_table_name,
            table_name: self.table_name,
            id_field_name: self.id_field_name,
            version_field_name: self.version_field_name,
            data_field_name: self.data_field_name,
            schema_name: self.schema_name,
        }
    }
}

pub trait C3p0<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn conf(&self) -> &Config<DATA>;

    fn to_model(&self, row: Row) -> Result<Model<DATA>, C3p0Error> {
        //id: Some(row.get(self.id_field_name.as_str())),
        //version: row.get(self.version_field_name.as_str()),
        //data: (conf.codec.from_value)(row.get(self.data_field_name.as_str()))?
        let conf = self.conf();
        let id = row.get(0);
        let version = row.get(1);
        let data = (conf.codec.from_value)(row.get(2))?;
        Ok(Model { id, version, data })
    }

    fn create_table_if_not_exists(&self, conn: &Connection) -> Result<u64, C3p0Error> {
        conn.execute(&self.conf().create_table_sql_query, &[])
            .map_err(into_c3p0_error)
    }

    fn drop_table_if_exists(&self, conn: &Connection) -> Result<u64, C3p0Error> {
        conn.execute(&self.conf().drop_table_sql_query, &[])
            .map_err(into_c3p0_error)
    }

    fn count_all(&self, conn: &Connection) -> Result<IdType, C3p0Error> {
        let conf = self.conf();
        let stmt = conn
            .prepare(&conf.count_all_sql_query)
            .map_err(into_c3p0_error)?;
        let result = stmt
            .query(&[])
            .map_err(into_c3p0_error)?
            .iter()
            .next()
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?
            .get(0);
        Ok(result)
    }

    fn exists_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Connection,
        id: ID,
    ) -> Result<bool, C3p0Error> {
        let conf = self.conf();
        let stmt = conn
            .prepare(&conf.exists_by_id_sql_query)
            .map_err(into_c3p0_error)?;
        let id_into = id.into();
        let result = stmt
            .query(&[id_into])
            .map_err(into_c3p0_error)?
            .iter()
            .next()
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?
            .get(0);
        Ok(result)
    }

    fn find_all(&self, conn: &Connection) -> Result<Vec<Model<DATA>>, C3p0Error> {
        let conf = self.conf();
        let stmt = conn
            .prepare(&conf.find_all_sql_query)
            .map_err(into_c3p0_error)?;
        stmt.query(&[])
            .map_err(into_c3p0_error)?
            .iter()
            .map(|row| self.to_model(row))
            .collect()
    }

    fn find_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Connection,
        id: ID,
    ) -> Result<Option<Model<DATA>>, C3p0Error> {
        let conf = self.conf();
        let stmt = conn
            .prepare(&conf.find_by_id_sql_query)
            .map_err(into_c3p0_error)?;
        stmt.query(&[id.into()])
            .map_err(into_c3p0_error)?
            .iter()
            .next()
            .map(|row| self.to_model(row))
            .transpose()
    }

    fn delete_all(&self, conn: &Connection) -> Result<u64, C3p0Error> {
        let conf = self.conf();
        let stmt = conn
            .prepare(&conf.delete_all_sql_query)
            .map_err(into_c3p0_error)?;
        stmt.execute(&[]).map_err(into_c3p0_error)
    }

    fn delete_by_id<'a, ID: Into<&'a IdType>>(
        &'a self,
        conn: &Connection,
        id: ID,
    ) -> Result<u64, C3p0Error> {
        let conf = self.conf();
        let stmt = conn
            .prepare(&conf.delete_by_id_sql_query)
            .map_err(into_c3p0_error)?;
        stmt.execute(&[id.into()]).map_err(into_c3p0_error)
    }

    fn save(&self, conn: &Connection, obj: NewModel<DATA>) -> Result<Model<DATA>, C3p0Error> {
        let conf = self.conf();
        let stmt = conn
            .prepare(&conf.save_sql_query)
            .map_err(into_c3p0_error)?;
        let json_data = (conf.codec.to_value)(&obj.data)?;
        let id = stmt
            .query(&[&obj.version, &json_data])
            .map_err(into_c3p0_error)?
            .iter()
            .next()
            .ok_or_else(|| C3p0Error::IteratorError {
                message: "Cannot iterate next element".to_owned(),
            })?
            .get(0);

        Ok(Model {
            id,
            version: obj.version,
            data: obj.data,
        })
    }
}

#[derive(Clone)]
pub struct C3p0Repository<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    conf: Config<DATA>,
    phantom_data: std::marker::PhantomData<DATA>,
}

impl<DATA> C3p0Repository<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn build(conf: Config<DATA>) -> Self {
        C3p0Repository {
            conf,
            phantom_data: std::marker::PhantomData,
        }
    }
}

impl<DATA> C3p0<DATA> for C3p0Repository<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn conf(&self) -> &Config<DATA> {
        &self.conf
    }
}
