use crate::{
    event::{Event, EventIter},
    time::{BeatTimeBase, BeatTimeStep},
    Rhythm, SampleTime,
};

// -------------------------------------------------------------------------------------------------

/// Emits `Some(Event)` every nth [`BeatTimeStep`]::Beat or Bar.
pub struct BeatTimeRhythm {
    time_base: BeatTimeBase,
    step: BeatTimeStep,
    offset: BeatTimeStep,
    current_sample_time: f64,
    event_iter: Box<dyn EventIter>,
}

impl BeatTimeRhythm {
    /// Create a new beat time pattern which emits the given `value` every beat-time `step`.  
    pub fn new<Iter: EventIter + 'static>(
        time_base: BeatTimeBase,
        step: BeatTimeStep,
        event_iter: Iter,
    ) -> Self {
        Self::new_with_offset(time_base, step, BeatTimeStep::Beats(0.0), event_iter)
    }

    /// Create a new beat time pattern which emits the given `value` every beat-time `step`
    /// starting at the given beat-time `offset`.  
    pub fn new_with_offset<Iter: EventIter + 'static>(
        time_base: BeatTimeBase,
        step: BeatTimeStep,
        offset: BeatTimeStep,
        event_iter: Iter,
    ) -> Self {
        let current_sample_time = offset.to_samples(&time_base);
        Self {
            time_base,
            step,
            offset,
            current_sample_time,
            event_iter: Box::new(event_iter),
        }
    }
}

impl Iterator for BeatTimeRhythm {
    type Item = (SampleTime, Option<Event>);

    fn next(&mut self) -> Option<Self::Item> {
        let sample_time = self.current_sample_time as SampleTime;
        self.current_sample_time += self.step.to_samples(&self.time_base);
        Some((sample_time, self.event_iter.next()))
    }
}

impl Rhythm for BeatTimeRhythm {
    fn reset(&mut self) {
        self.event_iter.reset();
        self.current_sample_time = self.offset.to_samples(&self.time_base);
    }
}
