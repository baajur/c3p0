use c3p0::{C3p0, NewModel};
use c3p0::client::{DbManager, DbManagerBuilder};

#[cfg(feature = "pg")]
mod shared_pg;
#[cfg(feature = "pg")]
use crate::shared_pg::*;

#[cfg(feature = "mysql")]
mod shared_mysql;
#[cfg(feature = "mysql")]
use crate::shared_mysql::*;

struct TestTableRepository<'a> {
    conf: DbManager<'a, TestData>,
}

impl<'a> C3p0<TestData, DbManager<'a, TestData>> for TestTableRepository<'a> {
    fn db_manager(&self) -> &DbManager<'a, TestData> {
        &self.conf
    }
}

#[test]
fn mysql_basic_crud() {
    SINGLETON.get(|(pool, _)| {
        let mut conn = pool.get().unwrap();

        let conf = DbManagerBuilder::new("TEST_TABLE").build();

        let jpo = TestTableRepository { conf };

        assert!(jpo.create_table_if_not_exists(&mut conn).is_ok());

        let model = NewModel::new(TestData {
            first_name: "my_first_name".to_owned(),
            last_name: "my_last_name".to_owned(),
        });

        let saved_model = jpo.save(&mut conn, model.clone()).unwrap();
        assert!(saved_model.id >= 0);

        let found_model = jpo.find_by_id(&mut conn, &saved_model.id).unwrap().unwrap();
        assert_eq!(saved_model.id, found_model.id);
        assert_eq!(saved_model.version, found_model.version);
        assert_eq!(saved_model.data.first_name, found_model.data.first_name);
        assert_eq!(saved_model.data.last_name, found_model.data.last_name);
    });
}
