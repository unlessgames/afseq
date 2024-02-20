//! Events and event iterators which get emitted by a `Rhythm`.

use crate::note::Note;
use fixed::{FixedEventIter, ToFixedEventIter, ToFixedEventIterSequence};

use derive_more::{Deref, Display, From, Into};

use core::{
    fmt::Debug,
    fmt::Display,
    sync::atomic::{AtomicUsize, Ordering},
};

pub mod empty;
pub mod fixed;
pub mod mutated;
#[cfg(feature = "scripting")]
pub mod scripted;

// -------------------------------------------------------------------------------------------------

/// Id to refer to a specific instrument in a NoteEvent.
#[derive(Copy, Clone, Debug, Display, Deref, From, Into, PartialEq, Eq, Hash)]
pub struct InstrumentId(usize);

/// Id to refer to a specific parameter in a ParameterChangeEvent.
#[derive(Copy, Clone, Debug, Display, Deref, From, Into, PartialEq, Eq, Hash)]
pub struct ParameterId(usize);

// -------------------------------------------------------------------------------------------------

/// Generate a new unique instrument id.
pub fn unique_instrument_id() -> InstrumentId {
    static ID: AtomicUsize = AtomicUsize::new(0);
    InstrumentId(ID.fetch_add(1, Ordering::Relaxed))
}

// -------------------------------------------------------------------------------------------------

/// Single note event in a [`Event`].
#[derive(Clone, PartialEq, Debug)]
pub struct NoteEvent {
    pub instrument: Option<InstrumentId>,
    pub note: Note,
    pub volume: f32,  // [0 - INF]
    pub panning: f32, // [-1 - 1]
    pub delay: f32,   // [0 - 1]
}

impl NoteEvent {
    pub fn to_string(&self, show_instruments: bool) -> String {
        if show_instruments {
            format!(
                "{} {} {:.2} {:.2} {:.2}",
                if let Some(instrument) = self.instrument {
                    format!("{:02}", instrument)
                } else {
                    "NA".to_string()
                },
                self.note,
                self.volume,
                self.panning,
                self.delay
            )
        } else {
            format!(
                "{} {:.2} {:.2} {:.2}",
                self.note, self.volume, self.panning, self.delay
            )
        }
    }
}

impl<I: Into<Option<InstrumentId>>, N: TryInto<Note>> From<(I, N)> for NoteEvent
where
    <N as TryInto<Note>>::Error: std::fmt::Debug,
{
    // Initialize from a (Instrument, Note) tuple
    fn from((instrument, note): (I, N)) -> Self {
        let note = note.try_into().expect("Failed to convert note");
        let instrument = instrument.into();
        Self {
            note,
            instrument,
            volume: 1.0,
            panning: 0.0,
            delay: 0.0,
        }
    }
}

impl<I: Into<Option<InstrumentId>>, N: TryInto<Note>> From<(I, N, f32)> for NoteEvent
where
    <N as TryInto<Note>>::Error: std::fmt::Debug,
{
    // Initialize from a (Instrument, Note, Volume) tuple
    fn from((instrument, note, volume): (I, N, f32)) -> Self {
        let note = note.try_into().expect("Failed to convert note");
        let instrument = instrument.into();
        Self {
            note,
            instrument,
            volume,
            panning: 0.0,
            delay: 0.0,
        }
    }
}

impl<I: Into<Option<InstrumentId>>, N: TryInto<Note>> From<(I, N, f32, f32)> for NoteEvent
where
    <N as TryInto<Note>>::Error: std::fmt::Debug,
{
    // Initialize from a (Instrument, Note, Volume, Panning) tuple
    fn from((instrument, note, volume, panning): (I, N, f32, f32)) -> Self {
        let note = note.try_into().expect("Failed to convert note");
        let instrument = instrument.into();
        Self {
            note,
            instrument,
            volume,
            panning,
            delay: 0.0,
        }
    }
}

impl<I: Into<Option<InstrumentId>>, N: TryInto<Note>> From<(I, N, f32, f32, f32)> for NoteEvent
where
    <N as TryInto<Note>>::Error: std::fmt::Debug,
{
    // Initialize from a (Instrument, Note, Volume, Panning, Delay) tuple
    fn from((instrument, note, volume, panning, delay): (I, N, f32, f32, f32)) -> Self {
        let note = note.try_into().expect("Failed to convert note");
        let instrument = instrument.into();
        Self {
            note,
            instrument,
            volume,
            panning,
            delay,
        }
    }
}

impl Display for NoteEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const SHOW_INSTRUMENTS: bool = true;
        f.write_fmt(format_args!("{}", self.to_string(SHOW_INSTRUMENTS)))
    }
}

/// Shortcut for creating an empty [`NoteEvent`]
pub fn new_empty_note() -> Option<NoteEvent> {
    None
}

/// Shortcut for creating a new [`NoteEvent`].
pub fn new_note<E: Into<NoteEvent>>(note_event: E) -> Option<NoteEvent> {
    Some(note_event.into())
}

/// Shortcut for creating a vector of [`NoteEvent`]:
/// e.g. a sequence of single notes
pub fn new_note_vector<E: Into<NoteEvent>>(
    sequence: Vec<Option<E>>,
) -> Vec<Option<NoteEvent>> {
    let mut event_sequence = Vec::with_capacity(sequence.len());
    for event in sequence {
        if let Some(event) = event {
            event_sequence.push(Some(event.into()));
        } else {
            event_sequence.push(None);
        }
    }
    event_sequence
}

/// Shortcut for creating a new sequence of polyphonic [`NoteEvent`]:
/// e.g. a sequence of chords
pub fn new_polyphonic_note_sequence<E: Into<NoteEvent>>(
    polyphonic_sequence: Vec<Vec<Option<E>>>,
) -> Vec<Vec<Option<NoteEvent>>> {
    let mut polyphonic_event_sequence = Vec::with_capacity(polyphonic_sequence.len());
    for sequence in polyphonic_sequence {
        let mut event_sequence = Vec::with_capacity(sequence.len());
        for event in sequence {
            if let Some(event) = event {
                event_sequence.push(Some(event.into()));
            } else {
                event_sequence.push(None)
            }
        }
        polyphonic_event_sequence.push(event_sequence);
    }
    polyphonic_event_sequence
}

/// Shortcut for creating a new empty [`NoteEvent`] [`EventIter`].
pub fn new_empty_note_event() -> FixedEventIter {
    new_empty_note().to_event()
}

/// Shortcut for creating a new [`NoteEvent`] [`EventIter`].
pub fn new_note_event<E: Into<NoteEvent>>(
    event: E,
) -> FixedEventIter
{
    new_note(event).to_event()
}

/// Shortcut for creating a new sequence of [`NoteEvent`] [`EventIter`].
pub fn new_note_event_sequence<E: Into<NoteEvent>>(
    sequence: Vec<Option<E>>,
) -> FixedEventIter {
    new_note_vector(sequence).to_event_sequence()
}

/// Shortcut for creating a single [`EventIter`] from multiple [`NoteEvent`]:
/// e.g. a chord.
pub fn new_polyphonic_note_event<E: Into<NoteEvent>>(
    polyphonic_events: Vec<Option<E>>,
) -> FixedEventIter {
    new_note_vector(polyphonic_events).to_event()
}

/// Shortcut for creating a single [`EventIter`] from multiple [`NoteEvent`]:
/// e.g. a sequence of chords.
pub fn new_polyphonic_note_sequence_event<E: Into<NoteEvent>>(
    polyphonic_sequence: Vec<Vec<Option<E>>>,
) -> FixedEventIter {
    new_polyphonic_note_sequence(polyphonic_sequence).to_event_sequence()
}

// -------------------------------------------------------------------------------------------------

/// Single parameter change event in a [`Event`].
#[derive(Clone, PartialEq, Debug)]
pub struct ParameterChangeEvent {
    pub parameter: Option<ParameterId>,
    pub value: f32,
}

impl ParameterChangeEvent {
    pub fn to_string(&self, show_parameter: bool) -> String {
        if show_parameter {
            format!(
                "{} {:.3}",
                if let Some(parameter) = self.parameter {
                    format!("{:02}", parameter)
                } else {
                    "NA".to_string()
                },
                self.value,
            )
        } else {
            format!("{:.3}", self.value)
        }
    }
}

impl Display for ParameterChangeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const SHOW_PARAMETERS: bool = true;
        f.write_fmt(format_args!("{}", self.to_string(SHOW_PARAMETERS)))
    }
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

/// Event which gets emitted by an [`EventIter`].
#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    NoteEvents(Vec<Option<NoteEvent>>),
    ParameterChangeEvent(ParameterChangeEvent),
}

impl Event {
    pub fn to_string(&self, show_instruments_and_parameters: bool) -> String {
        match self {
            Event::NoteEvents(note_vector) => note_vector
                .iter()
                .map(|n| {
                    if let Some(v) = n {
                        v.to_string(show_instruments_and_parameters)
                    } else {
                        "---".to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(" | "),
            Event::ParameterChangeEvent(change) => {
                change.to_string(show_instruments_and_parameters)
            }
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const SHOW_INSTRUMENTS_AND_PARAMETERS: bool = true;
        f.write_fmt(format_args!(
            "{}",
            self.to_string(SHOW_INSTRUMENTS_AND_PARAMETERS)
        ))
    }
}

// -------------------------------------------------------------------------------------------------

/// A resettable [`Event`] iterator, which typically will be used in
/// [Rhythm](`super::Rhythm`) trait impls to sequencially emit new events.
pub trait EventIter: Iterator<Item = Event> + Debug {
    /// Reset/rewind the iterator to its initial state.
    fn reset(&mut self);
}
