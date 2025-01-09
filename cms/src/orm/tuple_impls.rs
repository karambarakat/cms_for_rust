mod get_one_worker {
    use queries_for_sqlx::{
        ident_safety::PanicOnUnsafe, prelude::stmt,
        quick_query::QuickQuery,
    };
    use sqlx::{sqlite::SqliteRow, Pool, Sqlite};

    use crate::orm::{
        queries_bridge::SelectSt,
        relations::prelude::GetOneWorker,
    };

    impl GetOneWorker for () {
        type Output = ();
        type Inner = ();

        fn on_select(
            &self,
            data: &mut Self::Inner,
            st: &mut SelectSt,
        ) {
        }

        fn from_row(
            &self,
            data: &mut Self::Inner,
            row: &SqliteRow,
        ) {
        }

        fn sub_op<'a>(
            &'a self,
            data: &'a mut Self::Inner,
            pool: Pool<Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send
        {
            async {}
        }

        fn take(self, data: Self::Inner) -> Self::Output {}
    }
    impl<R1> GetOneWorker for (R1,)
    where
        R1: GetOneWorker,
    {
        type Output = (R1::Output,);
        type Inner = (R1::Inner,);

        fn on_select(
            &self,
            data: &mut Self::Inner,
            st: &mut SelectSt,
        ) {
            self.0.on_select(&mut data.0, st);
        }

        fn from_row(
            &self,
            data: &mut Self::Inner,
            row: &SqliteRow,
        ) {
            self.0.from_row(&mut data.0, row);
        }

        fn sub_op<'a>(
            &'a self,
            data: &'a mut Self::Inner,
            pool: Pool<Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send
        {
            async { self.0.sub_op(&mut data.0, pool).await }
        }

        fn take(self, data: Self::Inner) -> Self::Output {
            (self.0.take(data.0),)
        }
    }
    impl<R1, R2> GetOneWorker for (R1, R2)
    where
        R1: GetOneWorker + Send,
        R2: GetOneWorker + Send,
    {
        type Output = (R1::Output, R2::Output);
        type Inner = (R1::Inner, R2::Inner);

        fn on_select(
            &self,
            data: &mut Self::Inner,
            st: &mut SelectSt,
        ) {
            self.0.on_select(&mut data.0, st);
            self.1.on_select(&mut data.1, st);
        }

        fn from_row(
            &self,
            data: &mut Self::Inner,
            row: &SqliteRow,
        ) {
            self.0.from_row(&mut data.0, row);
            self.1.from_row(&mut data.1, row);
        }

        async fn sub_op<'a>(
            &'a self,
            data: &'a mut Self::Inner,
            pool: Pool<Sqlite>,
        ) {
            self.0.sub_op(&mut data.0, pool.clone()).await;
            self.1.sub_op(&mut data.1, pool.clone()).await;
        }

        fn take(self, data: Self::Inner) -> Self::Output {
            (self.0.take(data.0), self.1.take(data.1))
        }
    }
    impl<R1, R2, R3> GetOneWorker for (R1, R2, R3)
    where
        R1: GetOneWorker + Send,
        R2: GetOneWorker + Send,
        R3: GetOneWorker + Send,
    {
        type Output = (R1::Output, R2::Output, R3::Output);
        type Inner = (R1::Inner, R2::Inner, R3::Inner);

        fn on_select(
            &self,
            data: &mut Self::Inner,
            st: &mut SelectSt,
        ) {
            self.0.on_select(&mut data.0, st);
            self.1.on_select(&mut data.1, st);
            self.2.on_select(&mut data.2, st);
        }

        fn from_row(
            &self,
            data: &mut Self::Inner,
            row: &SqliteRow,
        ) {
            self.0.from_row(&mut data.0, row);
            self.1.from_row(&mut data.1, row);
            self.2.from_row(&mut data.2, row);
        }

        async fn sub_op<'a>(
            &'a self,
            data: &'a mut Self::Inner,
            pool: Pool<Sqlite>,
        ) {
            self.0.sub_op(&mut data.0, pool.clone()).await;
            self.1.sub_op(&mut data.1, pool.clone()).await;
            self.2.sub_op(&mut data.2, pool.clone()).await;
        }

        fn take(self, data: Self::Inner) -> Self::Output {
            (
                self.0.take(data.0),
                self.1.take(data.1),
                self.2.take(data.2),
            )
        }
    }
}
mod insert_one_worker {
    use crate::orm::{operations::insert_one::InsertOneWorker, queries_bridge::InsertSt};

    impl InsertOneWorker for () {
        type Inner = ();

        type Output = ();

        fn on_insert(
            &self,
            data: &mut Self::Inner,
            st: &mut InsertSt,
        ) {
        }

        fn from_row(
            &self,
            data: &mut Self::Inner,
            row: &sqlx::sqlite::SqliteRow,
        ) {
        }

        fn sub_op1<'this>(
            &'this self,
            data: &'this mut Self::Inner,
            pool: sqlx::Pool<sqlx::Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send + 'this
        {
            async {}
        }
        fn sub_op2<'this>(
            &'this self,
            data: &'this mut Self::Inner,
            pool: sqlx::Pool<sqlx::Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send + 'this
        {
            async {}
        }

        fn take(self, data: Self::Inner) -> Self::Output {}
    }

    impl<R1> InsertOneWorker for (R1,)
    where
        R1: InsertOneWorker,
    {
        type Inner = (R1::Inner,);

        type Output = (R1::Output,);

        fn on_insert(
            &self,
            data: &mut Self::Inner,
            st: &mut InsertSt,
        ) {
            self.0.on_insert(&mut data.0, st);
        }

        fn from_row(
            &self,
            data: &mut Self::Inner,
            row: &sqlx::sqlite::SqliteRow,
        ) {
            self.0.from_row(&mut data.0, row);
        }

        fn sub_op1<'this>(
            &'this self,
            data: &'this mut Self::Inner,
            pool: sqlx::Pool<sqlx::Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send + 'this
        {
            async { self.0.sub_op1(&mut data.0, pool).await }
        }
        fn sub_op2<'this>(
            &'this self,
            data: &'this mut Self::Inner,
            pool: sqlx::Pool<sqlx::Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send + 'this
        {
            async {}
        }

        fn take(self, data: Self::Inner) -> Self::Output {
            (self.0.take(data.0),)
        }
    }

    impl<R1, R2> InsertOneWorker for (R1, R2)
    where
        R1: InsertOneWorker,
        R2: InsertOneWorker,
    {
        type Inner = (R1::Inner, R2::Inner);

        type Output = (R1::Output, R2::Output);

        fn on_insert(
            &self,
            data: &mut Self::Inner,
            st: &mut InsertSt,
        ) {
            self.0.on_insert(&mut data.0, st);
            self.1.on_insert(&mut data.1, st);
        }

        fn from_row(
            &self,
            data: &mut Self::Inner,
            row: &sqlx::sqlite::SqliteRow,
        ) {
            self.0.from_row(&mut data.0, row);
            self.1.from_row(&mut data.1, row);
        }

        fn sub_op1<'this>(
            &'this self,
            data: &'this mut Self::Inner,
            pool: sqlx::Pool<sqlx::Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send + 'this
        {
            async { self.0.sub_op1(&mut data.0, pool).await }
        }
        fn sub_op2<'this>(
            &'this self,
            data: &'this mut Self::Inner,
            pool: sqlx::Pool<sqlx::Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send + 'this
        {
            async { self.1.sub_op1(&mut data.1, pool).await }
        }

        fn take(self, data: Self::Inner) -> Self::Output {
            (self.0.take(data.0), self.1.take(data.1))
        }
    }

    impl<R1, R2, R3> InsertOneWorker for (R1, R2, R3)
    where
        R1: InsertOneWorker,
        R2: InsertOneWorker,
        R3: InsertOneWorker,
    {
        type Inner = (R1::Inner, R2::Inner, R3::Inner);

        type Output = (R1::Output, R2::Output, R3::Output);

        fn on_insert(
            &self,
            data: &mut Self::Inner,
            st: &mut InsertSt,
        ) {
            self.0.on_insert(&mut data.0, st);
            self.1.on_insert(&mut data.1, st);
            self.2.on_insert(&mut data.2, st);
        }

        fn from_row(
            &self,
            data: &mut Self::Inner,
            row: &sqlx::sqlite::SqliteRow,
        ) {
            self.0.from_row(&mut data.0, row);
            self.1.from_row(&mut data.1, row);
            self.2.from_row(&mut data.2, row);
        }

        fn sub_op1<'this>(
            &'this self,
            data: &'this mut Self::Inner,
            pool: sqlx::Pool<sqlx::Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send + 'this
        {
            async { self.0.sub_op1(&mut data.0, pool).await }
        }
        fn sub_op2<'this>(
            &'this self,
            data: &'this mut Self::Inner,
            pool: sqlx::Pool<sqlx::Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send + 'this
        {
            async { self.1.sub_op1(&mut data.1, pool).await }
        }

        fn take(self, data: Self::Inner) -> Self::Output {
            (
                self.0.take(data.0),
                self.1.take(data.1),
                self.2.take(data.2),
            )
        }
    }

    impl<R1, R2, R3, R4> InsertOneWorker for (R1, R2, R3, R4)
    where
        R1: InsertOneWorker,
        R2: InsertOneWorker,
        R3: InsertOneWorker,
        R4: InsertOneWorker,
    {
        type Inner =
            (R1::Inner, R2::Inner, R3::Inner, R4::Inner);

        type Output =
            (R1::Output, R2::Output, R3::Output, R4::Output);

        fn on_insert(
            &self,
            data: &mut Self::Inner,
            st: &mut InsertSt,
        ) {
            self.0.on_insert(&mut data.0, st);
            self.1.on_insert(&mut data.1, st);
            self.2.on_insert(&mut data.2, st);
            self.3.on_insert(&mut data.3, st);
        }

        fn from_row(
            &self,
            data: &mut Self::Inner,
            row: &sqlx::sqlite::SqliteRow,
        ) {
            self.0.from_row(&mut data.0, row);
            self.1.from_row(&mut data.1, row);
            self.2.from_row(&mut data.2, row);
            self.3.from_row(&mut data.3, row);
        }

        fn sub_op1<'this>(
            &'this self,
            data: &'this mut Self::Inner,
            pool: sqlx::Pool<sqlx::Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send + 'this
        {
            async { self.0.sub_op1(&mut data.0, pool).await }
        }
        fn sub_op2<'this>(
            &'this self,
            data: &'this mut Self::Inner,
            pool: sqlx::Pool<sqlx::Sqlite>,
        ) -> impl std::future::Future<Output = ()> + Send + 'this
        {
            async { self.1.sub_op1(&mut data.1, pool).await }
        }

        fn take(self, data: Self::Inner) -> Self::Output {
            (
                self.0.take(data.0),
                self.1.take(data.1),
                self.2.take(data.2),
                self.3.take(data.3),
            )
        }
    }
}
mod get_query {
    use crate::orm::{queries::Filters, queries_bridge::SelectSt};

    impl<C> Filters<C> for () {
        fn on_select(self, st: &mut SelectSt) {}
    }

    impl<C, R1> Filters<C> for (R1,)
    where
        R1: Filters<C>,
    {
        fn on_select(self, st: &mut SelectSt) {
            self.0.on_select(st);
        }
    }

    impl<C, R1, R2> Filters<C> for (R1, R2)
    where
        R1: Filters<C>,
        R2: Filters<C>,
    {
        fn on_select(self, st: &mut SelectSt) {
            self.0.on_select(st);
            self.1.on_select(st);
        }
    }

    impl<C, R1, R2, R3> Filters<C> for (R1, R2, R3)
    where
        R1: Filters<C>,
        R2: Filters<C>,
        R3: Filters<C>,
    {
        fn on_select(self, st: &mut SelectSt) {
            self.0.on_select(st);
            self.1.on_select(st);
            self.2.on_select(st);
        }
    }

    impl<C, R1, R2, R3, R4> Filters<C> for (R1, R2, R3, R4)
    where
        R1: Filters<C>,
        R2: Filters<C>,
        R3: Filters<C>,
        R4: Filters<C>,
    {
        fn on_select(self, st: &mut SelectSt) {
            self.0.on_select(st);
            self.1.on_select(st);
            self.2.on_select(st);
            self.3.on_select(st);
        }
    }

    impl<C, R1, R2, R3, R4, R5> Filters<C> for (R1, R2, R3, R4, R5)
    where
        R1: Filters<C>,
        R2: Filters<C>,
        R3: Filters<C>,
        R4: Filters<C>,
        R5: Filters<C>,
    {
        fn on_select(self, st: &mut SelectSt) {
            self.0.on_select(st);
            self.1.on_select(st);
            self.2.on_select(st);
            self.3.on_select(st);
            self.4.on_select(st);
        }
    }
}
mod tuple_as_map {
    use core::fmt;

    use crate::tuple_index::tuple_as_map::{
        TupleAsMapTrait, TupleElementKey,
    };

    impl TupleAsMapTrait for () {
        fn keys() -> Vec<&'static str> {
            vec![]
        }
        fn fmt(
            &self,
            _: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            Ok(())
        }
    }
    impl<E0> TupleAsMapTrait for (E0,)
    where
        E0: TupleElementKey + fmt::Debug,
    {
        fn keys() -> Vec<&'static str> {
            vec![E0::key()]
        }
        fn fmt(
            &self,
            f: &mut fmt::Formatter<'_>,
        ) -> fmt::Result {
            f.debug_map().entry(&E0::key(), &self.0).finish()
        }
    }
    impl<E0, E1> TupleAsMapTrait for (E0, E1)
    where
        E0: TupleElementKey + fmt::Debug,
        E1: TupleElementKey + fmt::Debug,
    {
        fn keys() -> Vec<&'static str> {
            vec![E0::key(), E1::key()]
        }
        fn fmt(
            &self,
            f: &mut std::fmt::Formatter<'_>,
        ) -> fmt::Result {
            f.debug_map()
                .entry(&E0::key(), &self.0)
                .entry(&E1::key(), &self.1)
                .finish()
        }
    }
    impl<E0, E1, E2> TupleAsMapTrait for (E0, E1, E2)
    where
        E0: TupleElementKey + fmt::Debug,
        E1: TupleElementKey + fmt::Debug,
        E2: TupleElementKey + fmt::Debug,
    {
        fn keys() -> Vec<&'static str> {
            vec![E0::key(), E1::key(), E2::key()]
        }
        fn fmt(
            &self,
            f: &mut std::fmt::Formatter<'_>,
        ) -> fmt::Result {
            f.debug_map()
                .entry(&E0::key(), &self.0)
                .entry(&E1::key(), &self.1)
                .entry(&E2::key(), &self.2)
                .finish()
        }
    }
}
