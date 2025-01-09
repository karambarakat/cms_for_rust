#![allow(unused)]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use case::CaseExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::{sqlite::SqliteRow, Row};
use std::{future::Future, marker::PhantomData};

use queries_for_sqlx::{
    ident_safety::{define_schema, PanicOnUnsafe},
    prelude::*,
    quick_query::QuickQuery,
};
use sqlx::{Database, Pool, Sqlite};

use crate::{
    build_tuple::BuildTuple,
    // coll::client::{Category, Tag, Todo},
    collections::{
        error::{self, CollectionError},
        Collection,
    },
    orm::{prelude::*, relations::relation},
    tuple_index::{tuple_as_map::TupleElementKey, TupleAsMap},
};

use super::{
    dynamic_schema::{DynGetOneWorker, COLLECTIONS}, relations::Relation, worker,
};

pub struct GetOne<C, L, Q> {
    links: L,
    _pd: PhantomData<(C, L, Q)>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Output<C> {
    pub id: i64,
    pub attr: C,
}

impl<C: Collection> TupleElementKey for Output<C> {
    fn key() -> &'static str {
        C::table_name1()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OutputLinked<C, D> {
    pub id: i64,
    pub attr: C,
    pub links: D,
}

pub type OuputDynamic = OutputLinked<Value, Map<String, Value>>;

pub fn get_one<C>() -> GetOne<C, (), ()> {
    GetOne {
        links: (),
        _pd: PhantomData,
    }
}

#[derive(Debug, Deserialize)]
pub struct InputGetOne {
    pub query: Map<String, Value>,
    pub relation: Map<String, Value>,
    pub id: i64,
}

pub fn get_one_dynamic(
    db: State<Pool<Sqlite>>,
    collection_name: Path<String>,
    input: Json<InputGetOne>,
) -> impl Future<
    Output = Result<Json<OuputDynamic>, CollectionError>,
> + Send {
    async move {
        let collection_gaurd = COLLECTIONS.read().await;
        // let relation_gaurd =
        let collection = collection_gaurd
            .get(&collection_name.0.to_camel())
            .ok_or(
                error::to_refactor(StatusCode::NOT_FOUND)
                    .for_dev(
                        "entity does not exist".to_string(),
                    ),
            )?;
        // let rel_gaurd = RELATION
        let rel: Vec<
            Box<dyn DynGetOneWorker>
        > = vec![];


        let mut st = stmt::SelectSt::init(
            collection.table_name().to_string(),
        );

        collection.on_select(&mut st);
        st.select(col("id".to_string()).alias("local_id"));
        let id = input.0.id;
        st.where_(col("local_id".to_string()).eq(move || id));

        let res = st
            .fetch_optional(&db.0, |r| {
                let attr = collection.on_get(&r);
                let id: i64 = r.get("local_id");

                return Ok(OutputLinked {
                    id,
                    attr,
                    links: Map::default(),
                });
            })
            .await
            .unwrap()
            .ok_or(
                error::to_refactor(StatusCode::NOT_FOUND)
                    .for_dev(
                        "entry with id not found".to_string(),
                    ),
            )?;

        drop(collection_gaurd);

        Ok(Json(res))
    }
}

impl<C, L, Q> GetOne<C, L, Q>
where
    L: BuildTuple,
    Q: BuildTuple,
{
    pub fn link_data<N>(
        self,
        ty: N,
    ) -> GetOne<C, L::Bigger<N::Worker>, Q>
    where
        N: LinkData<C>,
        <N as LinkData<C>>::Worker: GetOneWorker + Send,
    {
        GetOne {
            links: self.links.into_bigger(ty.init()),
            _pd: PhantomData,
        }
    }
    pub fn relation<N>(
        self,
    ) -> GetOne<
        C,
        L::Bigger<<Relation<N> as LinkData<C>>::Worker>,
        Q,
    >
    where
        Relation<N>: LinkData<C, Worker: GetOneWorker + Send>,
    {
        GetOne {
            links: self
                .links
                .into_bigger(Relation(PhantomData).init()),
            _pd: PhantomData,
        }
    }
}

pub trait GetOneWorker: Sync + Send {
    type Inner: Default + Send + Sync;
    type Output;
    fn on_select(
        &self,
        data: &mut Self::Inner,
        st: &mut stmt::SelectSt<
            Sqlite,
            QuickQuery,
            PanicOnUnsafe,
        >,
    );
    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow);
    fn sub_op<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl Future<Output = ()> + Send + 'this;
    fn take(self, data: Self::Inner) -> Self::Output;
}

impl<C, R, Q> GetOne<C, R, Q>
where
    C: Collection,
    R: GetOneWorker + Send + Sync,
{
    pub async fn exec_op(
        mut self,
        db: Pool<Sqlite>,
    ) -> Option<OutputLinked<C, TupleAsMap<R::Output>>> {
        let mut st =
            stmt::SelectSt::init(C::table_name1().to_string());

        st.select(col("id".to_string()).alias("local_id"));
        C::on_select1(&mut st);

        let mut worker_data = R::Inner::default();

        self.links.on_select(&mut worker_data, &mut st);

        let res = st
            .fetch_optional(&db, |r| {
                let id: i64 = r.get("local_id");
                let attr = C::on_get1(&r);
                self.links.from_row(&mut worker_data, &r);
                Ok(OutputLinked {
                    id,
                    attr,
                    links: (),
                })
            })
            .await
            .unwrap()?;

        self.links.sub_op(&mut worker_data, db).await;
        let data = self.links.take(worker_data);

        return Some(OutputLinked {
            id: res.id,
            attr: res.attr,
            links: TupleAsMap(data),
        });
    }
}

#[cfg(test)]
mod test {
    use queries_for_sqlx::ident_safety::define_schema;
    use sqlx::{Pool, Sqlite};

    use crate::{
        client_example::{Category, Tag, Todo},
        orm::{
            relations::relation,
            select_one::{get_one, Output, OutputLinked},
        },
        tuple_index::TupleAsMap,
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
                FOREIGN KEY (category_id) REFERENCES Category (id)
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
                FOREIGN KEY (todo_id) REFERENCES Todo (id),
                FOREIGN KEY (tag_id) REFERENCES Tag (id)
            );

            INSERT INTO Tag (tag_title) VALUES ('tag_1'), ('tag_2'), ('tag_3');
            INSERT INTO Category (cat_title) VALUES ('category_1'), ('category_2'), ('category_3');

            INSERT INTO Todo (title, done, category_id) VALUES
                ('todo_1', 1, 3),
                ('todo_2', 0, 3),
                ('todo_3', 1, NULL),
                ('todo_4', 0, 1),
                ('todo_5', 1, NULL);
    
            INSERT INTO TodoTag (todo_id, tag_id) VALUES
                (1, 1), (1, 2),
                (2, 1), (2, 2),
                (3, 1), (3, 2),
                (4, 1), (4, 2),
                (5, 1), (5, 2);
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

        let res = get_one::<Todo>()
            .relation::<Tag>()
            // .relation::<Category>()
            .exec_op(db)
            .await;

        // let res = serde_json::to_string_pretty(&res).unwrap();

        pretty_assertions::assert_eq!(
            res,
            Some(OutputLinked {
                id: 1,
                attr: Todo {
                    title: "todo_1".to_string(),
                    done: true,
                    description: None,
                },
                links: TupleAsMap((
                    vec![
                        Output {
                            id: 1,
                            attr: Tag {
                                tag_title: "tag_1".to_string()
                            }
                        },
                        Output {
                            id: 2,
                            attr: Tag {
                                tag_title: "tag_2".to_string()
                            }
                        }
                    ],
                    // Some(Output {
                    //     id: 3,
                    //     attr: Category {
                    //         cat_title: "category_3".to_string()
                    //     }
                    // })
                ))
            })
        );
    }
}
