use crate::tests::util::rand_string;
use crate::*;

#[test]
fn should_commit_transaction() {
    SINGLETON.get(|(pool, _)| {
        let c3p0: C3p0Impl = pool.clone();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        {
            let conn = c3p0.connection().unwrap();
            assert!(conn
                .execute(
                    &format!(
                        r"CREATE TABLE {} (
                             name varchar(255)
                          )",
                        table_name
                    ),
                    &[]
                )
                .is_ok());
        }

        let result: Result<_, C3p0Error> = c3p0.transaction(|conn| {
            conn.execute(
                &format!(r"INSERT INTO {} (name) VALUES ('one')", table_name),
                &[],
            )
            .unwrap();
            conn.execute(
                &format!(r"INSERT INTO {} (name) VALUES ('two')", table_name),
                &[],
            )
            .unwrap();
            conn.execute(
                &format!(r"INSERT INTO {} (name) VALUES ('three')", table_name),
                &[],
            )
            .unwrap();
            Ok(())
        });

        assert!(result.is_ok());

        {
            let conn = c3p0.connection().unwrap();
            let count = conn
                .fetch_one_value::<i64>(&format!(r"SELECT COUNT(*) FROM {}", table_name), &[])
                .unwrap();
            assert_eq!(3, count);

            assert!(conn
                .execute(&format!(r"DROP TABLE {}", table_name), &[])
                .is_ok());
        }
    });
}

#[test]
fn should_rollback_transaction() {
    SINGLETON.get(|(pool, _)| {
        let c3p0: C3p0Impl = pool.clone();
        let table_name = format!("TEST_TABLE_{}", rand_string(8));
        {
            let conn = c3p0.connection().unwrap();
            assert!(conn
                .batch_execute(&format!(
                    r"CREATE TABLE {} (
                             name varchar(255)
                          )",
                    table_name
                ))
                .is_ok());
        }

        let result: Result<(), C3p0Error> = c3p0.transaction(|conn| {
            conn.execute(
                &format!(r"INSERT INTO {} (name) VALUES ('one')", table_name),
                &[],
            )
            .unwrap();
            conn.execute(
                &format!(r"INSERT INTO {} (name) VALUES ('two')", table_name),
                &[],
            )
            .unwrap();
            conn.execute(
                &format!(r"INSERT INTO {} (name) VALUES ('three')", table_name),
                &[],
            )
            .unwrap();
            Err(C3p0Error::ResultNotFoundError)?
        });

        assert!(result.is_err());

        {
            let conn = c3p0.connection().unwrap();
            let count = conn
                .fetch_one_value::<i64>(&format!(r"SELECT COUNT(*) FROM {}", table_name), &[])
                .unwrap();
            assert_eq!(0, count);

            assert!(conn
                .execute(&format!(r"DROP TABLE {}", table_name), &[])
                .is_ok());
        }
    });
}

#[test]
fn transaction_should_return_internal_error() {
    use err_derive::Error;

    #[derive(Error, Debug, PartialEq)]
    pub enum CustomError {
        #[error(display = "InnerError")]
        InnerError,
        #[error(display = "C3p0Error")]
        C3p0Error,
    }

    impl From<C3p0Error> for CustomError {
        fn from(_: C3p0Error) -> Self {
            CustomError::C3p0Error
        }
    }

    SINGLETON.get(|(pool, _)| {
        let c3p0: C3p0Impl = pool.clone();

        let result: Result<(), _> = c3p0.transaction(|_| Err(CustomError::InnerError));

        assert!(result.is_err());

        match &result {
            Err(CustomError::InnerError) => assert!(true),
            _ => assert!(false),
        }
    });
}
