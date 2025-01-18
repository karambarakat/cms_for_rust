pub use super::prelude::*;

pub struct OneToMany;

impl LinkSpec for OneToMany {}

// impl<From, To> GetOneWorker
//     for RelationWorker<ManyToMany, From, To>
// where
//     From: Collection,
//     To: Collection,
// {
//     type Inner = (Option<i64>, Vec<(i64, To)>);
//     type Output = Vec<Output<To>>;
//
//     fn on_select(
//         &self,
//         data: &mut Self::Inner,
//         st: &mut stmt::SelectSt<
//             Sqlite,
//             QuickQuery,
//             PanicOnUnsafe,
//         >,
//     ) {
//         // no-op
//     }
//
//     fn from_row(&self, data: &mut Self::Inner, row: &SqliteRow) {
//         *&mut data.0 = Some(row.get("local_id"))
//     }
//
//     fn sub_op<'t>(
//         &'t self,
//         data: &'t mut Self::Inner,
//         pool: Pool<Sqlite>,
//     ) -> impl Future<Output = ()> + Send + 't {
//         async move {
//             let id = data.0.unwrap();
//
//             let mut st: SelectSt<_, QuickQuery, _> =
//                 stmt::SelectSt::init(
//                     self.rel_spec.conjuction_table.to_string(),
//                 );
//
//             st.select(
//                 ft(self.rel_spec.conjuction_table.clone())
//                     .col(self.rel_spec.to_id.clone())
//                     .alias("to_id"),
//             );
//
//             To::on_select1(&mut st);
//
//             st.where_(
//                 col(self.rel_spec.from_id.clone())
//                     .eq(move || id),
//             );
//
//             st.join(Join {
//                 ty: "LEFT JOIN",
//                 on_table: To::table_name1().to_owned(),
//                 on_column: "id".to_string(),
//                 local_column: self.rel_spec.to_id.clone(),
//             });
//
//             let vals = st
//                 .fetch_all(&pool, |row| {
//                     let val = To::on_get1(&row);
//                     Ok((row.get::<'_, i64, _>("to_id"), val))
//                 })
//                 .await
//                 .unwrap();
//
//             *&mut data.1 = vals;
//         }
//     }
//
//     fn take(self, data: Self::Inner) -> Self::Output {
//         data.1
//             .into_iter()
//             .map(|e| Output { id: e.0, attr: e.1 })
//             .collect()
//     }
// }
