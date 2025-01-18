use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
pub mod deep_relation;
pub mod many_to_many;
pub mod one_to_many;
pub mod one_to_many_inverse;
pub mod optional_to_many;
pub mod optional_to_many_inverse;

pub mod prelude {
    pub use super::{LinkSpec, Linked};
    pub use crate::dynamic_schema::{
        CompleteRelationForServer, DynGetOneWorker,
        DynamicWorker,
    };
    pub use crate::operations::select_one::GetOneWorker;
    pub use crate::relations::RelationWorker;
    pub use crate::traits::Collection;
    pub use queries_for_sqlx::ident_safety::PanicOnUnsafe;
    pub use queries_for_sqlx::prelude::*;
    pub use queries_for_sqlx::quick_query::QuickQuery;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::Value;
    pub use sqlx::sqlite::SqliteRow;
    pub use sqlx::Row;
    pub use sqlx::{Pool, Sqlite};
    pub use std::future::Future;
    pub use std::marker::PhantomData;
    pub use std::ops::Not;
    pub use std::sync::Arc;
}

pub trait Linked<To> {
    type Spec: LinkSpec;
    fn spec() -> Self::Spec;
}

pub trait LinkData<F> {
    type Worker;
    fn init(self) -> Self::Worker;
}

pub trait LinkSpec {}

pub trait LinkSpecCanInsert: LinkSpec {
    type Input;
}
pub trait LinkSpecCanUpdate: LinkSpec {
    type Input;
}

pub struct Relation<T>(pub(crate) PhantomData<T>);

pub fn relation<To>() -> Relation<To> {
    Relation(PhantomData)
}

impl<FromTodo, ToCat> LinkData<FromTodo> for Relation<ToCat>
where
    FromTodo: Linked<ToCat>,
{
    type Worker =
        RelationWorker<FromTodo::Spec, FromTodo, ToCat>;
    fn init(self) -> Self::Worker {
        RelationWorker {
            rel_spec: FromTodo::spec(),
            _pd: PhantomData,
        }
    }
}

pub struct RelationWorker<S, F, T> {
    pub(crate) rel_spec: S,
    pub(crate) _pd: PhantomData<(F, T)>,
}

pub struct LinkId<T, L> {
    pub(crate) id: T,
    pub(crate) _pd: PhantomData<L>,
}

pub fn link_id<I, To>(
    to: PhantomData<To>,
    input: I,
) -> LinkId<I, To> {
    LinkId { id: input, _pd: to }
}

impl<B, T> LinkData<B>
    for LinkId<
        // what the spec require the input to be
        <<B as Linked<T>>::Spec as LinkSpecCanInsert>::Input,
        T,
    >
where
    B: Linked<T>,
    B::Spec: LinkSpecCanInsert,
{
    type Worker = LinkIdWorker<
        // what the spec require the input to be
        B,
        T,
        <B as Linked<T>>::Spec,
        <<B as Linked<T>>::Spec as LinkSpecCanInsert>::Input,
    >;

    fn init(self) -> Self::Worker {
        LinkIdWorker {
            input: self.id,
            spec: B::spec(),
            _pd: PhantomData,
        }
    }
}

pub struct LinkIdWorker<B, T, Spec, I> {
    pub(crate) input: I,
    pub(crate) spec: Spec,
    pub(crate) _pd: PhantomData<(B, T)>,
}

pub struct ManyWorker<B, T, Spec> {
    pub(crate) spec: Spec,
    pub(crate) _pd: PhantomData<(B, T)>,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum UpdateIdInput {
    remove_link(i64),
    set_link(i64),
}

pub struct UpdateId<T, L> {
    pub(crate) id: T,
    pub(crate) _pd: PhantomData<L>,
}

pub fn update_id<I, To>(
    to: PhantomData<To>,
    input: I,
) -> UpdateId<I, To> {
    UpdateId { id: input, _pd: to }
}

impl<B, T> LinkData<B>
    for UpdateId<
        // what the spec require the input to be
        <<B as Linked<T>>::Spec as LinkSpecCanUpdate>::Input,
        T,
    >
where
    B: Linked<T>,
    B::Spec: LinkSpecCanUpdate,
{
    type Worker = UpdateIdWorker<
        // what the spec require the input to be
        B,
        T,
        <B as Linked<T>>::Spec,
        <<B as Linked<T>>::Spec as LinkSpecCanUpdate>::Input,
    >;

    fn init(self) -> Self::Worker {
        UpdateIdWorker {
            input: self.id,
            spec: B::spec(),
            _pd: PhantomData,
        }
    }
}

pub struct UpdateIdWorker<B, T, Spec, I> {
    pub(crate) input: I,
    pub(crate) spec: Spec,
    pub(crate) _pd: PhantomData<(B, T)>,
}
