use std::marker::PhantomData;

use serde::Serialize;
use sqlx::Sqlite;

use crate::tuple_index::tuple_as_map::TupleElementKey;

use crate::traits::Resource;

pub mod delete_one;
pub mod insert_one;
pub mod select_many;
pub mod select_one;
pub mod update_one;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SimpleOutput<C> {
    pub id: i64,
    pub attr: C,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct IdOutput<C> {
    pub id: i64,
    #[serde(skip)]
    pub _pd: PhantomData<C>,
}

impl<C: Resource<Sqlite>> TupleElementKey for SimpleOutput<C> {
    fn key() -> &'static str {
        C::table_name()
    }
}

impl<C: Resource<Sqlite>> TupleElementKey for IdOutput<C> {
    fn key() -> &'static str {
        C::table_name()
    }
}

#[cfg(test)]
mod test {
    use std::{marker::PhantomData, panic::catch_unwind};

    use axum::{
        extract::{Path, State},
        Json,
    };
    use queries_for_sqlx::ident_safety::define_schema;
    use serde_json::{from_value, json, Map};
    use sqlx::{Pool, Sqlite};

    use crate::{
        client_example::{Category, Partial, Tag, Todo},
        dynamic_schema::COLLECTIONS,
        operations::{
            insert_one::insert_one_dynamic,
            select_many::get_all_dynamic,
            select_one::{
                get_one, get_one_dynamic, GetOneOutput,
                InputGetOne,
            },
            IdOutput, SimpleOutput,
        },
        relations::{link_id, relation},
        tuple_index::TupleAsMap,
    };

    use super::{
        delete_one::delete_one_dynmaic, insert_one::insert_one,
        update_one::update_one_dynmaic,
    };

    async fn init() -> Pool<Sqlite> {
        let pool = Pool::<Sqlite>::connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query::<Sqlite>(
            r#"
            CREATE TABLE Todo (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                done BOOLEAN NOT NULL,
                description TEXT,
                category_id INTEGER,
                FOREIGN KEY (category_id) REFERENCES Category (id) ON DELETE SET NULL
            );
            CREATE TABLE Tag (
                id INTEGER PRIMARY KEY,
                tag_title TEXT NOT NULL
            );
            CREATE TABLE Category (
                id INTEGER PRIMARY KEY,
                cat_title TEXT NOT NULL
            );

            CREATE TABLE TodoTag (
                todo_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (todo_id, tag_id),
                FOREIGN KEY (todo_id) REFERENCES Todo (id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES Tag (id) ON DELETE CASCADE
            );

            INSERT INTO Tag (tag_title) VALUES 
                ('tag_1'), ('tag_2'), ('tag_3');
            INSERT INTO Category (cat_title) VALUES ('category_1'), ('category_2'), ('category_3');

            INSERT INTO Todo (title, done, category_id) VALUES
                ('todo_1', 1, 3),
                ('todo_2', 0, 3),
                ('todo_3', 1, NULL),
                ('todo_4', 0, 1),
                ('todo_5', 1, NULL);
    
            INSERT INTO TodoTag (todo_id, tag_id) VALUES
                (1, 1),         (1, 3),
                (2, 1), (2, 2),
                (3, 1), 
                                (4, 3),
                        (5, 2)        ;

            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        define_schema(&[
            ("Todo", &["id", "title", "done"]),
            ("Tag", &["id", "tag_title"]),
            ("Category", &["id", "cat_title"]),
            ("TodoTag", &["todo_id", "tag_id"]),
        ]);

        pool
    }

    #[tokio::test]
    async fn test_1() {
        let db = init().await;

        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        test_get_one(db.clone()).await;

        test_get_all(db.clone()).await;

        test_insert_one(db.clone()).await;

        test_update_one(db.clone()).await;

        test_delete_one(db.clone()).await;
    }

    async fn test_delete_one(db: Pool<Sqlite>) {
        let res = delete_one_dynmaic(
            State(db.clone()),
            Path("todo".to_string()),
            Json(
                from_value(json!({
                    "id": 2,
                    "return_attr": true,
                    "return_residual": ["category"]
                }))
                .unwrap(),
            ),
        )
        .await
        .unwrap();

        pretty_assertions::assert_eq!(
            serde_json::to_value(res.0).unwrap(),
            json! {{
                "attr": {
                    "description": null,
                    "done": false,
                    "title": "todo_2",
                },
                "id": 2,
                "relations": {
                    "category": 3
                }
            }}
        );
    }

    async fn test_get_all(db: Pool<Sqlite>) {
        let res = get_all_dynamic(
            State(db.clone()),
            Path("todo".to_string()),
            Json(
                from_value(json!({
                    "pagination": {
                        "page": 0,
                        "page_size": 3,
                    },
                    "filters": {},
                    "relations": {
                        "category": {},
                        "tag": {}
                    },
                }))
                .unwrap(),
            ),
        )
        .await
        .unwrap();

        pretty_assertions::assert_eq!(
            serde_json::to_value(res.0).unwrap(),
            json! {{
                "page_count": null,
                "data": [
                    {
                        "id": 1,
                        "attr": { "title": "todo_1", "done": true, "description": null },
                        "relations": {
                            "category": { "id": 3, "attr": { "cat_title": "category_3" } },
                            "tag": [
                                { "id": 1, "attr": { "tag_title": "tag_1" } },
                                { "id": 3, "attr": { "tag_title": "tag_3" } }
                            ]
                        }
                    },

                    {
                        "id": 2,
                        "attr": { "title": "todo_2", "done": false, "description": null },
                        "relations": {
                            "category": { "id": 3, "attr": { "cat_title": "category_3" } },
                            "tag": [
                                { "id": 1, "attr": { "tag_title": "tag_1" } },
                                { "id": 2, "attr": { "tag_title": "tag_2" } }
                            ]
                        }
                    },

                    {
                        "id": 3,
                        "attr": { "title": "todo_3", "done": true, "description": null },
                        "relations": {
                            "category": null,
                            "tag": [
                                { "id": 1, "attr": { "tag_title": "tag_1" } }
                            ]
                        }
                    },

                    // {
                    //     "id": 4,
                    //     "attr": { "title": "todo_4", "done": false, "description": null },
                    //     "relations": {
                    //         "category": { "id": 1, "attr": { "cat_title": "category_1" } },
                    //         "tag": [
                    //             { "id": 3, "attr": { "tag_title": "tag_3" } }
                    //         ]
                    //     }
                    // },
                    //
                    // {
                    //     "id": 5,
                    //     "attr": { "title": "todo_5", "done": true, "description": null },
                    //     "relations": {
                    //         "category": null,
                    //         "tag": [
                    //             { "id": 2, "attr": { "tag_title": "tag_2" } }
                    //         ]
                    //     }
                    // },
                ],
            }}
        );
    }

    async fn test_deep_populate(db: Pool<Sqlite>) {
        let res = get_one::<Todo>()
            .relations_as::<Category, _, _>(|r| {
                // in theory you can do multiple deep relations and/or go deeper
                // for now this is only supported for one-time of
                // C1 --optional_to_many--> C2 --optional_to_many_inverse--> C1
                r.deep_populate::<Todo>()
            })
            .exec_op(db.clone())
            .await;

        pretty_assertions::assert_eq!(
            res,
            Some(GetOneOutput {
                id: 1,
                attr: Todo {
                    title: "todo_1".to_string(),
                    done: true,
                    description: None,
                },
                links: TupleAsMap((Some(GetOneOutput {
                    id: 3,
                    attr: Category {
                        cat_title: "category_3".to_string()
                    },
                    links: (vec![
                        SimpleOutput {
                            id: 1,
                            attr: Todo {
                                title: "todo_1".to_string(),
                                done: true,
                                description: None,
                            },
                        },
                        SimpleOutput {
                            id: 2,
                            attr: Todo {
                                title: "todo_2".to_string(),
                                done: false,
                                description: None,
                            },
                        },
                    ],)
                }),))
            })
        );
    }

    async fn test_get_one(db: Pool<Sqlite>) {
        let res = get_one_dynamic(
            State(db.clone()),
            Path("todo".to_string()),
            Json(
                from_value(json!({
                    "filters": {},
                    "id": 2,
                    "relations": {
                        "tag": {},
                        "category": {}
                    }
                }))
                .unwrap(),
            ),
        )
        .await
        .unwrap();

        pretty_assertions::assert_eq!(
            serde_json::to_value(res.0).unwrap(),
            json! {{
                "attr": {
                    "description": null,
                    "done": false,
                    "title": "todo_2",
                },
                "id": 2,
                "relations": {
                    "category": {
                        "id": 3,
                        "attr": {
                            "cat_title": "category_3",
                        }
                    },
                    "tag": [
                        {
                            "id": 1,
                            "attr": {
                                "tag_title": "tag_1",
                            },
                        },
                        {
                            "id": 2,
                            "attr": {
                                "tag_title": "tag_2",
                            },
                        },
                    ]
                }
            }}
        );

        let res = get_one::<Todo>()
            .by_id(2)
            .relation::<Tag>()
            .relation::<Category>()
            .exec_op(db.clone())
            .await;

        pretty_assertions::assert_eq!(
            res,
            Some(GetOneOutput {
                id: 2,
                attr: Todo {
                    title: "todo_2".to_string(),
                    done: false,
                    description: None,
                },
                links: TupleAsMap((
                    vec![
                        SimpleOutput {
                            id: 1,
                            attr: Tag {
                                tag_title: "tag_1".to_string()
                            }
                        },
                        SimpleOutput {
                            id: 2,
                            attr: Tag {
                                tag_title: "tag_2".to_string()
                            }
                        },
                    ],
                    Some(SimpleOutput {
                        id: 3,
                        attr: Category {
                            cat_title: "category_3".to_string()
                        }
                    }),
                ))
            })
        );
    }

    async fn test_update_one(db: Pool<Sqlite>) {
        let check: (String, u8) = sqlx::query_as(
            "SELECT title, category_id FROM Todo where id = 4",
        )
        .fetch_one(&db)
        .await
        .unwrap();

        assert_eq!(check, ("todo_4".to_owned(), 1));

        let res = update_one_dynmaic(
            State(db.clone()),
            Path("todo".to_string()),
            Json(
                from_value(json!({
                    "id": 4,
                    "partial": {
                        "title": ["set" , "new_title"]
                    },
                    "relations": {
                        "category": { "set": null },
                        "tag": [
                            { "remove_link": 3 },
                            { "set_link": 2 },
                        ],
                    }
                }))
                .unwrap(),
            ),
        )
        .await
        .unwrap();

        let check: (String, Option<i64>) = sqlx::query_as(
            "SELECT title, category_id FROM Todo where id = 4",
        )
        .fetch_one(&db)
        .await
        .unwrap();

        assert_eq!(check, ("new_title".to_owned(), None));

        let check: Vec<(i64,)> = sqlx::query_as(
            "SELECT tag_id FROM TodoTag where todo_id = 4",
        )
        .fetch_all(&db)
        .await
        .unwrap();

        pretty_assertions::assert_eq!(check, vec![(2,)]);

        pretty_assertions::assert_eq!(
            serde_json::to_value(res.0).unwrap(),
            json! {{
                "attr": {
                    "description": null,
                    "done": false,
                    "title": "new_title",
                },
                "id": 4,
                "relations": {
                    "category": null,
                    "tag": [2]
                }
            }}
        );
    }

    async fn test_insert_one(db: Pool<Sqlite>) {
        let res = insert_one_dynamic(
            State(db.clone()),
            Path("todo".to_string()),
            Json(
                from_value(json!({
                    "input": {
                        "title": "new_title",
                        "done": true,
                        "description": "description"
                    },
                    "relation": {
                        "category": {
                            "set_id_to_and_populate": 3
                        },
                        "tag": {
                            "set_id_to_and_populate": [1,2]
                        },

                    }
                }))
                .unwrap(),
            ),
        )
        .await
        .unwrap();

        pretty_assertions::assert_eq!(
            serde_json::to_value(res.0).unwrap(),
            json! {{
                "attr": {
                    "description": "description",
                    "done": true,
                    "title": "new_title",
                },
                "id": 6,
                "relations": {
                    "category": {
                        "id": 3,
                        "attr": {
                            "cat_title": "category_3",
                        }
                    },
                    "tag": [
                        {
                          "id": 1,
                          "attr": { "tag_title": "tag_1" }
                        },
                        {
                          "id": 2,
                          "attr": { "tag_title": "tag_2" }
                        },
                    ]
                }
            }}
        );
    }

    async fn test_insert_one_proof_of_concept(_: Pool<Sqlite>) {
        // there is `todo!()` in exec_op
        catch_unwind(|| {
            // this is just to make sure it compiles
            Box::new(async {
                let op = insert_one(Todo {
                    title: todo!(),
                    done: todo!(),
                    description: todo!(),
                })
                .link_data(link_id(
                    PhantomData::<Tag>,
                    vec![3, 4],
                ))
                .link_id::<Tag, _>(vec![3, 4])
                .link_id::<Category, _>(3)
                .exec_op(todo!())
                .await;

                assert_eq!(
                    op,
                    Ok(GetOneOutput {
                        id: todo!(),
                        attr: todo!(),
                        links: TupleAsMap((
                            vec![],
                            vec![],
                            SimpleOutput {
                                id: todo!(),
                                attr: todo!()
                            }
                        ))
                    })
                );
            });
        });
    }
}
