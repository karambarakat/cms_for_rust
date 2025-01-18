use crate::{Accept, Query, QueryHandlers};

pub struct Sanitize<T>(pub T);

pub trait SanitizeBehavior<S>: Send + Sync + 'static {
    fn sanitize(self) -> String;
}

impl<S> SanitizeBehavior<S> for String {
    fn sanitize(self) -> String {
        format!("`{}`", self.replace("`", r"\`"))
    }
}

impl<S> SanitizeBehavior<S> for bool {
    fn sanitize(self) -> String {
        match self {
            true => "true".to_string(),
            false => "false".to_string(),
        }
    }
}

macro_rules! sanitize_of_to_string_impls {
    ($ident:ident) => {
        impl<S> SanitizeBehavior<S> for $ident {
            fn sanitize(self) -> String {
                <Self as ToString>::to_string(&self)
            }
        }
    };
}

sanitize_of_to_string_impls!(i8);
sanitize_of_to_string_impls!(i16);
sanitize_of_to_string_impls!(i32);
sanitize_of_to_string_impls!(i64);
sanitize_of_to_string_impls!(u8);
sanitize_of_to_string_impls!(u16);
sanitize_of_to_string_impls!(u32);
sanitize_of_to_string_impls!(u64);

impl<S, T, Q> Accept<Sanitize<T>, S> for Q
where
    Q: QueryHandlers<S>,
    T: SanitizeBehavior<S>,
{
    fn accept(
        this: Sanitize<T>,
        _: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
    {
        move |_| this.0.sanitize()
    }
}

/// rely on sanitize implementation of String
impl<S, Q> Accept<String, S> for Q
where
    Q: QueryHandlers<S>,
{
    fn accept(
        this: String,
        _: &mut Self::Context1,
    ) -> impl FnOnce(&mut Self::Context2) -> String + 'static
    {
        move |_| <String as SanitizeBehavior<S>>::sanitize(this)
    }
}

pub mod exports {
    pub use crate::bind;
    pub fn sanitize<T>(this: T) -> crate::Sanitize<T> {
        super::Sanitize(this)
    }
    pub struct verbatim<T>(pub T);

    impl<S, Q>
        crate::Accept<crate::Sanitize<verbatim<String>>, S> for Q
    where
        Q: crate::QueryHandlers<S>,
    {
        fn accept(
            this: crate::Sanitize<verbatim<String>>,
            _: &mut Self::Context1,
        ) -> impl FnOnce(&mut Self::Context2) -> String + 'static + Send
        {
            move |_| this.0 .0
        }
    }
}
