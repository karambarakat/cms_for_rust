use std::any::{Any, TypeId};

pub trait TupleIndex {
    fn get<T: Any>(&self) -> Option<&T>;
    fn get_mut<T: Any>(&mut self) -> Option<&mut T>;
}

pub trait TupleSearch<T> {
    fn get_contain<SearchFor: 'static>(
        &self,
        filter: Option<&'static str>,
    ) -> Option<&dyn Any>;
    fn get_contain_mut<SearchFor: 'static>(
        &mut self,
        filter: Option<&'static str>,
    ) -> Option<&mut dyn Any>;
}

pub trait Contains<Craiterion>: 'static {
    const FILTER: bool = false;
    fn both(&self) -> (Option<&'static str>, TypeId) {
        if Self::FILTER {
            (self.filter(), self.inner_id())
        } else {
            #[cfg(debug_assertions)]
            {
                if self.filter().is_some() {
                    panic!("FILTER is not set");
                }
            }
            (None, self.inner_id())
        }
    }
    fn filter(&self) -> Option<&'static str> {
        None
    }
    fn inner_id(&self) -> TypeId;
}

// #[tuples_op::impl_trait]
// #[impl_trait::scope(Each)]
// #[impl_trait::impl_for(PhantomData<(Each, T)>)]
// pub trait TupleSearch<T> 
// where
//     Eech: Contains<T>,
// {
//     #[impl_trait::scope(Tuple)]
//     fn get_contain<SearchFor: 'static>(
//         &self,
//         filter: Option<&'static str>,
//     ) -> Option<&dyn Any> {
//         let arr: Vec<(&dyn Any, _)> =
//             self.iter(|element| (element, element.both()))
//                 .collect();
//
//         arr.into_iter().find_map(|(item, id)| {
//             if id == (filter, TypeId::of::<SearchFor>()) {
//                 return Some(item);
//             }
//             None
//         })
//     }
//
//     #[impl_trait::scope(Tuple)]
//     fn get_contain_mut<SearchFor: 'static>(
//         &mut self,
//         filter: Option<&'static str>,
//     ) -> Option<&mut dyn Any> {
//         let both = Tuple::iter(|element| element.both());
//
//         let arr: Vec<&mut dyn Any> =
//             Tuple::iter_mut(|element| element).collect();
//
//         let arr = arr.into_iter().zip(both).collect::<Vec<_>>();
//
//         arr.into_iter().find_map(|(item, id)| {
//             if id == (filter, TypeId::of::<SearchFor>()) {
//                 return Some(item);
//             }
//             None
//         })
//     }
// }

macro_rules! impls {
    ($([$ty:ident, $part:literal]),*) => {
        impl < $($ty: 'static,)* > TupleIndex for ($($ty,)*)
        {
            fn get<'a, T: Any>(&'a self) -> Option<&'a T> {
                let arr: Vec<&'a dyn Any> = vec![$(paste::paste!(&self.$part),)*];
                for item in arr {
                    if let Some(item) = item.downcast_ref::<T>() {
                        return Some(item);
                    }
                }
                None
            }
            fn get_mut<'a, T: Any>(&'a mut self) -> Option<&'a mut T> {
                let arr: Vec<&'a mut dyn Any> = vec![$(paste::paste!(&mut self.$part),)*];
                for item in arr {
                    if let Some(item) = item.downcast_mut::<T>() {
                        return Some(item);
                    }
                }
                None
            }
        }
        impl<T, $($ty,)*> TupleSearch<T> for ($($ty,)*)
        where
            $($ty: Contains<T>,)*
        {
            fn get_contain<SearchFor: 'static>(
                &self,
                filter: Option<&'static str>,
            ) -> Option<&dyn Any> {
                let arr: Vec<(&dyn Any, _)> = vec![
                    $(paste::paste!((&self.$part, self.$part.both())),)*
                ];

                arr.into_iter().find_map(|(item, id)| {
                    if id == (filter, TypeId::of::<SearchFor>()) {
                        return Some(item);
                    }
                    None
                })
            }
            fn get_contain_mut<SearchFor: 'static>(
                &mut self,
                filter: Option<&'static str>,
            ) -> Option<&mut dyn Any> {

                let both = vec![
                    $(paste::paste!(self.$part.both()),)*
                ];

                let arr: Vec<&mut dyn Any> = vec![
                    $(&mut paste::paste!(self.$part),)*
                ];

                let arr = arr .into_iter().zip(both).collect::<Vec<_>>();

                arr.into_iter().find_map(|(item, id)| {
                    if id == (filter, TypeId::of::<SearchFor>()) {
                        return Some(item);
                    }
                    None
                })
            }
        }
    };
}

#[rustfmt::skip]
mod impls_mod {
    use super::*;

    impls!([T0, 0]);
    impls!([T0, 0], [T1, 1]);
    impls!([T0, 0], [T1, 1], [T2, 2]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10]);
    impls!([T0, 0], [T1, 1], [T2, 2], [T3, 3], [T4, 4], [T5, 5], [T6, 6], [T7, 7], [T8, 8], [T9, 9], [T10, 10], [T11, 11]);
}

#[cfg(test)]
mod test {
    use std::marker::PhantomData;

    use super::*;

    pub struct Output<T> {
        pub relation: T,
    }

    pub struct ByRelation;

    impl<T: 'static> Contains<ByRelation> for Output<T> {
        fn inner_id(&self) -> TypeId {
            self.relation.type_id()
        }
    }

    #[test]
    fn tuple_index() {
        let tuple = ("hello", 42, 3.14);

        let found = tuple.get::<&'static str>().unwrap();

        assert_eq!(found, &"hello");

        let tuple = (
            Output { relation: "hello" },
            Output { relation: 42 },
            Output { relation: 3.14 },
        );

        fn get_contain<'a, T: 'static, Tuple>(
            tuple: &'a Tuple,
            _: PhantomData<T>,
        ) -> Option<&'a Output<T>>
        where
            Tuple: TupleSearch<ByRelation>,
        {
            let any = tuple.get_contain::<T>(None);
            any.map(|any| {
                any.downcast_ref::<Output<T>>().unwrap()
            })
        }

        let found =
            get_contain(&tuple, PhantomData::<&'static str>)
                .expect("i'm sure it has str relation");

        let st = std::any::type_name_of_val(found);

        assert_eq!(
            st,
            "cms_for_rust::tuple_index::test::Output<&str>"
        );

        let found =
            get_contain(&tuple, PhantomData::<String>).is_none();

        assert_eq!(found, true);
    }
}
