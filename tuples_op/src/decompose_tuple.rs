pub trait DecomposeTuple {
    type This;
    fn back_to_self(this: Self::This) -> Self;
}

impl DecomposeTuple for () {
    type This = ();
    fn back_to_self(this: Self::This) -> Self {
        this
    }
}

impl<T0> DecomposeTuple for (T0,) {
    type This = T0;
    fn back_to_self(this: Self::This) -> Self {
        (this,)
    }
}

impl<T0, T1> DecomposeTuple for (T0, T1) {
    type This = (T0, T1);
    fn back_to_self(this: Self::This) -> Self {
        this
    }
}

impl<T0, T1, T2> DecomposeTuple for (T0, T1, T2) {
    type This = (T0, T1, T2);
    fn back_to_self(this: Self::This) -> Self {
        this
    }
}

impl<T0, T1, T2, T3> DecomposeTuple for (T0, T1, T2, T3) {
    type This = (T0, T1, T2, T3);
    fn back_to_self(this: Self::This) -> Self {
        this
    }
}

impl<T0, T1, T2, T3, T4> DecomposeTuple for (T0, T1, T2, T3, T4) {
    type This = (T0, T1, T2, T3, T4);
    fn back_to_self(this: Self::This) -> Self {
        this
    }
}

impl<T0, T1, T2, T3, T4, T5> DecomposeTuple
    for (T0, T1, T2, T3, T4, T5)
{
    type This = (T0, T1, T2, T3, T4, T5);
    fn back_to_self(this: Self::This) -> Self {
        this
    }
}

impl<T0, T1, T2, T3, T4, T5, T6> DecomposeTuple
    for (T0, T1, T2, T3, T4, T5, T6)
{
    type This = (T0, T1, T2, T3, T4, T5, T6);
    fn back_to_self(this: Self::This) -> Self {
        this
    }
}

impl<T0, T1, T2, T3, T4, T5, T6, T7> DecomposeTuple
    for (T0, T1, T2, T3, T4, T5, T6, T7)
{
    type This = (T0, T1, T2, T3, T4, T5, T6, T7);
    fn back_to_self(this: Self::This) -> Self {
        this
    }
}
