use std::{cell::RefCell, rc::Rc, sync::Arc};

use mlua::prelude::*;

use crate::{event::scripted::ScriptedEventIter, prelude::*};

// ---------------------------------------------------------------------------------------------

// Error helpers
pub fn bad_argument_error<S1: Into<Option<&'static str>>, S2: Into<Option<&'static str>>>(
    func: S1,
    arg: S2,
    pos: usize,
    message: &str,
) -> mlua::Error {
    mlua::Error::BadArgument {
        to: func.into().map(String::from),
        name: arg.into().map(String::from),
        pos,
        cause: Arc::new(mlua::Error::RuntimeError(message.to_string())),
    }
}

// ---------------------------------------------------------------------------------------------

// Note Userdata in bindings
#[derive(Clone, Debug)]
pub struct NoteUserData {
    pub notes: Vec<Option<NoteEvent>>,
}

impl NoteUserData {
    pub fn from(
        args: LuaMultiValue,
        default_instrument: Option<InstrumentId>,
    ) -> mlua::Result<Self> {
        // single value, probably a table?
        if args.len() == 1 {
            let arg = args
                .iter()
                .next()
                .ok_or(mlua::Error::RuntimeError(
                    "Failed to access table content".to_string(),
                ))?
                .clone();
            Ok(NoteUserData {
                notes: note_events_from_value(arg, None, default_instrument)?,
            })
        // multiple values, maybe of different type
        } else {
            let mut notes = vec![];
            for (index, arg) in args.into_iter().enumerate() {
                notes.push(note_event_from_value(arg, Some(index), default_instrument)?);
            }
            Ok(NoteUserData { notes })
        }
    }
}

impl LuaUserData for NoteUserData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("with_volume", |lua, this, volume: LuaValue| {
            let volumes = volume_factors_from_value(lua, volume, this.notes.len())?;
            for (note, volume) in this.notes.iter_mut().zip(volumes.into_iter()) {
                if let Some(note) = note {
                    note.volume = volume;
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("amplify", |lua, this, volume: LuaValue| {
            let volumes = volume_factors_from_value(lua, volume, this.notes.len())?;
            for (note, volume) in this.notes.iter_mut().zip(volumes.into_iter()) {
                if let Some(note) = note {
                    note.volume *= volume;
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("transpose", |lua, this, volume: LuaValue| {
            let steps = transpose_steps_from_value(lua, volume, this.notes.len())?;
            for (note, step) in this.notes.iter_mut().zip(steps.into_iter()) {
                if let Some(note) = note {
                    if note.note.is_note_on() {
                        let transposed_note = (u8::from(note.note) as i32 + step).max(0).min(0x7f);
                        note.note = Note::from(transposed_note as u8);
                    }
                }
            }
            Ok(this.clone())
        });
    }
}

// Sequence
#[derive(Clone, Debug)]
pub struct SequenceUserData {
    pub notes: Vec<Vec<Option<NoteEvent>>>,
}

impl SequenceUserData {
    pub fn from(
        args: LuaMultiValue,
        default_instrument: Option<InstrumentId>,
    ) -> mlua::Result<Self> {
        // single value, probably a table?
        if args.len() == 1 {
            let arg = args
                .iter()
                .next()
                .ok_or(mlua::Error::RuntimeError(
                    "Failed to access table content".to_string(),
                ))?
                .clone();
            Ok(SequenceUserData {
                notes: note_events_from_value(arg, None, default_instrument)?
                    .into_iter()
                    .map(|v| vec![v])
                    .collect::<Vec<Vec<_>>>(),
            })
        // multiple values, maybe of different type
        } else {
            let mut notes = vec![];
            for (index, arg) in args.into_iter().enumerate() {
                notes.push(note_events_from_value(
                    arg,
                    Some(index),
                    default_instrument,
                )?);
            }
            Ok(SequenceUserData { notes })
        }
    }
}

impl LuaUserData for SequenceUserData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("with_volume", |lua, this, volume: LuaValue| {
            let volumes = volume_factors_from_value(lua, volume, this.notes.len())?;
            for (notes, volume) in this.notes.iter_mut().zip(volumes) {
                for note in notes.iter_mut().flatten() {
                    note.volume = volume;
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("amplify", |lua, this, volume: LuaValue| {
            let volumes = volume_factors_from_value(lua, volume, this.notes.len())?;
            for (notes, volume) in this.notes.iter_mut().zip(volumes) {
                for note in notes.iter_mut().flatten() {
                    note.volume *= volume;
                }
            }
            Ok(this.clone())
        });

        methods.add_method_mut("transpose", |lua, this, volume: LuaValue| {
            let steps = transpose_steps_from_value(lua, volume, this.notes.len())?;
            for (notes, step) in this.notes.iter_mut().zip(steps.into_iter()) {
                for note in notes.iter_mut().flatten() {
                    if note.note.is_note_on() {
                        let transposed_note = (u8::from(note.note) as i32 + step).max(0).min(0x7f);
                        note.note = Note::from(transposed_note as u8);
                    }
                }
            }
            Ok(this.clone())
        });
    }
}

// ---------------------------------------------------------------------------------------------

fn volume_from_string(str: &str) -> mlua::Result<f32> {
    let mut volume = 1.0;
    if !str.is_empty() {
        if let Ok(int) = str.parse::<i32>() {
            volume = int as f32;
        } else if let Ok(float) = str.parse::<f32>() {
            volume = float;
        } else {
            return Err(mlua::Error::FromLuaConversionError {
                from: "string",
                to: "Note",
                message: Some(format!(
                    "Failed to parse volume: \
                        Argument '{}' is neither a float or int value",
                    str
                )),
            });
        }
        if volume < 0.0 {
            return Err(mlua::Error::FromLuaConversionError {
                from: "string",
                to: "Note",
                message: Some(format!(
                    "Failed to parse volume propery in node: \
                        Volume must be >= 0 but is '{}",
                    volume
                )),
            });
        }
    }
    Ok(volume)
}

fn volume_factors_from_value(lua: &Lua, volume: LuaValue, len: usize) -> mlua::Result<Vec<f32>> {
    let volumes;
    if let Some(volume_table) = volume.as_table() {
        volumes = volume_table
            .clone()
            .sequence_values::<f32>()
            .enumerate()
            .map(|(_, result)| result)
            .collect::<mlua::Result<Vec<f32>>>()?;
    } else {
        let volume = f32::from_lua(volume, lua)?;
        volumes = (0..len).map(|_| volume).collect::<Vec<f32>>()
    }
    for volume in volumes.iter().copied() {
        if volume < 0.0 {
            return Err(bad_argument_error(
                None,
                "volume",
                1,
                format!("Volume must be >= 0 but is '{}", volume).as_str(),
            ));
        }
    }
    Ok(volumes)
}

fn transpose_steps_from_value(lua: &Lua, step: LuaValue, len: usize) -> mlua::Result<Vec<i32>> {
    let steps;
    if let Some(volume_table) = step.as_table() {
        steps = volume_table
            .clone()
            .sequence_values::<i32>()
            .enumerate()
            .map(|(_, result)| result)
            .collect::<mlua::Result<Vec<i32>>>()?;
    } else {
        let step = i32::from_lua(step, lua)?;
        steps = (0..len).map(|_| step).collect::<Vec<i32>>()
    }
    Ok(steps)
}

// ---------------------------------------------------------------------------------------------

fn is_empty_note_string(s: &str) -> bool {
    matches!(s, "" | "-" | "--" | "---" | "." | ".." | "...")
}

pub fn note_event_from_number(
    note_value: i64,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Option<NoteEvent>> {
    Ok(Some(NoteEvent {
        note: Note::from(note_value as u8),
        volume: 1.0,
        instrument: default_instrument,
    }))
}

pub fn note_event_from_string(
    str: &str,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Option<NoteEvent>> {
    let mut white_space_splits = str.split(' ').filter(|v| !v.is_empty());
    let note_part = white_space_splits.next().unwrap_or("");
    if is_empty_note_string(note_part) {
        Ok(None)
    } else {
        let note =
            Note::try_from(note_part).map_err(|err| mlua::Error::FromLuaConversionError {
                from: "string",
                to: "Note",
                message: Some(format!("Invalid note value '{}': {}", note_part, err)),
            })?;
        let volume = {
            if let Some(volume_part) = white_space_splits.next() {
                volume_from_string(volume_part)?
            } else {
                1.0
            }
        };
        Ok(Some(NoteEvent {
            instrument: default_instrument,
            note,
            volume,
        }))
    }
}

pub fn chord_events_from_string(
    str: &str,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Vec<Option<NoteEvent>>> {
    let mut white_space_splits = str.split(' ').filter(|v| !v.is_empty());
    let chord_part = white_space_splits.next().unwrap_or("");
    let chord = Chord::try_from(chord_part).map_err(|err| mlua::Error::FromLuaConversionError {
        from: "string",
        to: "Note",
        message: Some(format!("Invalid chord value '{}': {}", chord_part, err)),
    })?;
    let volume = {
        if let Some(volume_part) = white_space_splits.next() {
            volume_from_string(volume_part)?
        } else {
            1.0
        }
    };
    Ok(chord
        .intervals
        .iter()
        .map(|i| {
            Some(NoteEvent {
                instrument: default_instrument,
                note: Note::from(chord.note as u8 + i),
                volume,
            })
        })
        .collect::<Vec<_>>())
}

pub fn note_event_from_table(
    table: LuaTable,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Option<NoteEvent>> {
    if table.is_empty() {
        return Ok(None);
    }
    if table.contains_key("key")? {
        // get optional volume value
        let volume = if table.contains_key::<&str>("volume")? {
            if let Ok(value) = table.get::<&str, f32>("volume") {
                if value < 0.0 {
                    return Err(mlua::Error::FromLuaConversionError {
                        from: "string",
                        to: "Note",
                        message: Some("Invalid note volume value".to_string()),
                    });
                } else {
                    value
                }
            } else {
                return Err(mlua::Error::FromLuaConversionError {
                    from: "string",
                    to: "Note",
                    message: Some("Invalid note volume value".to_string()),
                });
            }
        } else {
            1.0
        };
        // { key = 60, [volume = 1.0] }
        if let Ok(note_value) = table.get::<&str, u8>("key") {
            let note = Note::from(note_value);
            Ok(Some(NoteEvent {
                note,
                volume,
                instrument: default_instrument,
            }))
        }
        // { key = "C4", [volume = 1.0] }
        else if let Ok(note_str) = table.get::<&str, String>("key") {
            if let Ok(note) = Note::try_from(note_str.as_str()) {
                Ok(Some(NoteEvent {
                    note,
                    volume,
                    instrument: default_instrument,
                }))
            } else {
                Err(mlua::Error::FromLuaConversionError {
                    from: "string",
                    to: "Note",
                    message: Some(format!("Invalid note value: '{}'", note_str)),
                })
            }
        } else {
            Err(mlua::Error::FromLuaConversionError {
                from: "table",
                to: "Note",
                message: Some("Table does not contain a valid 'key' property".to_string()),
            })
        }
    } else {
        Err(mlua::Error::FromLuaConversionError {
            from: "table",
            to: "Note",
            message: Some("Table does not contain valid note properties".to_string()),
        })
    }
}

pub fn note_event_from_value(
    arg: LuaValue,
    arg_index: Option<usize>,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Option<NoteEvent>> {
    match arg {
        LuaValue::Nil => Ok(None),
        LuaValue::Integer(note_value) => note_event_from_number(note_value, default_instrument),
        LuaValue::String(str) => note_event_from_string(&str.to_string_lossy(), default_instrument),
        LuaValue::Table(table) => note_event_from_table(table, default_instrument),
        _ => {
            return Err(mlua::Error::FromLuaConversionError {
                from: arg.type_name(),
                to: "Note",
                message: if let Some(index) = arg_index {
                    Some(
                        format!("Note arg #{} does not contain a valid note property", index)
                            .to_string(),
                    )
                } else {
                    Some("Argument does not contain a valid note property".to_string())
                },
            });
        }
    }
}

pub fn note_events_from_value(
    arg: LuaValue,
    arg_index: Option<usize>,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Vec<Option<NoteEvent>>> {
    match arg {
        LuaValue::UserData(userdata) => {
            if userdata.is::<SequenceUserData>() {
                Err(mlua::Error::FromLuaConversionError {
                    from: "Sequence",
                    to: "Note",
                    message: Some("Can not nest sequences into sequences".to_string()),
                })
            } else if let Ok(chord) = userdata.take::<NoteUserData>() {
                Ok(chord.notes)
            } else {
                Err(mlua::Error::FromLuaConversionError {
                    from: "UserData",
                    to: "Note",
                    message: if let Some(index) = arg_index {
                        Some(
                            format!(
                                "Sequence arg #{} does not contain a valid note property",
                                index
                            )
                            .to_string(),
                        )
                    } else {
                        Some("Argument does not contain a valid note property".to_string())
                    },
                })
            }
        }
        LuaValue::Table(table) => {
            // array like { "C4", "C5" }
            if table.clone().sequence_values::<LuaValue>().count() > 0 {
                let mut note_events = vec![];
                for (arg_index, arg) in table.sequence_values::<LuaValue>().enumerate() {
                    let value = arg?;
                    note_events.push(note_event_from_value(
                        value,
                        Some(arg_index),
                        default_instrument,
                    )?);
                }
                Ok(note_events)
            // { key = xxx } struct
            } else {
                Ok(vec![note_event_from_value(
                    mlua::Value::Table(table),
                    arg_index,
                    default_instrument,
                )?])
            }
        }
        LuaValue::String(str) => {
            let str = str.to_string_lossy().to_string();
            // a string with ' is a chord
            if str.contains('\'') {
                Ok(chord_events_from_string(&str, default_instrument)?)
            } else {
                Ok(vec![note_event_from_string(&str, default_instrument)?])
            }
        }
        _ => Ok(vec![note_event_from_value(
            arg,
            arg_index,
            default_instrument,
        )?]),
    }
}

// -------------------------------------------------------------------------------------------------

pub fn event_iter_from_value(
    value: LuaValue,
    default_instrument: Option<InstrumentId>,
) -> mlua::Result<Rc<RefCell<dyn EventIter>>> {
    match value {
        LuaValue::UserData(userdata) => {
            if userdata.is::<NoteUserData>() {
                let note = userdata.take::<NoteUserData>()?;
                Ok(Rc::new(RefCell::new(note.notes.to_event())))
            } else if userdata.is::<SequenceUserData>() {
                let sequence = userdata.take::<SequenceUserData>()?;
                Ok(Rc::new(RefCell::new(sequence.notes.to_event_sequence())))
            } else {
                Err(mlua::Error::FromLuaConversionError {
                    from: "table",
                    to: "Note",
                    message: Some("Invalid note table argument".to_string()),
                })
            }
        }
        LuaValue::Function(function) => {
            let iter = ScriptedEventIter::new(function, default_instrument)?;
            Ok(Rc::new(RefCell::new(iter)))
        }
        _ => {
            let iter = note_event_from_value(value, None, default_instrument)?.to_event();
            Ok(Rc::new(RefCell::new(iter)))
        }
    }
}
