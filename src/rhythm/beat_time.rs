use std::{cell::RefCell, rc::Rc};

use crate::{
    event::{empty::EmptyEventIter, Event, EventIter},
    pattern::{Pattern, fixed::FixedPattern},
    time::{BeatTimeBase, BeatTimeStep, SampleTimeDisplay},
    Rhythm, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Emits `Option(Event)` every nth [`BeatTimeStep`] with an optional pattern and offset.
#[derive(Clone, Debug)]
pub struct BeatTimeRhythm {
    time_base: BeatTimeBase,
    step: BeatTimeStep,
    offset: BeatTimeStep,
    pattern: Rc<RefCell<dyn Pattern>>,
    event_iter: Rc<RefCell<dyn EventIter>>,
    event_iter_sample_time: f64,
    sample_offset: SampleTime,
}

impl BeatTimeRhythm {
    /// Create a new pattern based emitter which emits `value` every beat_time `step`.  
    pub fn new(time_base: BeatTimeBase, step: BeatTimeStep) -> Self {
        let offset = BeatTimeStep::Beats(0.0);
        let pattern = Rc::new(RefCell::new(FixedPattern::default()));
        let event_iter = Rc::new(RefCell::new(EmptyEventIter {}));
        let event_iter_sample_time = offset.to_samples(&time_base);
        let sample_offset = 0;
        Self {
            time_base,
            step,
            offset,
            pattern,
            event_iter,
            event_iter_sample_time,
            sample_offset,
        }
    }

    /// Get current time base.
    pub fn time_base(&self) -> BeatTimeBase {
        self.time_base
    }
    /// Get current step.
    pub fn step(&self) -> BeatTimeStep {
        self.step
    }
    /// Get current offset.
    pub fn offset(&self) -> BeatTimeStep {
        self.offset
    }
    /// Get current pattern.
    pub fn pattern(&self) -> Rc<RefCell<dyn Pattern>> {
        self.pattern.clone()
    }

    /// Trigger events with the given [`Pattern`].  
    pub fn with_pattern<T: Pattern + Sized + 'static>(&self, pattern: T) -> Self {
        self.with_pattern_dyn(Rc::new(RefCell::new(pattern)))
    }

    /// Trigger events with the given [`Pattern`].  
    pub fn with_pattern_dyn(&self, pattern: Rc<RefCell<dyn Pattern>>) -> Self {
        Self {
            pattern: pattern.clone(),
            event_iter: self.event_iter.clone(),
            ..*self
        }
    }

    /// Apply the given beat-time step offset to all events.
    pub fn with_offset<O: Into<Option<BeatTimeStep>>>(&self, offset: O) -> Self {
        let offset = offset.into().unwrap_or(BeatTimeStep::Beats(0.0));
        let event_iter_sample_time = offset.to_samples(&self.time_base);
        Self {
            offset,
            pattern: self.pattern.clone(),
            event_iter: self.event_iter.clone(),
            event_iter_sample_time,
            ..*self
        }
    }
    /// Apply the given offset in out step's timing resolution to all events.
    pub fn with_offset_in_step(&self, offset: f32) -> Self {
        let mut offset_steps = self.step;
        offset_steps.set_steps(offset);
        self.with_offset(offset_steps)
    }

    /// Use the given [`EventIter`] to trigger events.
    pub fn trigger<Iter: EventIter + 'static>(&self, iter: Iter) -> Self {
        self.trigger_dyn(Rc::new(RefCell::new(iter)))
    }

    /// Use the given dyn [`EventIter`] to trigger events.
    pub fn trigger_dyn(&self, event_iter: Rc<RefCell<dyn EventIter>>) -> Self {
        Self {
            pattern: self.pattern.clone(),
            event_iter,
            ..*self
        }
    }
}

impl Iterator for BeatTimeRhythm {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut pattern = self.pattern.borrow_mut();
        if pattern.is_empty() {
            None
        } else {
            let sample_time = self.event_iter_sample_time as SampleTime + self.sample_offset;
            let value = if pattern.run() > 0.0 {
                let mut event_iter = self.event_iter.borrow_mut();
                Some((sample_time, event_iter.next()))
            } else {
                Some((sample_time, None))
            };
            self.event_iter_sample_time += self.samples_per_step();
            value
        }
    }
}

impl Rhythm for BeatTimeRhythm {
    fn time_display(&self) -> Box<dyn SampleTimeDisplay> {
        Box::new(self.time_base)
    }
    fn set_time_base(&mut self, time_base: BeatTimeBase) {
        self.time_base = time_base;
    }

    fn samples_per_step(&self) -> f64 {
        self.step.to_samples(&self.time_base)
    }
    fn pattern_length(&self) -> usize {
        self.pattern.borrow().len()
    }

    fn sample_offset(&self) -> SampleTime {
        self.sample_offset
    }
    fn set_sample_offset(&mut self, sample_offset: SampleTime) {
        self.sample_offset = sample_offset
    }

    fn next_until_time(&mut self, sample_time: SampleTime) -> Option<(SampleTime, Option<Event>)> {
        let current_sample_time = self.event_iter_sample_time + self.sample_offset as f64;
        if (current_sample_time as SampleTime) < sample_time {
            self.next()
        } else {
            None
        }
    }

    fn reset(&mut self) {
        // reset sample offset
        self.sample_offset = 0;
        // reset iterator state
        self.event_iter.borrow_mut().reset();
        self.event_iter_sample_time = self.offset.to_samples(&self.time_base);
        self.pattern.borrow_mut().reset();
    }
}
