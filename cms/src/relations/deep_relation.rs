use queries_for_sqlx::prelude::*;
use sqlx::{Pool, Row, Sqlite};
use std::marker::PhantomData;

use sqlx::sqlite::SqliteRow;

use crate::{
    operations::{select_one::GetOneOutput, SimpleOutput},
    queries_bridge::SelectSt,
    traits::Collection,
    tuple_index::{tuple_as_map::TupleElementKey, TupleAsMap},
};

use super::{
    optional_to_many::OptionalToMany,
    optional_to_many_inverse::OptionalToManyInverse,
    prelude::GetOneWorker, LinkData, Linked, Relation,
};

pub struct DeepPopulate<OR, D> {
    original_rel: OR,
    deep: PhantomData<D>,
}

pub struct OnlyId<To> {
    _pd: PhantomData<To>
}

impl<B> Relation<B> {
    // todo
    pub fn only_id(self) -> OnlyId<B> {
        OnlyId {
            _pd: PhantomData
        }
    }
    pub fn deep_populate<T>(self) -> DeepPopulate<Self, (T,)> {
        DeepPopulate {
            original_rel: self,
            deep: PhantomData,
        }
    }
}

/// this is limited deep population ment to domenstrate how
/// to do deep population, it needs a lot of work to become
/// usefull
pub struct DeepWorker<F, T, TD, S1, S2> {
    spec1: S1,
    spec2: S2,
    _pd: PhantomData<(F, T, TD)>,
}

impl<From, To, Deep> LinkData<From>
    for DeepPopulate<Relation<To>, (Deep,)>
where
    From: Linked<To>,
    To: Linked<
        Deep,
        // for now lets limit this example to optional_to_many relations
        Spec = OptionalToManyInverse,
    >,
{
    type Worker = DeepWorker<
        From,
        To,
        Deep,
        From::Spec,
        To::Spec,
    >;
    fn init(self) -> Self::Worker {
        DeepWorker {
            spec1: From::spec(),
            spec2: To::spec(),
            _pd: PhantomData,
        }
    }
}
impl<T, F> TupleElementKey for GetOneOutput<T, F> {
    fn key() -> &'static str {
        todo!()
    }
}

impl<FromTodo, ToCat> GetOneWorker
    for DeepWorker<
        FromTodo,
        ToCat,
        FromTodo,
        OptionalToMany,
        OptionalToManyInverse,
    >
where
    FromTodo: Linked<ToCat> + Send + Sync,
    ToCat: Linked<
            FromTodo,
            // for now lets limit this example to optional_to_many relations
            Spec = OptionalToManyInverse,
        > + Send
        + Sync,
    <FromTodo as Linked<ToCat>>::Spec: Send + Sync,
    // DeepTodo: Send + Sync,
    FromTodo: Collection<Sqlite>,
    ToCat: Collection<Sqlite>,
{
    type Inner = (
        // FromTodo:id
        Option<(i64, ToCat)>,
        // deep populate
        Vec<SimpleOutput<FromTodo>>,
    );

    type Output = Option<
        GetOneOutput<ToCat, (Vec<SimpleOutput<FromTodo>>,)>,
    >;

    fn on_select(
        &self,
        data: &mut Self::Inner,
        st: &mut SelectSt<Sqlite>,
    ) {
        st.left_join(join {
            on_table: ToCat::table_name().to_string(),
            on_column: "id".to_string(),
            local_column: self.spec1.foriegn_key.to_string(),
        });
        st.select_scoped(
            FromTodo::table_name().to_string(),
            self.spec1.foriegn_key.clone(),
        );
        ToCat::on_select(st);
    }

    fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
        *&mut data.0 = Some((
            row.get("local_id"),
            ToCat::from_row_scoped(row),
        ));
    }

    fn sub_op<'this>(
        &'this self,
        data: &'this mut Self::Inner,
        pool: Pool<Sqlite>,
    ) -> impl std::future::Future<Output = ()> + Send + 'this
    {
        async move {
            let todo = FromTodo::table_name();
            let category = ToCat::table_name();
            if let Some(deep) = &mut data.0 {
                // todo: use queries_for_sqlx
                let ret = sqlx::query(&format!(
                    "
SELECT {todo}.id as local, {} FROM {category}
RIGHT JOIN {todo} ON {todo}.{} = {category}.id
WHERE {category}.id = {}",
                    FromTodo::members()
                        .into_iter()
                        .zip(FromTodo::members_scoped())
                        .map(|(no_scope, scope)| format!(
                            "{todo}.{no_scope} as {scope}"
                        ))
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.spec1.foriegn_key,
                    deep.0.clone()
                ))
                .fetch_all(&pool.clone())
                .await
                .unwrap();

                for one in ret {
                    data.1.push(SimpleOutput {
                        id: one.get("local"),
                        attr: FromTodo::from_row_scoped(&one),
                    })
                }
            }
        }
    }

    fn take(self, data: Self::Inner) -> Self::Output {
        let deep = data.1;

        data.0.map(move |(id, attr)| GetOneOutput {
            id,
            attr,
            links: (deep,),
        })
    }
}
