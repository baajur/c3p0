use serde_derive::{Deserialize, Serialize};

pub type IdType = i64;
pub type VersionType = i32;

#[derive(Clone, Serialize, Deserialize)]
pub struct Model<DATA>
where
    DATA: Clone + serde::ser::Serialize,
{
    pub id: IdType,
    pub version: VersionType,
    #[serde(bound(deserialize = "DATA: serde::Deserialize<'de>"))]
    pub data: DATA,
}

impl<DATA> Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn into_new(self) -> NewModel<DATA> {
        NewModel {
            version: 0,
            data: self.data,
        }
    }
}

impl<'a, DATA> Into<&'a IdType> for &'a Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn into(self) -> &'a IdType {
        &self.id
    }
}

impl<DATA> std::fmt::Debug for Model<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "Model {{ id: {:?}, version: {:?}, data: {:?} }}",
            self.id, self.version, self.data
        )?;
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewModel<DATA>
where
    DATA: Clone + serde::ser::Serialize,
{
    pub version: VersionType,
    #[serde(bound(deserialize = "DATA: serde::Deserialize<'de>"))]
    pub data: DATA,
}

impl<DATA> NewModel<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    pub fn new(data: DATA) -> Self {
        NewModel { version: 0, data }
    }
}

impl<DATA> std::fmt::Debug for NewModel<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "NewModel {{ version: {:?}, data: {:?} }}",
            self.version, self.data
        )?;
        Ok(())
    }
}

impl<DATA> Default for NewModel<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned + Default,
{
    fn default() -> Self {
        NewModel::new(DATA::default())
    }
}

impl<DATA> From<DATA> for NewModel<DATA>
where
    DATA: Clone + serde::ser::Serialize + serde::de::DeserializeOwned,
{
    fn from(data: DATA) -> Self {
        NewModel::new(data)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use serde_derive::{Deserialize, Serialize};
    use serde_json;

    #[test]
    fn model_should_be_serializable() -> Result<(), Box<dyn std::error::Error>> {
        let model = Model {
            id: 1,
            version: 1,
            data: SimpleData {
                name: "test".to_owned(),
            },
        };

        let serialize = serde_json::to_string(&model)?;
        let deserialize: Model<SimpleData> = serde_json::from_str(&serialize)?;

        assert_eq!(model.id, deserialize.id);
        assert_eq!(model.version, deserialize.version);
        assert_eq!(model.data, deserialize.data);

        Ok(())
    }

    #[test]
    fn new_model_should_be_serializable() -> Result<(), Box<dyn std::error::Error>> {
        let model = NewModel::new(SimpleData {
            name: "test".to_owned(),
        });

        let serialize = serde_json::to_string(&model)?;
        let deserialize: NewModel<SimpleData> = serde_json::from_str(&serialize)?;

        assert_eq!(model.version, deserialize.version);
        assert_eq!(model.data, deserialize.data);
        Ok(())
    }

    #[test]
    fn model_should_impl_debug_if_data_is_debug() {
        let model = Model {
            id: 1,
            version: 1,
            data: SimpleData {
                name: "test".to_owned(),
            },
        };

        println!("Debug model: {:?}", model);
    }

    #[test]
    fn new_model_should_impl_debug_if_data_is_debug() {
        let model = NewModel::new(SimpleData {
            name: "test".to_owned(),
        });

        println!("Debug model: {:?}", model);
    }

    #[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
    struct SimpleData {
        name: String,
    }
}
