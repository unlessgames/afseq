---@meta
error("Do not try to execute this file. It's just a type definition file.")
---
---Part of the afseq trait: Defines LuaLS annotations for the afseq Rhythm class.
---

----------------------------------------------------------------------------------------------------

---RENOISE SPECIFIC: Optional trigger context passed to `pattern` and 'emit' functions.
---@class TriggerContext
---
---Note value that triggered, started the rhythm, if any.
---@field trigger_note integer?
---Note volume that triggered, started the rhythm, if any.
---@field trigger_volume number?
---Note slice offset value that triggered, started the rhythm, if any.
---@field trigger_offset integer?

----------------------------------------------------------------------------------------------------

---Context passed to `pattern` functions.
---@class PatternContext : TriggerContext
-----Transport playback running.
-----TODO: @field playing boolean
-----Project's tempo in beats per minutes.
---@field beats_per_min number
-----Project's beats per bar setting.
---@field beats_per_bar integer
-----Project's sample rate in samples per second.
---@field sample_rate integer
---
---Continues pulse counter, incrementing with each new **skipped or emitted pulse**.
---Unlike `step` in emitter this includes all pulses, so it also counts pulses which do
---not emit events. Starts from 1 when the rhythm starts running or is reset.
---@field pulse_step integer
---Continues pulse time counter, incrementing with each new **skipped or emitted pulse**.
---Starts from 0 and increases with each new pulse by the pulse's step time duration.
---@field pulse_time_step number
---
---Length of the pattern in pulses (including all pulses from all subdivisions).
---For pattern generator functions this will be the length of the currently emitted 
---pulse or subdivision only as the entire pattern is not predictable.
---@field pattern_length integer
---Pulse counter, which wraps around with the pattern length, incrementing with each 
---new **skipped or emitted pulse**.
---@field pattern_pulse_step integer

----------------------------------------------------------------------------------------------------

---Context passed to 'emit' functions.
---@class EmitterContext : PatternContext
---
---Current pulse's step time as fraction of a full step in the pattern. For simple pulses this
---will be 1, for pulses in subdivisions this will be the reciprocal of the number of steps in the
---subdivision, relative to the parent subdivisions pulse step time.
---### examples:
---```lua
---{1, {1, 1}} --> step times: {1, {0.5, 0.5}}
---```
---@field pulse_time number
---Current pulse value. For binary pulses this will be 1, 0 pulse values will not cause the emitter
---to be called, so they never end up here.
---Values between 0 and 1 will be used as probabilities and thus are maybe emitted or skipped.
---@field pulse_value number
---
---Continues step counter, incrementing with each new *emitted* pulse.
---Unlike `pulse_step` this does not include skipped, zero values pulses so it basically counts
---how often the emit function already got called.
---Starts from 1 when the rhythm starts running or is reset.
---@field step integer

----------------------------------------------------------------------------------------------------

---Single pulse value or a nested subdivion of pulses within a pattern.
---@alias Pulse (0|1|number|boolean|nil)|(Pulse)[]

----------------------------------------------------------------------------------------------------

---Construction options for a new rhythm.
---@class RhythmOptions
---
---Base time unit of the emitter. Use `resolution` to apply an additional factor, in order to
---create other less common rhythm bases.
---### examples:
---```lua
---unit = "beats", resolution = 1.01 --> slightly off beat pulse
---unit = "1/16", resolution = 4/3 --> tripplet
---```
---@field unit "ms"|"seconds"|"bars"|"beats"|"1/1"|"1/2"|"1/4"|"1/8"|"1/16"|"1/32"|"1/64"
---Factor which is applied on `unit` to specify the final time resolution of the emitter.
---### examples:
---```lua
---unit = "beats", resolution = 1.01 --> slightly off beat pulse
---unit = "1/16", resolution = 4/3 --> tripplet
---```
---@field resolution number?
---
---Optional offset in `unit * resolution` time units. By default 0.
---When set, the rhythm's event output will be delayed by the given offset value.
---### examples:
---```lua
---unit = "1/4",
---resolution = 4,
---offset = 4 -- start emitting after 4*4 beats
---```  
---@field offset number?
---
---Specify the rythmical pattern of the emitter. Each pulse with a value of 1 or true
---will cause an event from the `emitter` property to be triggered in the emitters
---time unit. 0 or nil values never trigger, and values inbetween do *maybe* trigger.
---
---To create deterministic random patterns, seed the random number generator before
---creating the rhythm via `math.randomseed(some_seed)`
---
---Patterns can contains subdivisions, sub tables of pulses, to "cram" multiple pulses
---into a single pulse's time interval. This way more complex rhythmical patterns can
---be created.
---
---When no pattern is defined, a constant pulse of `1` is triggered by the rhythm.
---
---Just like the `emitter` property, patterns can either be a fixed array of values or a
---function or iterator which produces values dynamically.
---
---### examples:
---```lua
----- a fixed pattern
---pattern = { 1, 0, 0, 1 }
----- maybe trigger with probabilities
---pattern = { 1, 0, 0.5, 0.9 }
----- "cram" pulses into a sigle pulse slot via subdivisions
---pattern = { 1, { 1, 1, 1 } }
---
----- fixed pattern with require "pattern"
---pattern = pattern.from{ 1, 0 } * 3 + { 1, 1 }
---pattern = pattern.euclidean(7, 16, 2)
---
----- stateless generator function
---pattern = function(context)
---  return math.random(0, 1)
---end
---
----- statefull generator function
---pattern = function(context)
---  local mypattern = table.create({0, 6, 10})
---  ---@param context EmitterContext
---  return function(context)
---    return mypattern:find((context.step - 1) % 16) ~= nil
---  end
---end
---
----- 'fun' iterator as pulse generator
---local fun = require "fun"
---pattern = fun.rands(5, 10):map(function(x) return x / 10.0; end):take(12):cycle()
---
---```
---@field pattern Pulse[]|(fun(context: PatternContext):Pulse)|(fun(context: PatternContext):fun(context: PatternContext):Pulse)?
---
---If and how many times a pattern should repeat. When 0 or false, the pattern does not repeat 
---and plays back only once. When true, the pattern repeats endlessly, which is the default.
---When a number > 0, this specifies the number of times the pattern repeats until it stops.
---
---Note: When `pattern` is a function or iterator, the repeat count is the number of 
---*function calls or iteration steps*. When the pattern is a pulse array, this is the number of
---times the whole pattern gets repeated.
---
---### examples:
---```lua
---repeat = 0 -- one-shot
---repeat = false -- also a one-shot
---repeat = 3 -- play the pattern 4 times
---repeat = true -- play & repeat forever
---```
---@field repeats (integer|boolean)?
---
---Specify the melodic pattern of the rhythm. For every pulse in the rhythmical pattern, the
---next event from the specified emit sequence gets triggered. When the end of the sequence is
---reached, it restarts from the beginning.<br>
---In order to dynamically generate notes, you can pass a function or a functions iterator, 
---instead of a fixed note array or sequence.
---
---### examples:
---```lua
----- a sequence of c4, g4
---emit = {"c4", "g4"}
----- a chord of c4, d#4, g4
---emit = {{"c4", "d#4", "g4"}} -- or {"c4'min"}
----- a sequence of c4, g4 with volume 0.5
---emit = sequence{"c4", "g4"}:with_volume(0.5)
---
----- stateless generator function
----- a function
---emit = function(context)
---  return 48 + math.random(1, 4) * 5
---end
---
----- statefull generator function
---emit = function(initial_context) 
---  local count, step, notes = 1, 2, scale("c5", "minor").notes
---  ---@param context EmitterContext
---  return function(context)
---    local key = notes[count]
---    count = (count + step - 1) % #notes + 1
---    return { key = key, volume = 0.5 }
---  end
---end
---
------ a fun iterator
---local fun = require "fun"
---local cmin = scale("c5", "minor")
---local degree = function(x) return cmin:degree(x) end 
---...
---emit = fun.iter({1, 5, 2, 7, 3, 5, 3}):map(degree):cycle()
---
----- a note pattern
---local pattern = require "pattern"
---local tritone = scale("c5", "tritone")
---...
---emit = pattern.from(tritone:chord(1, 4)):euclidean(6) +
---  pattern.from(tritone:chord(5, 4)):euclidean(6)
---
---```
---@field emit Sequence|Note|NoteValue|(NoteValue|Note)[]|(fun(context: EmitterContext):NoteValue)|(fun(context: EmitterContext):fun(context: EmitterContext):NoteValue)


----------------------------------------------------------------------------------------------------

---Create a new rhythm with the given configuration.
---
---### examples:
---```lua
----- trigger a chord sequence every 4 bars after 4 bars
---return rhythm {
---  unit = "bars",
---  resolution = 4,
---  offset = 1,
---  emit = sequence("c4'm", note("g3'm7"):transposed({0, 12, 0, 0}))
---}
---
-----trigger notes in an euclidean tripplet pattern
---local pattern = require "pattern"
---return rhythm {
---  unit = "1/8",
---  resolution = 3/2,
---  pattern = pattern.euclidean(6, 16, 2),
---  emit = sequence("c3", "c3", note{ "c4", "a4" }:with_volume(0.75))
---}
---
-----trigger notes in a seeded, random subdivision pattern
---math.randomseed(23498)
---return rhythm {
---  unit = "1/8",
---  pattern = { 1, { 0, 1 }, 0, 0.3, 0.2, 1, { 0.5, 0.1, 1 }, 0.5 },
---  emit = { "c4" },
---}
-----trigger random notes in a random pattern from a pentatonic scale
---return rhythm {
---  unit = "1/16",
---  pattern = function(context)
---    return (context.pulse_step % 4 == 1) or (math.random() > 0.8)
---  end,
---  emit = function(context)
---    local cmin = scale("c5", "pentatonic minor").notes
---    return function(context)
---      return { key = cmin[math.random(#cmin)], volume = 0.7 }
---    end
---  end
---}
---```
---@param options RhythmOptions
---@return userdata
function rhythm(options) end
