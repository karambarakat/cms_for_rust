pub struct StringQuery<I> {
    pub sql: String,
    pub input: I,
}

pub mod row_into_json {
    pub struct RowIntoJson {
        cache: Option<Vec<(String, bool, String)>>,
        output: Vec<Value>,
    }

    impl RowIntoJson {
        pub fn sink() -> Self {
            Self {
                cache: Default::default(),
                output: Default::default(),
            }
        }
        pub fn take_rows<'this, S>(
            &'this mut self,
        ) -> impl FnMut(S::Row) -> sqlx::Result<()> + 'this
        where
            S: Database + RowToJson,
            S::Row: Row<Database = S>,
        {
            use sqlx::Column;
            use sqlx::Row;
            use sqlx::TypeInfo;
            |row| {
                match &self.cache {
                    None => {
                        let tc = row
                            .columns()
                            .iter()
                            .map(|c| {
                                (
                                    c.name().to_string(),
                                    c.type_info().is_null(),
                                    c.type_info()
                                        .name()
                                        .to_string(),
                                )
                            })
                            .collect::<Vec<_>>();

                        self.cache = Some(tc.clone());
                    }
                    Some(_) => {}
                };

                let res = S::row_to_json(
                    row,
                    self.cache.as_ref().unwrap(),
                );
                self.output.push(Value::Object(res));

                Ok(())
            }
        }
        pub fn result(self) -> Vec<Value> {
            self.output
        }
    }

    pub trait RowToJson: Database {
        fn row_to_json(
            row: Self::Row,
            type_info: &Vec<(String, bool, String)>,
        ) -> Map<String, Value>;
    }

    use serde_json::{Map, Value};
    use sqlx::{Database, Row, Sqlite};

    impl RowToJson for Sqlite {
        fn row_to_json(
            row: Self::Row,
            type_info: &Vec<(String, bool, String)>,
        ) -> Map<String, Value> {
            let mut output: Map<String, Value> =
                Default::default();

            let mut type_info = type_info.iter();

            while let Some((col, _is_null, ty)) =
                type_info.next()
            {
                match ty.as_str() {
                    "NULL" => {
                        output.insert(col.clone(), Value::Null);
                    }
                    "INTEGER" => {
                        output.insert(
                            col.clone(),
                            Value::Number(
                                row.get::<i64, _>(col.as_str())
                                    .into(),
                            ),
                        );
                    }
                    "TEXT" => {
                        output.insert(
                            col.clone(),
                            Value::String(
                                row.get::<String, _>(
                                    col.as_str(),
                                ),
                            ),
                        );
                    }
                    "REAL" => {
                        panic!("REAL is not supported in converting to json");
                        // output.insert(
                        //     col.clone(),
                        //     Value::Number(
                        //         row.get::<f32, _>(col.as_str())
                        //             .into(),
                        //     ),
                        // );
                    }
                    "BLOB" => {
                        output.insert(
                            col.clone(),
                            Value::String(
                                row.get::<Vec<u8>, _>(
                                    col.as_str(),
                                )
                                .iter()
                                .map(|b| format!("{:02x}", b))
                                .collect::<String>(),
                            ),
                        );
                    }
                    // non-standard extensions
                    "BOOLEAN" => {
                        output.insert(
                            col.clone(),
                            Value::Bool(
                                row.get::<bool, _>(col.as_str()),
                            ),
                        );
                    }
                    "DATETIME" | "DATE" | "TIME" => {
                        output.insert(
                            col.clone(),
                            Value::String(
                                row.get::<String, _>(
                                    col.as_str(),
                                ),
                            ),
                        );
                    }
                    _ => {
                        tracing::warn!(
                            "unsupported type: {}",
                            ty
                        );
                    }
                }
            }
            output
        }
    }

    #[cfg(test)]
    #[tokio::test]
    async fn test_json_rows() {
        use serde_json::json;

        use crate::{
            execute_no_cache::ExecuteNoCache,
            string_query::StringQuery,
        };

        let pool =
            sqlx::Pool::<Sqlite>::connect("sqlite::memory:")
                .await
                .unwrap();

        let mut json_rows = RowIntoJson::sink();

        StringQuery {
            sql: String::from(
                "
                CREATE TABLE test (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                );
                INSERT INTO test (name) VALUES ($1), ($2), ($3);
                SELECT *, id_ FROM test 
                   LEFT JOIN 
                      (SELECT id as id_ FROM test where id>1)
                   ON id_=id;
            ",
            ),
            input: ("Alice", "Bob", "Charlie"),
        }
        .fetch_all(&pool, json_rows.take_rows::<Sqlite>())
        .await
        .unwrap();

        assert_eq!(
            serde_json::to_value(json_rows.result()).unwrap(),
            json!([
                {"id": 1, "name": "Alice", "id_": 0},
                {"id": 2, "name": "Bob", "id_": 2},
                {"id": 3, "name": "Charlie", "id_": 3}
            ])
        );
    }
}
