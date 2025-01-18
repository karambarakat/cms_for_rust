// #[rustfmt::skip]
// mod impl_schema_cols_for_tuples {
//     use crate::SchemaColumn;
//     use crate::Query;
//
//     macro_rules! impls {
//         ($([$ident:ident, $part:literal]),*) => {
//             impl<S, Q: Query<S>, $($ident,)*> SchemaColumn<S, Q> for ($($ident,)*)
//             where
//             $(
//                 $ident: SchemaColumn<S, Q>,
//             )*
//             {
//                 fn column(
//                     self,
//                     ctx: &mut Q::Context1,
//                 ) -> impl FnOnce(&mut Q::Context2) -> String
//                 {
//                     let ptr = ctx as *mut _;
//                     let first = (  $( paste::paste! {
//                         self.$part.column(unsafe { &mut *ptr })
//                     },)* );
//                     move |ctx2| {
//                         let mut str = Vec::new();
//
//                         paste::paste! { $(
//                             str.push(first.$part(ctx2));
//                         )* }
//
//                         str.join(" ")
//                     }
//                 }
//             }
//         };
//     }
//
//     impls!([T0, 0]);
//     impls!([T0, 0], [T1, 1]);
//     impls!([T0, 0], [T1, 1], [T2, 2]);
//     impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3]);
//     impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4]);
//     impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5]);
//     impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6]);
//     impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7]);
//     impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8]);
//     impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9]);
//     impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10]);
//     impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10], [T11, 11]);
// }

#[rustfmt::skip]
mod impl_consume_into_args_for_encode_types {
    pub use crate::IntoMutArguments;
    pub use sqlx::Database;
    pub use sqlx::database::HasArguments;
    pub use sqlx::Encode;
    pub use sqlx::Type;
    pub use sqlx::Arguments;

    macro_rules! impls {
        ($len:literal $([$ident:ident, $part:literal])*) => {
            #[allow(unused)]
            impl<'q, DB, $($ident,)*> IntoMutArguments<'q, DB> for ($($ident,)*)
            where
                DB: Database,
                $($ident: Encode<'q, DB> + Type<DB> + Send + 'q,)*
            {
            const LEN : usize = $len;
                fn into_arguments(
                    self,
                    argument: &mut <DB as HasArguments<'q>>::Arguments,
                ) {
                    paste::paste! { $(
                        argument.add(self.$part);
                    )* }
                }
            }
        };
    }

    impls!(0);
    impls!(1 [T0, 0]);
    impls!(2 [T0, 0] [T1, 1]);
    impls!(3 [T0, 0] [T1, 1] [T2, 2]);
    impls!(4 [T0, 0] [T1, 1] [T2, 2] [T3, 3]);
    impls!(5 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4]);
    impls!(6 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5]);
    impls!(7 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6]);
    impls!(8 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7]);
    impls!(9 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8]);
    impls!(10 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8] [T9, 9]);
    impls!(11 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8] [T9, 9] [T10, 10]);
    impls!(12 [T0, 0] [T1, 1] [T2, 2] [T3, 3] [T4, 4] [T5, 5] [T6, 6] [T7, 7] [T8, 8] [T9, 9] [T10, 10] [T11, 11]);

}
