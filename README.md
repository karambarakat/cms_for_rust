# Customizable, Performant, and Convenient CMS written in rust.

this is a going effort; it is not polished project, I think I have done the core part of the cms.

my focus now is to make an admit UI and monetize the project by providing a cloud options, I'm looking for sponsership to dedicate more time on the project, if you are interested please contact me at k99.barakat@gmail.com

# Intoduction

```rust
use cms_for_rust::schema_prelude::*;

#[standard_collection]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[standard_collection]
pub struct Category {
    pub title: String,
}

#[standard_collection]
pub struct Tag {
    pub title: String,
}

relation! { optional_to_many Todo Category }
relation! { many_to_many Todo Tag }
```

Just by defining this schema, you have a full CRUD HTTP server, automatic migration, an admin UI (coming) and an ORM-like API.

# Why Not SeaORM
I worked with SeaORM for a while, before I decided to do this. there are many issues I tried to solve here.

I think ORMs in general focus on the wrong level of abstract, here I'm trying to build a Content Management System, but there is an ORM-like API too.

To summarize I think this crate is more flexible and convienient than SeaORM, to list few differences:

1. Out of the box migration. (see Migration)
2. support multiple relation population and/or deep population. (see Deep Relation)
3. Out of the box Axum server. (see HTTP Server)

# All Features

## Migration

migaration can be done with one line of code:

```rust
cms_for_rust::migration::run_migrate(&sqlx_db_conn).await;
```

this will look at the schema defined above and create the necessary tables, keys, etc.


## HTTP Server

you can run fully-functional CRUD REST API with:

```rust
use cms_for_rust::axum_router::collections_router;
use cms_for_rust::auth::auth_router;

let sqlx_db_conn = Pool::<Sqlite>::connect("sqlite::memory:")
    .await
    .unwrap();

cms_for_rust::migration::run_migration(&sqlx_db_conn).await;

let app = axum::Router::new()
    .route("/", get(|| async { "Server is running" }))
    .nest("/collections", collections_router())
    .nest("/auth", auth_router())
    .with_state(sqlx_db_conn);

let listner = tokio::net::TcpListener::bind("0.0.0.0:3000")
    .await
    .unwrap();
```

the authentication strategy is basic for now -- reading is public, and writing is protected via Bearer JWT token for _super_users table entries. In the future, I will make a more customizable permission plugin.

checkout `all http REST features` section for all supported features.

## ORM API

you have access to ORM-like client that supports populating relations:

```rust
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
```

## Deep Relation
in addition to multiple relation domenstrated above, you can do deep relations like this:
```rust
    async fn test_deep_populate(db: Pool<Sqlite>) {
        let res = get_one::<Todo>()
            .relation_as::<Category, _, _>(|r| {
                // in theory you can do multiple deep relations and/or go deeper
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
```

## all http REST features

`POST /collections/{collection}/get_one`
```rust
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
    }
```

`POST /collections/{collection}/get_all`
```rust
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
```

`POST /collections/{collection}/insert_one`
```rust
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
```

`POST /collections/{collection}/update_one`
```rust
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
```

`POST /collections/{collection}/delete_one`
```rust
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
```

for all feature supported all tests are at cms/src/operations

## Dynamic query builder (low level customization)

The CMS is built on top of other crate I have called `queries_for_sqlx`, this crate is meant to be an extention for the famouse crate `sqlx`, and closely mimics sqlx and how different databases behave.

here is a summary of the features:
1. extentions of sqlx to build queries dynamicly
2. protect against SQL Injection
3. handle database that support `?` syntax (unlike Sqlite which support `$1` for binding), see '/src/positional_query.rs' for full example of how that was achevied.

I need to elaborate more on this crate, but that crate was not the purpose for this project.



*binding values example*
```rust
let mut st = select_st::init("Todo");

// SELECT * FROM Todo
st.select(col("id"));
st.select(col("title"));
st.where_(col("id").eq(3));

// the sql query: "SELECT id, title WHERE id = $1"
#[derive(FromRow)]
struct Todo {
    id: i32,
    title: String
}

let res = st.fetch(&db.0, |row| Todo::from_row(&row))
    .await
    .unwrap()

// test the response
assert_eq!(
    res,
    Todo { id: 3, title: "hi".to_string()},
);
```

*How databases that supports `?` is different from ones that supports `$1`*

take this as an example
```
let mut st = SelectSt("Todo");

st.select(col("id"));

st.limit(3);
st.where_(col("title").eq("common_title"));
```

if you are using Sqlite or Postgres, rust will infer that generic and produce the following psuedocode:

```
let mut buffer = Default::default();
let mut query = String::default();

// the same order as called
buffer.add(3);
let limit_str = "$1";

buffer.add("common_title");
let where_str = "$2";

query.push_str("SELECT id FROM Todo WHERE id = ");

// where clause comes first
query.push_str(where_str);

query.push_str(" LIMIT ");

query.push_str(limit_str);

query.push_str(";");
```

buf if you are using MySql, rust will infer that and use 'PositionQuery', which will put the values on the heap temporarily until the query is built and bind those in the order they should be. this corresponds to the following psuedocode:

```
let mut tmp_buffer: Vec<Box<dyn PsuedoTrait>> = Default::default();
let mut buffer = Default::default();
let mut query = String::default();

// the same order as called
tmp_buffer.push(Box::new(3));
let limit_index = 0;

tmp_buffer.push(Box::new("common_title"));
let where_index = 1;

query.push_str("SELECT id FROM Todo WHERE id = ");

// where clause comes first
query.push_str("?");
buffer.add(*tmp_buffer[where_index].take());

query.push_str(" LIMIT ");

query.push_str("?");
buffer.add(*tmp_buffer[limit_index].take());

query.push_str(";");
```

there is a performance cost of putting things at the heap, but this is unavoidable if you want rust to figure out the correct order at a runtime (TBH I hope sql uses BSON query, its easy to validate and protect against injection, and there would be no need for this whole thing).

* easy to come with custome API Endpoint *
```rust
axum::Router::new()
    .route("/", get(|| async {
        let mut st = select_st::init("Todo");

        // SELECT * FROM Todo
        st.select(all_columns());

        // This will use information from sqlx::Type to figure out
        // the type of the output on the fly
        st.fetch_all(&db.0, row_to_json_cached::sqlite_row())
            .await
            .unwrap()
    }));

// test the response
assert_eq!(
    response_body,
    json!([
        {"id": 3, "title": "hi", "done": true, "description": "hello"},
        {"id": 4, "title": "bye", "done": false, "description": "goodbye"}
    ])
);
```

# Workspace Structure
there are two core crates in this workspace:
1.`queries_for_sqlx` low-level query builder, extention for sqlx
2.`cms_for_rust` high level CMS/ORM crate, built on top of the former.

the idea behind this seperation is that I realize by working with SeaORM that convenient API, and performent API are inherently irreconsible. here is key aspect:
    - performence: `queries_for_sqlx` maximizes performant and it is exteremely optimizeable and completly generic.
    - convience: on the other hand `cms_for_rust` maximizes convenience at the expense of performance or I have some opinionated API. If I figured out the perfect API of something I will move it down to `queries_for_sqlx`.
    - unopinionated: every thing inside `queries_for_sqlx` closely mimics the underlying database, where if there is anything opinionated it will be in `cms_for_rust`.
    - strict semver policy: there will be no breaking changes in `queries_for_sqlx` beyond v0.1.0, as long as sqlx doesn't have its v1 this will not release v1.

# Plugin System
I'm working on a "Modular" plugin system, I'm inspired by Nvim plugins where each plugin exports functions, and it's up to you (or other plugins) to use them as they wish.

The idea revolves around the crate `inventory`; each plugin defines how they wish to be customized by exporting `impl Collect`.

This way there would be no such thing as 'core plugins'. Each plugin is just an entry in Cargo.toml that submits inventory items.

Every CMS have one enyry-point "fn main", but this is 100% your code. I'm thinking of shipping a default entry-point that includes all "built-in" plugins and provides examples of more customizable ones.

For example, the migration in this crate is a completely separate unit from the rest of the codebase; downstream crates can submit `dyn Migrate` that configures how the migration is run.

I also dislike the idea of a headless CMS; providing a basic frontend, as long as it's not built-in and somehow customizable, is great. Anyone who doesn't like it can opt out because it is not built-in and you can use other plugins with different ecosystems around it.
