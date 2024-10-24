// mod v1 {
//     #![allow(unused)]
//
//     use std::ops::{Deref, DerefMut};
//
//     pub trait IList<C> {
//         fn run(
//             &self,
//             ctx: &mut Context<C>,
//         ) -> Result<(), String>;
//     }
//     pub struct DynList<C> {
//         list: Vec<Box<dyn IList<C>>>,
//     }
//
//     impl<C> DynList<C> {
//         pub fn new(list: Vec<Box<dyn IList<C>>>) -> Self {
//             Self { list }
//         }
//         pub fn run(mut self, mut data: C) -> Result<C, String> {
//             let mut points = vec![];
//
//             while let Some(item) = self.list.pop() {
//                 let mut ctx = Context {
//                     points: &mut points,
//                     list: &mut self.list,
//                     data: &mut data,
//                 };
//
//                 item.run(&mut ctx)?;
//             }
//             Ok(data)
//         }
//     }
//
//     pub struct Context<'l, C> {
//         points: &'l mut Vec<String>,
//         list: &'l mut Vec<Box<dyn IList<C>>>,
//         data: &'l mut C,
//     }
//
//     impl<'l, C> Context<'l, C> {
//         pub fn has_event_occured(&self, name: &str) -> bool {
//             self.points.iter().any(|e| e.eq(name))
//         }
//         pub fn wait_for_event(
//             &mut self,
//             name: String,
//         ) -> Result<(), String> {
//             if self.points.contains(&name) {
//                 return Ok(());
//             }
//
//             if self.list.is_empty() {
//                 return Err("No events".to_string());
//             }
//
//             while let Some(item) = self.list.pop() {
//                 let mut ctx = Context {
//                     points: &mut self.points,
//                     list: &mut self.list,
//                     data: &mut self.data,
//                 };
//
//                 item.run(&mut ctx)?;
//
//                 if self.points.contains(&name) {
//                     return Ok(());
//                 }
//             }
//
//             Err("No events".to_string())
//         }
//         pub fn emit_event(&mut self, name: &str) {
//             self.points.push(name.to_string());
//         }
//     }
//
//     impl<'l, C> Deref for Context<'l, C> {
//         type Target = C;
//         fn deref(&self) -> &Self::Target {
//             &self.data
//         }
//     }
//
//     impl<'l, C> DerefMut for Context<'l, C> {
//         fn deref_mut(&mut self) -> &mut Self::Target {
//             &mut self.data
//         }
//     }
// }

pub mod ilist {
    pub struct DynList<C> {
        list: Vec<Box<dyn EventfulList<C>>>,
    }

    impl<Any> Default for DynList<Any> {
        fn default() -> Self {
            Self { list: vec![] }
        }
    }

    #[derive(Debug)]
    pub enum IListError {
        WaitingForNonExistentEvent(String),
    }

    impl<C> DynList<C> {
        pub fn add<N: EventfulList<C> + 'static>(
            &mut self,
            item: N,
        ) {
            self.list.push(Box::new(item));
        }
        pub fn new(list: Vec<Box<dyn EventfulList<C>>>) -> Self {
            Self { list }
        }
        pub fn run(
            mut self,
            mut data: C,
        ) -> Result<C, IListError> {
            let mut points = vec![];

            while let Some(item) = self.list.pop() {
                let mut ctx = Context {
                    events: &mut points,
                    list: &mut self.list,
                    data: &mut data,
                };

                item.run(&mut ctx)?;
            }
            Ok(data)
        }
    }

    pub struct Context<'l, C> {
        events: &'l mut Vec<&'static str>,
        list: &'l mut Vec<Box<dyn EventfulList<C>>>,
        data: &'l mut C,
    }

    impl<C> std::ops::Deref for Context<'_, C> {
        type Target = C;

        fn deref(&self) -> &Self::Target {
            self.data
        }
    }

    impl<C> std::ops::DerefMut for Context<'_, C> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.data
        }
    }

    impl<'l, C> Context<'l, C> {
        pub fn has_event_occured(&self, name: &str) -> bool {
            self.events.iter().any(|e| e.eq(&name))
        }
        pub fn wait_for_event(
            &mut self,
            name: &'static str,
        ) -> Result<(), IListError> {
            if self.events.contains(&name) {
                return Ok(());
            }

            while let Some(item) = self.list.pop() {
                let mut ctx = Context {
                    events: &mut self.events,
                    list: &mut self.list,
                    data: &mut self.data,
                };

                item.run(&mut ctx)?;

                if self.events.contains(&name) {
                    return Ok(());
                }
            }

            Err(IListError::WaitingForNonExistentEvent(
                name.to_string(),
            ))
        }
        pub fn event(&mut self, name: &'static str) {
            self.events.push(name);
        }
    }

    // pub fn collect_statics(&self) -> Result<(), IListError2<C::Event>>

    pub trait EventfulList<C> {
        fn run(
            &self,
            ctx: &mut Context<C>,
        ) -> Result<(), IListError>;
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use std::collections::HashMap;
        #[derive(Default)]
        struct Map(HashMap<String, Vec<String>>);

        impl std::ops::Deref for Map {
            type Target = HashMap<String, Vec<String>>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for Map {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        struct NewTable(&'static str);

        impl EventfulList<Map> for NewTable {
            fn run(
                &self,
                ctx: &mut Context<Map>,
            ) -> Result<(), IListError> {
                ctx.event(self.0);
                ctx.insert(self.0.to_string(), vec![]);
                Ok(())
            }
        }

        struct NewFN(&'static str, &'static str);

        impl EventfulList<Map> for NewFN {
            fn run(
                &self,
                ctx: &mut Context<Map>,
            ) -> Result<(), IListError> {
                ctx.wait_for_event(self.0)?;

                ctx.get_mut(self.0)
                    .expect("should be waited for")
                    .push(self.1.to_string());

                Ok(())
            }
        }

        #[test]
        fn it_works() {
            let mut list = DynList::<Map>::default();

            list.add(NewFN("Table2", "Fn3"));
            list.add(NewFN("Table1", "Fn1"));
            list.add(NewFN("Table2", "Fn1"));
            list.add(NewTable("Table1"));
            list.add(NewTable("Table2"));

            let hm =
                list.run(Map::default()).expect("should be ok");

            assert_eq!(
                hm.0,
                HashMap::from_iter(vec![
                    (
                        "Table1".to_string(),
                        vec!["Fn1".to_string()]
                    ),
                    (
                        "Table2".to_string(),
                        vec![
                            "Fn1".to_string(),
                            "Fn3".to_string()
                        ]
                    )
                ])
            );
        }
    }
}

#[cfg(test)]
pub mod testing_prelude;
