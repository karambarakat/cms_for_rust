use std::ops::Not;
use std::{borrow::BorrowMut, marker::PhantomData};

use constraints::dyn_object::{DynConstraint, ImplConstraint};
use constraints::exports::not_null;
use constraints::{Constraints, Noop};
use foreign_key::Fk;
use sqlx::{Database, Type};
use types::{ColumnType, ColumnTypeStruct};

use crate::{Statement, Query};

// #[derive(Clone)]
pub struct CreateTableSt<'q, S, Q: Query<S>> {
    pub(crate) header: String,
    pub(crate) ident: (Option<String>, String),
    pub(crate) columns: Vec<(
        String,
        Box<dyn ColumnType + Send>,
        Box<dyn DynConstraint<S, Q> + 'q + Send>,
    )>,
    pub(crate) foreign_keys: Vec<Fk>,
    pub(crate) verbatim: Vec<String>,
    pub(crate) ctx: Q::Context1,
    pub(crate) _sqlx: PhantomData<S>,
}

impl<'q, S, Q> Statement<S, Q> for CreateTableSt<'q, S, Q>
where
    S: 'static + Database,
    Q: Query<S>,
{
    type Init = ();
    fn init(_: Self::Init) -> Self {
        panic!("depricate")
    }
    fn _build(self) -> (String, Q::Output) {
        Q::build_query(self.ctx, |ctx| {
            let mut str = String::from(&self.header);

            str.push(' ');

            if let Some(schema) = self.ident.0 {
                str.push_str(&schema);
            }

            str.push_str(&self.ident.1);

            str.push_str(" (");

            let mut clauses = Vec::new();

            for mut col in self.columns {
                let mut str =
                    format!("{} {}", col.0, col.1.sqlx_info());

                let mut str_2 = String::new();
                col.2.call(ctx, &mut str_2);

                if str_2.is_empty().not() {
                    str.push(' ');
                    str.push_str(&str_2);
                }

                clauses.push(str);
            }

            for fk in self.foreign_keys {
                let mut str = format!(
                    "FOREIGN KEY ({}) REFERENCES {}({})",
                    fk.column, fk.refer_table, fk.refer_column
                );

                if fk.not_null.not() {
                    str.push_str(" ON DELETE SET NULL");
                }

                clauses.push(str);
            }

            for verbatim in self.verbatim {
                clauses.push(verbatim);
            }

            str.push_str(&clauses.join(", "));
            str.push_str(");");

            str
        })
    }
}

impl<'q, S, Q> CreateTableSt<'q, S, Q>
where
    S: 'static + Database,
    Q: Query<S>,
{
    pub fn verbatim(&mut self, verbatim: &str) -> &mut Self {
        self.verbatim.push(verbatim.to_string());
        self
    }
    pub fn column<Ty>(
        &mut self,
        column: &'static str,
        con: impl Constraints<S, Q, Ty> + 'static + Send + Sync,
    ) -> &mut Self
    where
        S: Sync,
        Ty: Type<S> + 'static + Send + Sync,
        Q::Context1: 'q,
        Q: 'q,
        Ty: 'q,
    {
        self.columns.push((
            column.to_string(),
            Box::new(ColumnTypeStruct::<S, Ty>(PhantomData)),
            {
                let bo_mut = self.ctx.borrow_mut();
                let bo_mut = unsafe { &mut *(bo_mut as *mut _) };
                let part = con.constraint(bo_mut);

                Box::new(ImplConstraint {
                    closure: Some(part),
                    _pd: PhantomData::<Ty>,
                })
            },
        ));
        self
    }

    #[deprecated = "use constraint in create_table_st2"]
    pub fn foreign_key(&mut self, fk: Fk) -> &mut Self
    where
        S: SqlxQuery + Sync,
        Q: 'q,
        <S as SqlxQuery>::KeyType: Send,
    {
        if fk.not_null {
            self.column::<S::KeyType>(fk.column, not_null());
        } else {
            self.column::<Option<S::KeyType>>(fk.column, Noop);
        }
        self.foreign_keys.push(fk);
        self
    }
}

pub trait SqlxQuery: Database {
    type KeyType: Type<Self>
        + constraints::IsNull
        + Send
        + Sync
        + 'static;
    fn default_primary_key() -> &'static str;
}

impl SqlxQuery for sqlx::Sqlite {
    type KeyType = i64;
    fn default_primary_key() -> &'static str {
        "PRIMARY KEY AUTOINCREMENT"
    }
}

pub mod foreign_key {
    #[derive(Clone)]
    #[deprecated = "use create_table_st2"]
    pub struct Fk {
        pub not_null: bool,
        pub column: &'static str,
        pub refer_table: &'static str,
        pub refer_column: &'static str,
    }
}

pub mod types {
    use std::marker::PhantomData;

    use sqlx::{prelude::Type, Database};

    pub trait ColumnType {
        fn sqlx_info(&self) -> String;
        fn clone_dyn(&self) -> Box<dyn ColumnType>;
    }

    impl Clone for Box<dyn ColumnType> {
        fn clone(&self) -> Box<dyn ColumnType> {
            self.clone_dyn()
        }
    }
    pub struct ColumnTypeStruct<S, Ty>(
        pub(crate) PhantomData<(S, Ty)>,
    );
    impl<S, Ty> ColumnType for ColumnTypeStruct<S, Ty>
    where
        S: Database,
        Ty: Type<S> + 'static,
    {
        fn clone_dyn(&self) -> Box<dyn ColumnType> {
            Box::new(ColumnTypeStruct::<S, Ty>(PhantomData))
        }
        fn sqlx_info(&self) -> String {
            use sqlx::TypeInfo;
            Ty::type_info().name().to_string()
        }
    }
}

pub mod constraints {
    use std::ops::Not;

    use crate::{Accept, Query};

    use super::SqlxQuery;

    pub trait Constraints<S, Q: Query<S>, Ty> {
        fn constraint(
            self,
            ctx1: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2, &mut String) + Send
        where
            Self: Sized;
    }
    pub mod dyn_object {
        use std::{marker::PhantomData, mem::take};

        use crate::Query;

        pub trait DynConstraint<S, Q: Query<S>> {
            fn call(
                &mut self,
                ctx2: &mut Q::Context2,
                str: &mut String,
            );
        }

        impl<'q, S, Q: Query<S>> Clone
            for Box<dyn DynConstraint<S, Q> + 'q>
        {
            fn clone(&self) -> Self {
                todo!("Clonsing constraint is not supported yet")
            }
        }

        pub struct ImplConstraint<F: Send, T> {
            pub closure: Option<F>,
            pub _pd: PhantomData<T>,
        }

        impl<S, Q, F, T> DynConstraint<S, Q> for ImplConstraint<F, T>
        where
            Q: Query<S>,
            F: FnOnce(&mut Q::Context2, &mut String) + Send,
        {
            fn call(
                &mut self,
                ctx2: &mut <Q as Query<S>>::Context2,
                str: &mut String,
            ) {
                if let Some(c) = take(&mut self.closure) {
                    c(ctx2, str)
                } else {
                    panic!("is there an issue with cloning")
                }
            }
        }
    }

    pub struct Noop;

    impl<S, Q: Query<S>, Ty> Constraints<S, Q, Ty> for Noop {
        fn constraint(
            self,
            _ctx1: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2, &mut String)
        where
            Self: Sized,
        {
            |_, _| {}
        }
    }

    pub fn noop() -> Noop {
        Noop
    }

    impl<S, Q: Query<S>, Ty: IsNull> Constraints<S, Q, Ty> for () {
        fn constraint(
            self,
            _ctx1: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2, &mut String)
        where
            Self: Sized,
        {
            |_, str| {
                if Ty::is_null().not() {
                    str.push_str("NOT NULL");
                }
            }
        }
    }

    pub struct NotNull;

    impl<S, Q: Query<S>, Ty> Constraints<S, Q, Ty> for NotNull {
        fn constraint(
            self,
            _ctx1: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2, &mut String)
        where
            Self: Sized,
        {
            |_, str| str.push_str("NOT NULL")
        }
    }

    pub struct DefaultConstraint<T>(T);

    impl<S, Q, Ty, T> Constraints<S, Q, Ty> for DefaultConstraint<T>
    where
        Q: Accept<T, S>,
    {
        fn constraint(
            self,
            ctx1: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2, &mut String) + Send
        where
            Self: Sized,
        {
            let save = Q::accept(self.0, ctx1);
            |ctx, str| {
                str.push_str(&format!("DEFAULT {}", save(ctx)))
            }
        }
    }

    pub struct DefaultPrimaryKey;

    impl<S: SqlxQuery, Q: Query<S>> Constraints<S, Q, S::KeyType>
        for DefaultPrimaryKey
    {
        fn constraint(
            self,
            _ctx1: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2, &mut String)
        where
            Self: Sized,
        {
            |_, str| str.push_str(&S::default_primary_key())
        }
    }

    pub struct CheckIfNull;

    /// this trait should have only two impls, one default
    /// for every type T (is_null=false), and specialized one for
    /// Option<T> (is_null=true).
    /// when feature specialization gets stebilized I will
    /// simplify this trait's impls, no breaking changes will
    /// occur.
    pub trait IsNull {
        fn is_null() -> bool;
    }

    impl<S, Q: Query<S>, T: IsNull> Constraints<S, Q, T>
        for CheckIfNull
    {
        fn constraint(
            self,
            _ctx1: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2, &mut String)
        where
            Self: Sized,
        {
            |_, str| {
                if T::is_null().not() {
                    str.push_str("NOT NULL");
                }
            }
        }
    }

    mod impl_waiting_specialization {
        use super::IsNull;

        impl<T> IsNull for Option<T> {
            fn is_null() -> bool {
                true
            }
        }

        macro_rules! impl_no_gens {
            ($($ident:ident)*) => {
                $(impl IsNull for $ident {
                    fn is_null() -> bool {
                        false
                    }
                })*
            };
        }

        impl_no_gens!(i32 i64 bool char String);
    }

    pub trait IsConstraint {}
    pub trait NotNullOnlyOnce {}
    pub trait DefaultOnlyOnce {}
    pub trait CheckIfNullOnlyOnce {}

    mod waiting_auto_feature_and_negative_impl_feature {
        use super::*;

        impl IsConstraint for () {}
        impl IsConstraint for NotNull {}
        impl<T> IsConstraint for DefaultConstraint<T> {}

        impl NotNullOnlyOnce for () {}
        impl DefaultOnlyOnce for () {}
        impl CheckIfNullOnlyOnce for () {}

        impl<T> NotNullOnlyOnce for DefaultConstraint<T> {}
        impl<T> CheckIfNullOnlyOnce for DefaultConstraint<T> {}

        impl DefaultOnlyOnce for NotNull {}
        impl CheckIfNullOnlyOnce for NotNull {}

        impl NotNullOnlyOnce for CheckIfNull {}
        impl DefaultOnlyOnce for CheckIfNull {}

        impl<A1, A2> NotNullOnlyOnce for And<A1, A2> where
            A1: NotNullOnlyOnce // A2: NotNullOnlyOnce,
        {
        }
    }

    pub mod exports {
        use super::CheckIfNull;

        pub trait ConstraintExtention: Sized {
            fn not_null(self) -> And<Self, NotNull>
            where
                Self: NotNullOnlyOnce,
            {
                And(self, not_null())
            }
            fn default<T>(
                self,
                t: T,
            ) -> And<Self, DefaultConstraint<T>>
            where
                Self: DefaultOnlyOnce,
            {
                And(self, default(t))
            }
            fn check_if_null(self) -> And<Self, CheckIfNull>
            where
                Self: CheckIfNullOnlyOnce,
            {
                And(self, check_if_null())
            }
        }

        impl<T: IsConstraint> ConstraintExtention for T {}

        pub fn check_if_null() -> CheckIfNull {
            CheckIfNull
        }

        use super::*;
        pub fn not_null() -> NotNull {
            super::NotNull
        }
        pub fn default<T>(t: T) -> DefaultConstraint<T> {
            super::DefaultConstraint(t)
        }
        pub fn primary_key() -> DefaultPrimaryKey {
            DefaultPrimaryKey
        }
    }

    pub struct And<T1, T2>(T1, T2);

    impl<S, Q: Query<S>, Ty, T1, T2> Constraints<S, Q, Ty>
        for And<T1, T2>
    where
        T1: Constraints<S, Q, Ty>,
        T2: Constraints<S, Q, Ty>,
    {
        fn constraint(
            self,
            ctx1: &mut Q::Context1,
        ) -> impl FnOnce(&mut Q::Context2, &mut String)
        where
            Self: Sized,
        {
            let ctx_u1 = unsafe { &mut *(ctx1 as *mut _) };
            let s1 = self.0.constraint(ctx_u1);
            // let _ctx_u1 = unsafe { &mut *(ctx1 as *mut _) };
            let s2 = self.1.constraint(ctx1);
            |ctx, str| {
                s1(ctx, str);
                let mut str_inner = String::new();
                s2(ctx, &mut str_inner);

                if str_inner.is_empty().not() {
                    str.push(' ');
                    str.push_str(&str_inner);
                }
            }
        }
    }
}

pub mod exports {
    pub use super::foreign_key::Fk;
}
