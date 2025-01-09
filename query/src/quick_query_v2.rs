/// I'm trying to make a query that behave like PositionalStaticBuffer
/// for MySql and NamedBorrowedBuffer for Sqlite and Postgres
/// I reached a deadend
/// and I think I cannot do that without specialization
use sqlx::{
    prelude::Type, Database, Encode, MySql, Postgres, Sqlite,
};

use crate::{
    ident_safety::PanicOnUnsafe, named_buffer::NamedBorrowedBuffer, positional_buffer::PositionalStaticBuffer, sql_part::{
        AcceptToSqlPart, ColumnToSqlPart, ConstraintToSqlPart,
        ToSqlPart, WhereItemToSqlPart,
    }, Accept, Constraint, Query, SchemaColumn, WhereItem
};

pub struct QuickQuery;

// macro_rules! impl_for_named_static_buffer {
//     ($db:ident) => {
//
// impl Query<$db> for QuickQuery
//     {
//     type SqlPart =
//         <NamedBorrowedBuffer<'static> as Query<$db>>::SqlPart;
//     type Context1 = <NamedBorrowedBuffer<'static> as Query<
//         $db,
//     >>::Context1;
//     type Context2 = <NamedBorrowedBuffer<'static> as Query<
//         $db,
//     >>::Context2;
//     fn build_sql_part_back(
//         ctx2: &mut Self::Context2,
//         from: Self::SqlPart,
//     ) -> String {
//         <NamedBorrowedBuffer<'static> as Query<$db>>::build_sql_part_back(ctx2, from)
//     }
//     type Output =
//         <NamedBorrowedBuffer<'static> as Query<$db>>::Output;
//     fn build_query(
//         ctx1: Self::Context1,
//         f: impl FnOnce(&mut Self::Context2) -> String,
//     ) -> (String, Self::Output) {
//         <NamedBorrowedBuffer<'static> as Query<$db>>::build_query(ctx1, f)
//     }
// }
//
//     };
// }
//
// macro_rules! impl_for_positional_static_buffer {
//     ($db:ident) => {
//
// impl Query<$db> for QuickQuery {
//     type SqlPart =
//         <PositionalStaticBuffer as Query<$db>>::SqlPart;
//     type Context1 =
//         <PositionalStaticBuffer as Query<$db>>::Context1;
//     type Context2 =
//         <PositionalStaticBuffer as Query<$db>>::Context2;
//     fn build_sql_part_back(
//         ctx2: &mut Self::Context2,
//         from: Self::SqlPart,
//     ) -> String {
//         <PositionalStaticBuffer as Query<$db>>::build_sql_part_back(ctx2, from)
//     }
//     type Output =
//         <PositionalStaticBuffer as Query<$db>>::Output;
//     fn build_query(
//         ctx1: Self::Context1,
//         f: impl FnOnce(&mut Self::Context2) -> String,
//     ) -> (String, Self::Output) {
//         <PositionalStaticBuffer as Query<$db>>::build_query(
//             ctx1, f,
//         )
//     }
// }
//     };
// }
//
// macro_rules! impl_for_named_static_buffer2 {
//     ($db:ident) => {
// impl<ToBeAccepted, T> Accept<ToBeAccepted, $db>
//     for QuickQuery
// where
//     T: for<'e> Encode<'e, $db> + Type<$db> + Send + 'static,
//     ToBeAccepted: FnOnce() -> T + 'static,
// {
//     fn accept(
//         this: ToBeAccepted,
//         ctx1: &mut Self::Context1,
//     ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
//     {
//         <NamedBorrowedBuffer<'static> as Accept<
//             ToBeAccepted,
//             $db,
//         >>::accept(this, ctx1)
//     }
// }
//
// impl<T> ToSqlPart<QuickQuery, $db> for WhereItemToSqlPart<T>
// where
//     Self: ToSqlPart<NamedBorrowedBuffer<'static>, $db>,
//     T: WhereItem<$db, QuickQuery>,
//     QuickQuery: Query<
//         $db,
//         SqlPart = <NamedBorrowedBuffer<'static> as Query<
//             $db,
//         >>::SqlPart,
//         Context1 = <NamedBorrowedBuffer<'static> as Query<
//             $db,
//         >>::Context1,
//     >,
// {
//     fn to_sql_part(
//         self,
//         ctx: &mut <QuickQuery as Query<$db>>::Context1,
//     ) -> <QuickQuery as Query<$db>>::SqlPart {
//         <Self as ToSqlPart<
//             NamedBorrowedBuffer<'static>,
//             $db,
//         >>::to_sql_part(self, ctx)
//     }
// }
//
// impl<T> ToSqlPart<QuickQuery, $db> for AcceptToSqlPart<T>
// where
//     Self: ToSqlPart<NamedBorrowedBuffer<'static>, $db>,
//     T: Accept<T, $db>,
//     QuickQuery: Query<
//         $db,
//         SqlPart = <NamedBorrowedBuffer<'static> as Query<
//             $db,
//         >>::SqlPart,
//         Context1 = <NamedBorrowedBuffer<'static> as Query<
//             $db,
//         >>::Context1,
//     >,
// {
//     fn to_sql_part(
//         self,
//         ctx: &mut <QuickQuery as Query<$db>>::Context1,
//     ) -> <QuickQuery as Query<$db>>::SqlPart {
//         <Self as ToSqlPart<
//             NamedBorrowedBuffer<'static>,
//             $db,
//         >>::to_sql_part(self, ctx)
//     }
// }
//
// impl<T> ToSqlPart<QuickQuery, $db> for ConstraintToSqlPart<T>
// where
//     Self: ToSqlPart<NamedBorrowedBuffer<'static>, $db>,
//     T: Constraint<$db, NamedBorrowedBuffer<'static>>,
//     QuickQuery: Query<
//         $db,
//         SqlPart = <NamedBorrowedBuffer<'static> as Query<
//             $db,
//         >>::SqlPart,
//         Context1 = <NamedBorrowedBuffer<'static> as Query<
//             $db,
//         >>::Context1,
//     >,
// {
//     fn to_sql_part(
//         self,
//         ctx: &mut <QuickQuery as Query<$db>>::Context1,
//     ) -> <QuickQuery as Query<$db>>::SqlPart {
//         <Self as ToSqlPart<
//             NamedBorrowedBuffer<'static>,
//             $db,
//         >>::to_sql_part(self, ctx)
//     }
// }
//
// impl<T> ToSqlPart<QuickQuery, $db> for ColumnToSqlPart<T>
// where
//     Self: ToSqlPart<NamedBorrowedBuffer<'static>, $db>,
//     T: SchemaColumn<$db, NamedBorrowedBuffer<'static>>,
//     QuickQuery: Query<
//         $db,
//         SqlPart = <NamedBorrowedBuffer<'static> as Query<
//             $db,
//         >>::SqlPart,
//         Context1 = <NamedBorrowedBuffer<'static> as Query<
//             $db,
//         >>::Context1,
//     >,
// {
//     fn to_sql_part(
//         self,
//         ctx: &mut <QuickQuery as Query<$db>>::Context1,
//     ) -> <QuickQuery as Query<$db>>::SqlPart {
//         <Self as ToSqlPart<
//             NamedBorrowedBuffer<'static>,
//             $db,
//         >>::to_sql_part(self, ctx)
//     }
// }
//     };
// }
//
// macro_rules! impl_for_positional_static_buffer2 {
//     ($db:ident) => {
//         impl<ToBeAccepted, T> Accept<ToBeAccepted, $db>
//             for QuickQuery
//         where
//             T: for<'e> Encode<'e, $db>
//                 + Type<$db>
//                 + Send
//                 + 'static,
//             ToBeAccepted: FnOnce() -> T + 'static,
//         {
//             fn accept(
//                 this: ToBeAccepted,
//                 ctx1: &mut Self::Context1,
//             ) -> impl FnOnce(&mut Self::Context2) -> String
//                    + 'static
//                    + Send {
//                 <PositionalStaticBuffer as Accept<
//                     ToBeAccepted,
//                     $db,
//                 >>::accept(this, ctx1)
//             }
//         }
//
//         impl<T> ToSqlPart<QuickQuery, $db>
//             for WhereItemToSqlPart<T>
//         where
//             Self: ToSqlPart<PositionalStaticBuffer, $db>,
//             T: WhereItem<$db, QuickQuery>,
//             QuickQuery: Query<
//                 $db,
//                 SqlPart = <PositionalStaticBuffer as Query<
//                     $db,
//                 >>::SqlPart,
//                 Context1 = <PositionalStaticBuffer as Query<
//                     $db,
//                 >>::Context1,
//             >,
//         {
//             fn to_sql_part(
//                 self,
//                 ctx: &mut <QuickQuery as Query<$db>>::Context1,
//             ) -> <QuickQuery as Query<$db>>::SqlPart {
//                 <Self as ToSqlPart<
//                     PositionalStaticBuffer,
//                     $db,
//                 >>::to_sql_part(self, ctx)
//             }
//         }
//
//         impl<T> ToSqlPart<QuickQuery, $db> for AcceptToSqlPart<T>
//         where
//             Self: ToSqlPart<PositionalStaticBuffer, $db>,
//             T: Accept<T, $db>,
//             QuickQuery: Query<
//                 $db,
//                 SqlPart = <PositionalStaticBuffer as Query<
//                     $db,
//                 >>::SqlPart,
//                 Context1 = <PositionalStaticBuffer as Query<
//                     $db,
//                 >>::Context1,
//             >,
//         {
//             fn to_sql_part(
//                 self,
//                 ctx: &mut <QuickQuery as Query<$db>>::Context1,
//             ) -> <QuickQuery as Query<$db>>::SqlPart {
//                 <Self as ToSqlPart<
//                     PositionalStaticBuffer,
//                     $db,
//                 >>::to_sql_part(self, ctx)
//             }
//         }
//
//         impl<T> ToSqlPart<QuickQuery, $db>
//             for ConstraintToSqlPart<T>
//         where
//             Self: ToSqlPart<PositionalStaticBuffer, $db>,
//             T: Constraint<$db, PositionalStaticBuffer>,
//             QuickQuery: Query<
//                 $db,
//                 SqlPart = <PositionalStaticBuffer as Query<
//                     $db,
//                 >>::SqlPart,
//                 Context1 = <PositionalStaticBuffer as Query<
//                     $db,
//                 >>::Context1,
//             >,
//         {
//             fn to_sql_part(
//                 self,
//                 ctx: &mut <QuickQuery as Query<$db>>::Context1,
//             ) -> <QuickQuery as Query<$db>>::SqlPart {
//                 <Self as ToSqlPart<
//                     PositionalStaticBuffer,
//                     $db,
//                 >>::to_sql_part(self, ctx)
//             }
//         }
//
//         impl<T> ToSqlPart<QuickQuery, $db> for ColumnToSqlPart<T>
//         where
//             Self: ToSqlPart<PositionalStaticBuffer, $db>,
//             T: SchemaColumn<$db, PositionalStaticBuffer>,
//             QuickQuery: Query<
//                 $db,
//                 SqlPart = <PositionalStaticBuffer as Query<
//                     $db,
//                 >>::SqlPart,
//                 Context1 = <PositionalStaticBuffer as Query<
//                     $db,
//                 >>::Context1,
//             >,
//         {
//             fn to_sql_part(
//                 self,
//                 ctx: &mut <QuickQuery as Query<$db>>::Context1,
//             ) -> <QuickQuery as Query<$db>>::SqlPart {
//                 <Self as ToSqlPart<
//                     PositionalStaticBuffer,
//                     $db,
//                 >>::to_sql_part(self, ctx)
//             }
//         }
//     };
// }

// impl_for_named_static_buffer!(Sqlite);
// impl_for_named_static_buffer!(Postgres);
// impl_for_positional_static_buffer!(MySql);

/// waiting for rust's specialization to depricate this
pub trait DatabaseSubset: Database {
    type Q: Query<Self>;
    fn accept_closure<T>(
        this: impl FnOnce() -> T + 'static,
        ctx1: &mut <Self::Q as Query<Self>>::Context1,
    ) -> impl FnOnce(
        &mut <Self::Q as Query<Self>>::Context2,
    ) -> String
           + 'static
           + Send
    where
        T: for<'e> Encode<'e, Self>
            + Type<Self>
            + Send
            + 'static,
    {
        |_| todo!()
    }

    fn handle_where_item<T>(
        this: T,
        ctx: &mut <Self::Q as Query<Self>>::Context1,
    ) -> <Self::Q as Query<Self>>::SqlPart
    where
        T: WhereItem<Self, Self::Q, PanicOnUnsafe> + 'static,
        WhereItemToSqlPart<T>: ToSqlPart<Self::Q, Self>;

    fn handle_accept<T>(
        this: T,
        ctx: &mut <Self::Q as Query<Self>>::Context1,
    ) -> <Self::Q as Query<Self>>::SqlPart
    where
        Self::Q: Accept<T, Self>,
        AcceptToSqlPart<T>: ToSqlPart<Self::Q, Self>,
    {
        todo!()
    }
}

// impl<S, ToBeAccepted, T> Accept<ToBeAccepted, S> for QuickQuery
// where
//     S: sqlx::Database + DatabaseSubset,
//     ToBeAccepted: FnOnce() -> T + 'static,
//     T: for<'e> Encode<'e, S> + Type<S> + Send + 'static,
//     QuickQuery: Query<S>,
// {
//     fn accept(
//         this: ToBeAccepted,
//         ctx1: &mut <Self as Query<S>>::Context1,
//     ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
//     {
//         // S::accept_closure(this, ctx1)
//         |_| todo!()
//     }
// }

impl<S> Query<S> for QuickQuery
where
    S: sqlx::Database + DatabaseSubset,
{
    type SqlPart = <S::Q as Query<S>>::SqlPart;
    type Context1 = <S::Q as Query<S>>::Context1;
    type Context2 = <S::Q as Query<S>>::Context2;
    fn build_sql_part_back(
        ctx2: &mut Self::Context2,
        from: Self::SqlPart,
    ) -> String {
        S::Q::build_sql_part_back(ctx2, from)
        // todo!()
    }
    type Output = <S::Q as Query<S>>::Output;
    fn build_query(
        ctx1: Self::Context1,
        f: impl FnOnce(&mut Self::Context2) -> String,
    ) -> (String, Self::Output) {
        S::Q::build_query(ctx1, f)
    }
}

impl<S, T> ToSqlPart<QuickQuery, S> for WhereItemToSqlPart<T>
where
    S: sqlx::Database + DatabaseSubset,
    S::Q: Query<S>,
    T: WhereItem<S, QuickQuery, PanicOnUnsafe> + 'static,
    // WhereItemToSqlPart<T>: ToSqlPart<S::Q, S>,
    // T: WhereItem<S, S::Q>,
{
    fn to_sql_part(
        self,
        ctx: &mut <S::Q as Query<S>>::Context1,
    ) -> <QuickQuery as Query<S>>::SqlPart {
        todo!()
        // S::handle_where_item(self.0, ctx)
    }
}

impl<S, T, Closure> Accept<Closure, S> for QuickQuery
where
    S: sqlx::Database + DatabaseSubset,
    S::Q: Query<S>,
    // S::Q: Accept<Closure, S>,
    Closure: FnOnce() -> T + 'static,
    T: for<'e> Encode<'e, S> + Type<S> + Send + 'static,
{
    fn accept(
        this: Closure,
        ctx1: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
    {
        // S::accept_closure(this, ctx1)
        |_| todo!()
    }
}

impl DatabaseSubset for Sqlite {
    type Q = NamedBorrowedBuffer<'static>;
    fn handle_where_item<T>(
        this: T,
        ctx: &mut <Self::Q as Query<Self>>::Context1,
    ) -> <Self::Q as Query<Self>>::SqlPart
    where
        T: WhereItem<Self, Self::Q, PanicOnUnsafe> + 'static,
        WhereItemToSqlPart<T>: ToSqlPart<Self::Q, Self>,
    {
        <WhereItemToSqlPart<T> as ToSqlPart<Self::Q, Self>>::to_sql_part(WhereItemToSqlPart(this), ctx)
    }
    fn handle_accept<T>(
        this: T,
        ctx: &mut <Self::Q as Query<Self>>::Context1,
    ) -> <Self::Q as Query<Self>>::SqlPart
    where
        Self::Q: Accept<T, Self>,
        AcceptToSqlPart<T>: ToSqlPart<Self::Q, Self>,
    {
        <AcceptToSqlPart<T> as ToSqlPart<Self::Q, Self>>::to_sql_part(AcceptToSqlPart(this), ctx)
    }
    fn accept_closure<T>(
        this: impl FnOnce() -> T + 'static,
        ctx1: &mut <Self::Q as Query<Self>>::Context1,
    ) -> impl FnOnce(
        &mut <Self::Q as Query<Self>>::Context2,
    ) -> String
           + 'static
           + Send
    where
        T: for<'e> Encode<'e, Self>
            + Type<Self>
            + Send
            + 'static,
    {
        |_| todo!()
    }
}
impl DatabaseSubset for MySql {
    type Q = PositionalStaticBuffer;
    fn handle_where_item<T>(
        this: T,
        ctx: &mut <Self::Q as Query<Self>>::Context1,
    ) -> <Self::Q as Query<Self>>::SqlPart
    where
        T: WhereItem<Self, Self::Q, PanicOnUnsafe> + 'static,
    {
        <WhereItemToSqlPart<T> as ToSqlPart<Self::Q, Self>>::to_sql_part(WhereItemToSqlPart(this), ctx)
    }
}

#[cfg(todo)]
mod tests {
    use sqlx::{Encode, Sqlite, Type};
    use stmt::SelectSt;

    use crate::{
        prelude::*,
        quick_query_v2::{DatabaseSubset, QuickQuery},
        Query,
    };
    #[tokio::test]
    async fn wtodo() {
        let mut st =
            stmt::SelectSt::<Sqlite, QuickQuery>::init("Todo");

        st.select(col("id"));
        st.where_(col("id").eq(|| 3));
    }

    fn query<S>(st: &mut SelectSt<S, QuickQuery>)
    where
        S: sqlx::Database + DatabaseSubset,
        QuickQuery: Query<S>,
        i32: for<'e> Encode<'e, S> + Type<S>,
    {
        // st.limit(|| 1);
        st.where_(verbatim__warning__does_not_sanitize(
            "id = ?".to_string(),
        ));
        st.where_(col("id").eq(|| 3));
    }
}
