pub trait BuildTuple {
    const LEN: usize;
    type Next<N>;
    fn into_next<N>(self, n: N) -> Self::Next<N>;
}

impl BuildTuple for () {
    const LEN: usize = 0;
    type Next<N> = (N,);
    fn into_next<N>(self, n: N) -> Self::Next<N> {
        (n,)
    }
}

impl<T0> BuildTuple for (T0,) {
    const LEN: usize = 1;
    type Next<N> = (T0, N);
    fn into_next<N>(self, n: N) -> Self::Next<N> {
        (self.0, n)
    }
}

impl<T0, T1> BuildTuple for (T0, T1) {
    const LEN: usize = 2;
    type Next<N> = (T0, T1, N);
    fn into_next<N>(self, n: N) -> Self::Next<N> {
        (self.0, self.1, n)
    }
}

impl<T0, T1, T2> BuildTuple for (T0, T1, T2) {
    const LEN: usize = 3;
    type Next<N> = (T0, T1, T2, N);
    fn into_next<N>(self, n: N) -> Self::Next<N> {
        (self.0, self.1, self.2, n)
    }
}

impl<T0, T1, T2, T3> BuildTuple for (T0, T1, T2, T3) {
    const LEN: usize = 4;
    type Next<N> = (T0, T1, T2, T3, N);
    fn into_next<N>(self, n: N) -> Self::Next<N> {
        (self.0, self.1, self.2, self.3, n)
    }
}

impl<T0, T1, T2, T3, T4> BuildTuple for (T0, T1, T2, T3, T4) {
    const LEN: usize = 5;
    type Next<N> = (T0, T1, T2, T3, T4, N);
    fn into_next<N>(self, n: N) -> Self::Next<N> {
        (self.0, self.1, self.2, self.3, self.4, n)
    }
}

impl<T0, T1, T2, T3, T4, T5> BuildTuple for (T0, T1, T2, T3, T4, T5) {
    const LEN: usize = 6;
    type Next<N> = (T0, T1, T2, T3, T4, T5, N);
    fn into_next<N>(self, n: N) -> Self::Next<N> {
        (self.0, self.1, self.2, self.3, self.4, self.5, n)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6> BuildTuple
    for (T0, T1, T2, T3, T4, T5, T6)
{
    const LEN: usize = 7;
    type Next<N> = (T0, T1, T2, T3, T4, T5, T6, N);
    fn into_next<N>(self, n: N) -> Self::Next<N> {
        (self.0, self.1, self.2, self.3, self.4, self.5, self.6, n)
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7> BuildTuple
    for (T0, T1, T2, T3, T4, T5, T6, T7)
{
    const LEN: usize = 8;
    type Next<N> = (T0, T1, T2, T3, T4, T5, T6, T7, N);
    fn into_next<N>(self, n: N) -> Self::Next<N> {
        (
            self.0, self.1, self.2, self.3, self.4, self.5, self.6,
            self.7, n,
        )
    }
}