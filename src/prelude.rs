//! The afseq prelude.
//!
//! The purpose of this module is to alleviate imports of common afseq traits:
//!
//! ```
//! # #![allow(unused_imports)]
//! use afseq::prelude::*;
//! ```

pub use super::{
    event::{
        fixed::ToFixedEventValue, mapped::ToMappedEventIter, mapped_note::ToMappedNotesEventIter,
        new_note_event, new_note_event_sequence, new_parameter_change_event,
        new_polyphonic_note_event, new_polyphonic_note_sequence_event,
        sequence::ToEventIterSequence, unique_instrument_id, InstrumentId, ParameterId,
    },
    time::{BeatTimeBase, BeatTimeStep, SecondTimeBase, SecondTimeStep, TimeBase},
    Event, EventIter, Phrase, Rhythm, SampleTime,
};