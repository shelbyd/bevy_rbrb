use bevy_app::EventWriter;
use bevy_ecs::{prelude::Res, system::SystemParam};
use derive_more::*;

#[derive(SystemParam)]
pub struct NetworkEventWriter<'a, T: Send + Sync + 'static> {
    confirmed_writer: EventWriter<'a, Confirmed<T>>,
    unconfirmed_writer: EventWriter<'a, Unconfirmed<T>>,
    advance_confirmation: Res<'a, rbrb::Confirmation>,
}

impl<'a, T: Send + Sync + 'static> NetworkEventWriter<'a, T> {
    pub fn send(&mut self, event: T)
    where
        T: Clone,
    {
        if *self.advance_confirmation == rbrb::Confirmation::First {
            self.confirmed_writer.send(Confirmed(event.clone()));
        }
        self.unconfirmed_writer.send(Unconfirmed(event));
    }
}

#[derive(Deref, DerefMut, Clone, Copy, Debug)]
pub struct Confirmed<T>(pub T);

#[derive(Deref, DerefMut, Clone, Copy, Debug)]
pub struct Unconfirmed<T>(pub T);
