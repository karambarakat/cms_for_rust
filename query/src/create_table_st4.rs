

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

