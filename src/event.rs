//! Value and value iters which get emitted by `Rhythms`.

use self::{
    fixed::{FixedEventIter, ToFixedEventValue},
    sequence::{EventIterSequence, ToEventIterSequence},
};

use core::sync::atomic::{AtomicUsize, Ordering};
use std::fmt::Debug;

pub mod empty;
pub mod fixed;
pub mod mapped;
pub mod mapped_note;
pub mod sequence;

// -------------------------------------------------------------------------------------------------

/// Id to refer to a specific instrument in a NoteEvent.
pub type InstrumentId = usize;
/// Id to refer to a specific parameter in a ParameterChangeEvent.
pub type ParameterId = usize;

// -------------------------------------------------------------------------------------------------

/// Generate a new unique instrument id.
pub fn unique_instrument_id() -> InstrumentId {
    static ID: AtomicUsize = AtomicUsize::new(0);
    ID.fetch_add(1, Ordering::Relaxed)
}

// -------------------------------------------------------------------------------------------------

/// Single note event in a [`Event`].
#[derive(Clone)]
pub struct NoteEvent {
    pub instrument: Option<InstrumentId>,
    pub note: u32,
    pub velocity: f32,
}

/// Shortcut for creating a new [`NoteEvent`].
pub fn new_note<Instrument: Into<Option<InstrumentId>>>(
    instrument: Instrument,
    note: u32,
    velocity: f32,
) -> NoteEvent {
    let instrument: Option<InstrumentId> = instrument.into();
    NoteEvent {
        instrument,
        note,
        velocity,
    }
}

/// Shortcut for creating a new sequence of [`NoteEvent`]:
/// e.g. a sequence of single notes
pub fn new_note_sequence<I: Into<Option<InstrumentId>>>(
    sequence: Vec<(I, u32, f32)>,
) -> Vec<NoteEvent> {
    let mut event_sequence = Vec::with_capacity(sequence.len());
    for (instrument, note, velocity) in sequence {
        let instrument = instrument.into();
        event_sequence.push(NoteEvent {
            instrument,
            note,
            velocity,
        });
    }
    event_sequence
}

/// Shortcut for creating a new sequence of polyphonic [`NoteEvent`]:
/// e.g. a sequence of chords
pub fn new_polyphonic_note_sequence<I: Into<Option<InstrumentId>>>(
    polyphonic_sequence: Vec<Vec<(I, u32, f32)>>,
) -> Vec<Vec<NoteEvent>> {
    let mut polyphonic_event_sequence = Vec::with_capacity(polyphonic_sequence.len());
    for sequence in polyphonic_sequence {
        let mut event_sequence = Vec::with_capacity(sequence.len());
        for (instrument, note, velocity) in sequence {
            let instrument = instrument.into();
            event_sequence.push(NoteEvent {
                instrument,
                note,
                velocity,
            });
        }
        polyphonic_event_sequence.push(event_sequence);
    }
    polyphonic_event_sequence
}

/// Shortcut for creating a new [`NoteEvent`] [`EventIter`].
pub fn new_note_event<Instrument: Into<Option<InstrumentId>>>(
    instrument: Instrument,
    note: u32,
    velocity: f32,
) -> FixedEventIter {
    new_note(instrument, note, velocity).to_event()
}

/// Shortcut for creating a new sequence of [`NoteEvent`] [`EventIter`].
pub fn new_note_event_sequence<I: Into<Option<InstrumentId>>>(
    sequence: Vec<(I, u32, f32)>,
) -> EventIterSequence {
    new_note_sequence(sequence).to_event_sequence()
}

/// Shortcut for creating a single [`EventIter`] from multiple [`NoteEvent`]:
/// e.g. a chord.
pub fn new_polyphonic_note_event<I: Into<Option<InstrumentId>>>(
    polyphonic_events: Vec<(I, u32, f32)>,
) -> FixedEventIter {
    new_note_sequence(polyphonic_events).to_event()
}

/// Shortcut for creating a single [`EventIter`] from multiple [`NoteEvent`]:
/// e.g. a sequence of chords.
pub fn new_polyphonic_note_sequence_event<I: Into<Option<InstrumentId>>>(
    polyphonic_sequence: Vec<Vec<(I, u32, f32)>>,
) -> EventIterSequence {
    new_polyphonic_note_sequence(polyphonic_sequence).to_event_sequence()
}

// -------------------------------------------------------------------------------------------------

/// Single parameter change event in a [`Event`].
#[derive(Clone)]
pub struct ParameterChangeEvent {
    pub parameter: Option<ParameterId>,
    pub value: f32,
}

/// Shortcut for creating a new [`ParameterChangeEvent`].
pub fn new_parameter_change<Parameter: Into<Option<ParameterId>>>(
    parameter: Parameter,
    value: f32,
) -> ParameterChangeEvent {
    let parameter: Option<ParameterId> = parameter.into();
    ParameterChangeEvent { parameter, value }
}

/// Shortcut for creating a new [`ParameterChangeEvent`] [`EventIter`].
pub fn new_parameter_change_event<Parameter: Into<Option<ParameterId>>>(
    parameter: Parameter,
    value: f32,
) -> FixedEventIter {
    new_parameter_change(parameter, value).to_event()
}

// -------------------------------------------------------------------------------------------------

/// Event which gets triggered by [`EventIter`].
#[derive(Clone)]
pub enum Event {
    NoteEvents(Vec<NoteEvent>),
    ParameterChangeEvent(ParameterChangeEvent),
}

impl Event {
    pub fn new_note(note: NoteEvent) -> Self {
        Self::NoteEvents(vec![note])
    }
    pub fn new_note_vector(notes: Vec<NoteEvent>) -> Self {
        Self::NoteEvents(notes)
    }
    pub fn new_parameter_change(parameter: Option<ParameterId>, value: f32) -> Self {
        Self::ParameterChangeEvent(ParameterChangeEvent { parameter, value })
    }
}

// -------------------------------------------------------------------------------------------------

/// A resettable [`Event`] iterator, which typically will be used in
/// [Rhythm](`super::Rhythm`) trait impls to sequencially emit new events.
pub trait EventIter: Iterator<Item = Event> {
    /// Reset/rewind the iterator to its initial state.
    fn reset(&mut self);
}

// -------------------------------------------------------------------------------------------------

impl Debug for NoteEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {} {}",
            if self.instrument.is_some() {
                self.instrument.unwrap().to_string()
            } else {
                "NA".to_string()
            },
            self.note,
            self.velocity
        ))
    }
}

impl Debug for ParameterChangeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} {}",
            if self.parameter.is_some() {
                self.parameter.unwrap().to_string()
            } else {
                "NA".to_string()
            },
            self.value,
        ))
    }
}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::NoteEvents(note_vector) => f.write_fmt(format_args!(
                "{:?}",
                note_vector
                    .iter()
                    .map(|f| format!("{:?}", f))
                    .collect::<Vec<String>>()
                    .join(" | ")
            )),
            Event::ParameterChangeEvent(change) => f.write_fmt(format_args!("{:?}", change)),
        }
    }
}