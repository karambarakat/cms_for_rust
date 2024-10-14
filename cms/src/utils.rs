pub(crate) mod eventfull_list {
    use std::ops::{Deref, DerefMut};

    pub trait EventType {
        type Event: Eq;
    }

    pub trait EventfulList<C: EventType> {
        fn run(
            &self,
            ctx: &mut Context<C>,
        ) -> Result<(), C::Event>;
    }

    impl<'l, C: EventType> Context<'l, C> {
        pub fn has_event_occured(
            &self,
            event: C::Event,
        ) -> bool {
            self.events.iter().any(|e| e.eq(&event))
        }
        pub fn wait_for_event(
            &mut self,
            event: C::Event,
        ) -> Result<(), C::Event> {
            if self.events.contains(&event) {
                return Ok(());
            }

            while let Some(item) = self.list.pop() {
                let mut ctx = Context {
                    events: &mut self.events,
                    list: &mut self.list,
                    data: &mut self.data,
                };

                item.run(&mut ctx)?;

                if self.events.contains(&event) {
                    return Ok(());
                }
            }

            Err(event)
        }
        pub fn event(&mut self, event: C::Event) {
            self.events.push(event);
        }
    }

    pub struct Context<'l, C: EventType> {
        events: &'l mut Vec<C::Event>,
        list: &'l mut Vec<Box<dyn EventfulList<C>>>,
        data: &'l mut C,
    }

    impl<'l, C: EventType> Context<'l, C> {
        pub fn new(
            events: &'l mut Vec<C::Event>,
            list: &'l mut Vec<Box<dyn EventfulList<C>>>,
            data: &'l mut C,
        ) -> Self {
            Self { events, list, data }
        }
    }

    impl<C: EventType> Deref for Context<'_, C> {
        type Target = C;

        fn deref(&self) -> &Self::Target {
            self.data
        }
    }

    impl<C: EventType> DerefMut for Context<'_, C> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.data
        }
    }

    // pub struct Submitable<C: EventType> {
    //     object: fn() -> Box<dyn EventfulList<C>>,
    // }

    // impl<C> inventory::Collect for Submitable<C> {}
}
