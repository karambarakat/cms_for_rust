// use query_macros::IntoQuery;
// use sqlx::{Pool, Sqlite};
// use query::{prelude::*, PrepareStatement};
// use query::{select_st::SelectSt, QuickQuery};
//
//
// #[tokio::test]
// async fn quick_queries() {
//     let conn = Pool::<Sqlite>::connect(":memory:").await.unwrap();
//     let mut stmt = stmt::select("Todos", ());
//
//     sqlx::query("CREATE TABLE Todos ( id INTEGER PRIMARY KEY, title TEXT NOT NULL);")
//         .execute(&conn)
//         .await
//         .unwrap();
//
//     sqlx::query("INSERT INTO Todos (title) VALUES ('hi'), ('new todo'), ('new todo 2');")
//         .execute(&conn)
//         .await
//         .unwrap();
//
//     stmt.select(col("title"));
//     stmt.select(col("id"));
//
//     stmt.where_(col("id").eq(|| 1));
//
//     let stmt = stmt.prepare::<(String, i32)>();
//
//     let sql = stmt.sql_str();
//     assert_eq!(sql, "SELECT title, id FROM Todos WHERE id = $1;");
//
//     let res = stmt.fetch_all(&conn).await.unwrap();
//
//     assert_eq!(vec![("hi".to_string(), 1),], res);
// }
//
// #[derive(Debug, IntoQuery)]
// struct Input {
//     id: i32,
// }
//
// #[tokio::test]
// async fn prepared_queries() {
//     let conn = Pool::<Sqlite>::connect(":memory:").await.unwrap();
//     // let mut stmt = SelectSt::<
//     //     Sqlite,
//     //     PrepareStatement<Sqlite, (Input,)>,
//     // >::from("Todos");
//     let (input, mut stmt) =
//         stmt::select("Todos", prepared_stmt::<Input>());
//
//     sqlx::query("CREATE TABLE Todos ( id INTEGER PRIMARY KEY, title TEXT NOT NULL);")
//         .execute(&conn)
//         .await
//         .unwrap();
//
//     sqlx::query("INSERT INTO Todos (title) VALUES ('hi'), ('new todo'), ('new todo 2');")
//         .execute(&conn)
//         .await
//         .unwrap();
//
//     stmt.select(col("title"));
//     stmt.select(col("id"));
//
//     stmt.where_(col("id").eq(input.id));
//
//     let stmt = stmt.prepare::<(String, i32)>();
//
//     let sql = stmt.as_str();
//     assert_eq!(sql, "SELECT title, id FROM Todos WHERE id = $1;");
//
//     let res = stmt.fetch_all(Input { id: 1 }, &conn).await.unwrap();
//
//     assert_eq!(vec![("hi".to_string(), 1),], res);
//
//     let res = stmt.fetch_all(Input { id: 2 }, &conn).await.unwrap();
//     assert_eq!(vec![("new todo".to_string(), 2),], res);
// }
