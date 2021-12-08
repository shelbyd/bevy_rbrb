use bevy_app::EventWriter;
use bevy_ecs::{
    component::Component,
    prelude::{Local, Res},
    system::SystemParam,
};
use derive_more::*;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Default)]
pub struct RbrbFrame(pub u32);

#[derive(SystemParam)]
pub struct NetworkEventWriter<'a, T: Component> {
    confirmed_writer: EventWriter<'a, Confirmed<T>>,
    unconfirmed_writer: EventWriter<'a, Unconfirmed<T>>,

    advance_confirmation: Res<'a, rbrb::Confirmation>,
    frame: Res<'a, RbrbFrame>,

    // TODO(shelbyd): Don't grow memory forever.
    sent_events: Local<'a, HashMap<u32, HashSet<T>>>,
}

impl<'a, T: Send + Sync + 'static> NetworkEventWriter<'a, T> {
    pub fn send(&mut self, event: T)
    where
        T: Clone + Hash + Eq,
    {
        // TODO(shelbyd): Allowing sending multiples of the same event.
        let sent = self.sent_events.entry(self.frame.0).or_default();
        if !sent.contains(&event) {
            self.unconfirmed_writer.send(Unconfirmed(event.clone()));
            sent.insert(event.clone());
        }

        if *self.advance_confirmation == rbrb::Confirmation::First {
            self.confirmed_writer.send(Confirmed(event));
        }
    }
}

#[derive(Deref, DerefMut, Clone, Copy, Debug)]
pub struct Confirmed<T>(pub T);

#[derive(Deref, DerefMut, Clone, Copy, Debug)]
pub struct Unconfirmed<T>(pub T);
