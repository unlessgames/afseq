use std::{cell::RefCell, rc::Rc};

use crate::{
    event::{Event, EventIter},
    BeatTimeBase, PulseIterItem,
};

// -------------------------------------------------------------------------------------------------

/// Emits an empty, None [`Event`].
#[derive(Clone, Debug)]
pub struct EmptyEventIter {}

impl Iterator for EmptyEventIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl EventIter for EmptyEventIter {
    fn set_time_base(&mut self, _time_base: &BeatTimeBase) {
        // nothing to do
    }

    fn set_pulse(&mut self, _context: PulseIterItem, _pattern_pulse_count: usize, _emit_event: bool) {
        // nothing to do
    }

    fn duplicate(&self) -> Rc<RefCell<dyn EventIter>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn reset(&mut self) {
        // nothing to do
    }
}
