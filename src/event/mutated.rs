use std::{borrow::Cow, cell::RefCell, fmt::Debug, rc::Rc};

use crate::{
    event::{fixed::FixedEventIter, Event, EventIter},
    BeatTimeBase,
};

// -------------------------------------------------------------------------------------------------

/// Pointer to a function which mutates an Event.
type EventMapFn = dyn FnMut(Event) -> Event + 'static;

// -------------------------------------------------------------------------------------------------

/// Endlessly emits [`Event`] which's value can be mutated in each iter step
/// with a custom closure.
///
/// NB: This event iter can not be cloned. `clone_dyn` thus will cause a panic!
pub struct MutatedEventIter {
    events: Vec<Event>,
    event_index: usize,
    initial_events: Vec<Event>,
    map: Box<EventMapFn>,
    reset_map: Box<dyn Fn() -> Box<EventMapFn>>,
}

impl MutatedEventIter {
    pub fn new<F>(events: Vec<Event>, map: F) -> Self
    where
        F: FnMut(Event) -> Event + Clone + 'static,
    {
        // capture initial map state
        let initial_map = map.clone();
        // apply first mutation and memorize initial set of events
        let mut map = Box::new(map);
        let mut initial_events = events;
        if !initial_events.is_empty() {
            initial_events[0] = Self::mutate(initial_events[0].clone(), &mut map);
        }
        let events = initial_events.clone();
        let event_index = 0;
        Self {
            events,
            event_index,
            initial_events,
            reset_map: Box::new(move || Box::new(initial_map.clone())),
            map,
        }
    }

    fn mutate(event: Event, map: &mut dyn FnMut(Event) -> Event) -> Event {
        (*map)(event)
    }
}

impl Debug for MutatedEventIter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("MutatedEventIter")
            .field("events", &self.events)
            .field("event_index", &self.event_index)
            .field("initial_events", &self.initial_events)
            .finish()
    }
}

impl EventIter for MutatedEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_external_context(&mut self, _data: &[(Cow<str>, f64)]) {
        // nothing to do
    }

    fn run(&mut self, _pulse: crate::PulseIterItem, emit_event: bool) -> Option<Event> {
        if emit_event {
            let event = self.events[self.event_index].clone();
            self.events[self.event_index] = Self::mutate(event.clone(), &mut self.map);
            self.event_index += 1;
            if self.event_index >= self.events.len() {
                self.event_index = 0;
            }
            Some(event)
        } else {
            None
        }
    }

    fn duplicate(&self) -> Rc<RefCell<dyn EventIter>> {
        panic!("Mutated event iters can't be cloned")
    }

    fn reset(&mut self) {
        self.events = self.initial_events.clone();
        self.event_index = 0;
        self.map = (self.reset_map)();
    }
}

// -------------------------------------------------------------------------------------------------

pub trait ToMutatedEventIter<F>
where
    F: FnMut(Event) -> Event + Clone + 'static,
{
    fn mutate(self, map: F) -> MutatedEventIter;
}

impl<F> ToMutatedEventIter<F> for FixedEventIter
where
    F: FnMut(Event) -> Event + Clone + 'static,
{
    /// Upgrade a [`FixedEventIter`] to a [`MutatedEventIter`].
    fn mutate(self, map: F) -> MutatedEventIter {
        MutatedEventIter::new(self.events(), map)
    }
}
