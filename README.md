Customization, Performant CMS written in rust.

this is a going effort; it is not polished project, I think I have done the core part of the cms. 

my focus now is to make an admit UI and monetize the project by providing a cloud options, I'm looking for sponsership to dedicate more time on the project, if you are interested please contact me at karam.barakat.99@gmail.com

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

axum::Router::new()
    .route("/", get(|| async { "Server is running" }))
    .nest("/todo", Todo::router())
    .nest("/category", Category::router())
    .with_state(sqlx_db_conn);
```

for all feature supported I have an example at client/examples/todo_app

I also have low-level API from queries_for_sqlx crate that makes it easy to build something custom (see client/examples/easy_to_customize):

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
            (
                "Category", 
                Output { 
                    id: 2, 
                    attributes: Todo { title: "hi" }, 
                    relations: () 
                }
            )
        )
    }
);

```

# Plugin System
I'm working on a "Modular" plugin system inspired by Nvim plugins. the idea revolves arout the crate `inventory` the idea, each plugin is jsut an entry in Cargo.toml, and they export inventory items that define exaclty how they wish to be configured.

for example, the migration in this crate is completely separate unit from the rest of the codebase, downstream crates can submit `dyn Migrate` that configure how the migration is run. same thing with 'entities' unit and 'axum' unit.

this way there would be no such thing as 'core plugins', the cms will ship with default plugins and default entrypoint, but that just a code you can replace completely with something else, and the cms serve as just a code management tool.
