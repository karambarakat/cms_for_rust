# Customization, Performant CMS written in rust.

this is a going effort; it is not polished project, I think I have done the core part of the cms. 

my focus now is to make an admit UI and monetize the project by providing a cloud options, I'm looking for sponsership to dedicate more time on the project, if you are interested please contact me at k99.barakat@gmail.com

# Intoduction

```rust
use cms_for_rust::schema_prelude::*;

schema! { db = "sqlx::Sqlite", }

#[entity]
pub struct Todo {
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

#[entity]
pub struct Category {
    pub title: String,
}

#[entity]
pub struct Tag {
    pub title: String,
}

relations! { optional_to_many Todo Category }
relations! { many_to_many Todo Tag }
```

and just like that you a full CRUD http server and automatic migration:

```rust
use cms_for_rust::axum_router::AxumRouter;

cms_for_rust::migration::migrate(sqlx_db_conn.clone()).await;


let app = axum::Router::new()
    .route("/", get(|| async { "Server is running" }))
    .nest("/todo", Todo::router())
    .nest("/category", Category::router())
    .with_state(sqlx_db_conn);

// example for request

// load_some_dumpy_data()

let mut res = app
    .oneshot(
        Request::builder()
            .uri("/todo")
            .method(Method::GET)
            .json_body(json!({
                "query": {
                    // support pagination
                    "pagination": {
                        "page_size": 2,
                        "page_shift": 2,
                    },
                },
                "relations": {
                    // populate both category and tags,
                    // this will do a left join for category 
                    // and spin another request for tags
                    "category": { "id": true, "attributes": true },
                    "tags": { "id": true, "attributes": true },
                },
            }))
            .expect("request"),
    )
    .await
    .expect("oneshot");

let res = res.into_json().await.unwrap();

expect(&res).to_be(&json!({
    "meta": { "page_number": 2 },
    "data": [
        {
            "id": 3,
            "attributes": {
                "title": "todo_3",
                "done": false,
                "description": "blue",
            },
            "relations": {
                "category": {
                    "id": 3,
                    "attributes": { "title": "cat_3" }
                },
                "tags": [
                    { "id": 1, "attributes": { "title": "tag_1" } },
                    { "id": 2, "attributes": { "title": "tag_2" } },
                ]
            }
        },
        {
            "id": 4,
            "attributes": {
                "title": "todo_4",
                "done": true,
                "description": "yellow",
            },
            "relations": {
                "category": null,
                "tags": [
                    { "id": 1, "attributes": { "title": "tag_1" } },
                ]
            },
        },
    ]
}));
```

for all feature supported I have an example at client/examples/todo_app

to build something custom, I have low-level API from queries_for_sqlx crate (see client/examples/easy_to_customize):

```rust
axum::Router::new()
    .route("/", get(|| async {
        let mut st = select_st::<_, Todo>();

        st.select(all_columns());

        st.fetch_all(&db.0, row_to_json_cached::sqlite_row())
            .await
            .unwrap()
    }));

// test the response
assert_eq!(
    response_body,
    json!([
        {"title": "hi", "done": true, "description": "hello"},
        {"title": "bye", "done": false, "description": "goodbye"}
    ])
);
```

in the future I will have ORM-like API, the core of this code is already done I just need time to implement it. 

```rust
let res = select_st::new::<Todo>()
    .by_id(3)
    .relation::<Category>()
    .fetch_one(&db)
    .await
    .unwrap();

assert_eq!(
    res,
    Output {
        id: 3,
        attributes: Todo {
            title: "hi", 
            done: true,
            description: "hello",
        },
        relations: (
            Output {
                id: 2,
                attributes: Category { title: "cat_1" },
                relations: ()
            }
        )
    }
);

```

# Workspace Structure
there are two core crates in this workspace:
1.`queries_for_sqlx` low-level query builder, extention for sqlx
2.`cms_for_rust` high level CMS/ORM crate, depends on the former.

the idea behind this seperation is that I realize by working with SeaORM that convenient API and performent API lives in opposite ends of a spectrum. here is key aspect:
    - performence: `queries_for_sqlx` maximizes performant and it is exteremely optimizeable and completly generic.
    - convience: `cms_for_rust` maximizes convenience when performance might be impacted or I have some opinionated API, if I figured out the perfect API of something I will move it down to `queries_for_sqlx`. 
    - unopinionated: every thing inside `queries_for_sqlx` is closely mimic the underlying database
    - strict semver policy: there will be no breaking changes in `queries_for_sqlx` beyond v0.1.0, as long as sqlx don't have their v1 this will stay in v0,.

# Plugin System
I'm working on a "Modular" plugin system inspired by Nvim plugins. the idea revolves arout the crate `inventory` the idea, each plugin is jsut an entry in Cargo.toml, and they export inventory items that define exaclty how they wish to be configured.

this way there would be no such thing as 'core plugins', the cms will ship with default plugins and default entrypoint, but that just a code you can replace completely with something else, and the cms serve as just a code management tool.

for example, the migration in this crate is completely separate unit from the rest of the codebase, downstream crates can submit `dyn Migrate` that configure how the migration is run. same thing with 'entities' unit and 'axum' unit.

