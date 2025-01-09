use std::{marker::PhantomData, sync::Arc};

pub struct This {
    to: String,
    to2: String,
}
pub trait Trait {
    fn fn_1(&mut self, str: &mut String) {}
    fn fn_2(&mut self, str: &mut String) {}
}

pub struct Dep {
    this: Arc<This>,
    to3: String,
}

impl Trait for Dep {}

pub struct Dep2<Fn1, Fn2>
where
    Fn1: FnMut(&mut String),
    Fn2: FnMut(&mut String),
{
    fm_1: Fn1,
    fm_2: Fn2,
}

impl<F1, F2> Trait for Dep2<F1, F2>
where
    F1: FnMut(&mut String),
    F2: FnMut(&mut String),
{
    fn fn_1(&mut self, str: &mut String) {
        (self.fm_1)(str)
    }
    fn fn_2(&mut self, str: &mut String) {
        (self.fm_2)(str)
    }
}

fn init(this: Arc<This>) -> Box<dyn Trait + Sync + Send> {
    let this1 = this.clone();
    let this2 = this.clone();
    let mut dep2 = Dep2 {
        fm_2: move |str| {
            let take = this1.clone().to.clone();
        },
        fm_1: move |str| {
            let take = this2.clone().to.clone();
        },
    };
    Box::new(dep2)
}
