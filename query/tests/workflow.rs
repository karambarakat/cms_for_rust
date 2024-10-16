// #[tokio::test]
// async fn query_workflow_1() {
//     tracing_subscriber::fmt()
//         .with_max_level(Level::DEBUG)
//         .init();
//
//     let conn = sqlx::SqlitePool::connect("sqlite://:memory:")
//         .await
//         .unwrap();
//
//     let mut users =
//         stmt::create_table_if_not_exists::<Sqlite>("Users");
//
//     users.column("id", primary_key());
//     users.column::<String>(
//         "avatar",
//         default(sanitize("NAN".to_string())).not_null(),
//     );
//     users.column::<Option<String>>(
//         "display_name",
//         check_if_null(),
//     );
//     users.column::<String>("password", check_if_null());
//
//     let mut testing_copy = String::new();
//
//     users
//         .execute(DebugSql(&conn, |stmt| {
//             assert_eq!(
//                 stmt,
//                 "
//                    CREATE TABLE IF NOT EXISTS Users (
//                        id INTEGER PRIMARY KEY AUTOINCREMENT,
//                        avatar TEXT DEFAULT `NAN` NOT NULL,
//                        display_name TEXT,
//                        password TEXT NOT NULL
//                    );"
//                 .un_pretty()
//                 .to_string()
//             )
//         }))
//         .await
//         .expect("creating users table failed");
//
//     let mut logs = stmt::create_table_if_not_exists("Logs");
//
//     logs.column("id", primary_key());
//     logs.column::<String>("title", ());
//     logs.column::<i32>("amount", ());
//
//     logs.foreign_key(Fk {
//         not_null: true,
//         column: "created_by",
//         refer_table: "Users",
//         refer_column: "id",
//     });
//
//     logs.foreign_key(Fk {
//         not_null: false,
//         column: "category_id",
//         refer_table: "Categories",
//         refer_column: "id",
//     });
//
//     logs.execute(DebugSql(&conn, |stmt| {
//         assert_eq!(
//             stmt,
//             "
//                    CREATE TABLE IF NOT EXISTS Logs (
//                        id INTEGER PRIMARY KEY AUTOINCREMENT,
//                        title TEXT NOT NULL,
//                        amount INTEGER NOT NULL,
//                        created_by INTEGER NOT NULL,
//                        category_id INTEGER,
//                        FOREIGN KEY (created_by)
//                            REFERENCES Users(id),
//                        FOREIGN KEY (category_id)
//                            REFERENCES Categories(id)
//                            ON DELETE SET NULL
//                    );"
//             .un_pretty()
//             .to_string()
//         )
//     }))
//     .await
//     .expect("creating logs table failed");
//     let mut category =
//         stmt::create_table_if_not_exists("Categories");
//
//     category.column("id", primary_key());
//     category.column::<String>("name", ());
//
//     category
//         .execute(DebugSql(&conn, |stmt| {
//             assert_eq!(
//                 stmt,
//                 "
//                    CREATE TABLE IF NOT EXISTS Categories (
//                        id INTEGER PRIMARY KEY AUTOINCREMENT,
//                        name TEXT NOT NULL
//                    );"
//                 .un_pretty()
//                 .to_string()
//             )
//         }))
//         .await
//         .expect("creating categories table failed");
//
//     #[derive(Query)]
//     struct Logs {
//         title: String,
//         amount: i32,
//         created_by: i32,
//         category_id: Option<i32>,
//     }
//
//     #[derive(Query)]
//     struct Users {
//         avatar: Option<String>,
//         display_name: Option<String>,
//         password: String,
//     }
//
//     #[derive(Query)]
//     struct Categories {
//         name: String,
//     }
//
//     #[derive(
//         Query, FromRow, PartialEq, Eq, PartialOrd, Ord, Debug,
//     )]
//     struct ReturningUser {
//         id: i32,
//         avatar: String,
//     }
//
//     let user: Vec<ReturningUser> = insert_many_returning(
//         &conn,
//         vec![
//             Users {
//                 avatar: Some("avatar_01".to_string()),
//                 display_name: Some("John Doe".to_string()),
//                 password: "password".to_string(),
//             },
//             Users {
//                 avatar: Some("avatar_02".to_string()),
//                 display_name: Some("Sarah Miller".to_string()),
//                 password: "123pass".to_string(),
//             },
//         ],
//     )
//     .await
//     .unwrap();
//
//     assert_eq!(
//         user,
//         vec!(
//             ReturningUser {
//                 id: 1,
//                 avatar: "avatar_01".to_string(),
//             },
//             ReturningUser {
//                 id: 2,
//                 avatar: "avatar_02".to_string(),
//             }
//         )
//     );
//
//     stmt::insert_many(
//         &conn,
//         vec![
//             Categories {
//                 name: "Food".to_string(),
//             },
//             Categories {
//                 name: "Transport".to_string(),
//             },
//         ],
//     )
//     .await
//     .unwrap();
//
//     stmt::insert(
//         &conn,
//         Logs {
//             title: "First Log".to_string(),
//             created_by: 1,
//             amount: 100,
//             category_id: Some(1),
//         },
//     )
//     .await
//     .unwrap();
//
//     stmt::insert_many(
//         &conn,
//         vec![
//             Logs {
//                 title: "Second Log".to_string(),
//                 created_by: 1,
//                 amount: 200,
//                 category_id: None,
//             },
//             Logs {
//                 title: "Third Log".to_string(),
//                 created_by: 2,
//                 amount: 300,
//                 category_id: Some(2),
//             },
//         ],
//     )
//     .await
//     .unwrap();
//
//     #[derive(
//         FromRow, Debug, PartialEq, Eq, PartialOrd, Ord, Query,
//     )]
//     struct ReturnCategory {
//         id: i64,
//         name: String,
//     }
//
//     #[derive(
//         FromRow, Debug, PartialEq, Eq, PartialOrd, Ord, Query,
//     )]
//     struct ReturnUser {
//         id: i64,
//         avatar: String,
//         display_name: Option<String>,
//         password: String,
//     }
//
//     #[derive(
//         FromRow, Debug, PartialEq, Eq, PartialOrd, Ord, Query,
//     )]
//     struct ReturnLog {
//         id: i16,
//         title: String,
//         amount: i16,
//         created_by: i16,
//         category_id: i16,
//     }
//
//     let mut user = stmt::select("Users");
//
//     user.select(col("id"));
//     user.select(col("avatar"));
//     user.select(col("display_name"));
//     user.select(col("password"));
//
//     let user: Vec<ReturnUser> =
//         user.fetch_all(&conn).await.unwrap();
//
//     assert_eq!(
//         user,
//         vec![
//             ReturnUser {
//                 id: 1,
//                 avatar: "avatar_01".to_string(),
//                 display_name: Some("John Doe".to_string()),
//                 password: "password".to_string(),
//             },
//             ReturnUser {
//                 id: 2,
//                 avatar: "avatar_02".to_string(),
//                 display_name: Some("Sarah Miller".to_string()),
//                 password: "123pass".to_string(),
//             }
//         ]
//     );
//
//     let mut user = stmt::select("Logs");
//
//     user.select(col("title"));
//     user.where_(col("created_by").eq(|| 1));
//
//     user.select(ft("Users").col("avatar").alias("user_avatar"));
//     user.join(Join {
//         ty: join_type::Left,
//         on_table: "Users",
//         on_column: "id",
//         local_column: "created_by",
//     });
//
//     user.select(
//         ft("Categories").col("name").alias("category_name"),
//     );
//     user.join(Join {
//         ty: join_type::Left,
//         on_table: "Categories",
//         on_column: "id",
//         local_column: "category_id",
//     });
//
//     user.limit(|| 2);
//     user.order_by("title", order_by::DESC);
//
//     let user: Vec<(String, String, Option<String>)> = user
//         .fetch_all(DebugSql(&conn, |stmt| {
//             assert_eq!(
//                 stmt,
//                 "
//      SELECT
//         title, Users.avatar AS user_avatar, Categories.name AS category_name
//      FROM Logs
//      LEFT JOIN Users ON Users.id = Logs.created_by
//      LEFT JOIN Categories ON Categories.id = Logs.category_id
//      WHERE created_by = $1
//      ORDER BY title DESC
//      LIMIT $2;
// "
//                     .un_pretty()
//                     .to_string()
//             )
//         }))
//         .await
//         .unwrap();
//
//     assert_eq!(
//         user,
//         vec![
//             (
//                 "Second Log".to_string(),
//                 "avatar_01".to_string(),
//                 None,
//             ),
//             (
//                 "First Log".to_string(),
//                 "avatar_01".to_string(),
//                 Some("Food".to_string()),
//             ),
//         ]
//     );
//
//     let mut delete =
//         stmt::delete("Categories").returning::<ReturnCategory>();
//
//     delete.where_(col("id").eq(|| 2));
//
//     let user = delete
//         .delete(DebugSql(&conn, |stmt| {
//             assert_eq!(
//                 stmt,
//                 "
//             DELETE FROM
//                 Categories
//             WHERE id = $1
//             RETURNING
//               id, name;
// "
//                 .un_pretty()
//                 .to_string()
//             )
//         }))
//         .await
//         .unwrap();
//
//     assert_eq!(
//         user,
//         ReturnCategory {
//             id: 2,
//             name: "Transport".to_string(),
//         }
//     );
//
//     let mut select_st = stmt::select("Logs");
//
//     select_st.select(ft("Categories").col("name"));
//
//     select_st.join(Join {
//         ty: join_type::Left,
//         on_table: "Categories",
//         on_column: "id",
//         local_column: "category_id",
//     });
//
//     let selected: Vec<(Option<String>,)> =
//         select_st.fetch_all(&conn).await.unwrap();
//
//     assert_eq!(
//         selected,
//         vec![(Some("Food".to_string()),), (None,), (None,),]
//     );
//
//     let mut update_st = stmt::update("Logs");
//
//     update_st.set("title", || "new_title".to_string());
//     update_st.set("category_id", || 1);
//     update_st.where_(col("id").eq(|| 2));
//
//     update_st
//         .update(DebugSql(&conn, |stmt| {
//             assert_eq!(
//                 stmt,
//                 "
//             UPDATE
//                 Logs
//             SET
//                 title = $1,
//                 category_id = $2
//             WHERE id = $3;
// "
//                 .un_pretty()
//                 .to_string()
//             )
//         }))
//         .await
//         .unwrap();
//
//     let mut select_st = stmt::select("Logs");
//
//     select_st.select(col("title"));
//     select_st.select(col("category_id"));
//
//     let selected: Vec<(String, Option<i32>)> =
//         select_st.fetch_all(&conn).await.unwrap();
//
//     assert_eq!(
//         selected,
//         vec![
//             ("First Log".to_string(), Some(1)),
//             ("new_title".to_string(), Some(1)),
//             ("Third Log".to_string(), None),
//         ]
//     );
// }
