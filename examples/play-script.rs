use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use notify::{RecursiveMode, Watcher};
use simplelog::*;

use afseq::prelude::*;

// -------------------------------------------------------------------------------------------------

#[allow(non_snake_case)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init logging
    TermLogger::init(
        log::STATIC_MAX_LEVEL,
        ConfigBuilder::default().build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap_or_else(|err| {
        log::error!("init_logger error: {:?}", err);
    });

    // preload samples
    let sample_pool = SamplePool::new();
    let KICK = sample_pool.load_sample("assets/kick.wav")?;
    let SNARE = sample_pool.load_sample("assets/snare.wav")?;
    let HIHAT = sample_pool.load_sample("assets/hihat.wav")?;
    let BASS = sample_pool.load_sample("assets/bass.wav")?;
    let SYNTH = sample_pool.load_sample("assets/synth.wav")?;
    let TONE = sample_pool.load_sample("assets/tone.wav")?;
    let FX = sample_pool.load_sample("assets/fx.wav")?;

    // create event player
    let mut player = SamplePlayer::new(Arc::new(RwLock::new(sample_pool)), None)?;

    // set default time base config
    let beat_time = BeatTimeBase {
        beats_per_min: 124.0,
        beats_per_bar: 4,
        samples_per_sec: player.file_player().output_sample_rate(),
    };

    // Watch for script changes, signaling in 'script_files_changed'
    let script_files_changed = Arc::new(AtomicBool::new(false));

    let mut watcher = notify::recommended_watcher({
        let script_files_changed = script_files_changed.clone();
        move |res| match res {
            Ok(event) => {
                log::info!("File change event: {:?}", event);
                script_files_changed.store(true, Ordering::Relaxed);
            }
            Err(err) => log::error!("File watch error: {}", err),
        }
    })?;
    watcher.watch(Path::new("./assets"), RecursiveMode::Recursive)?;

    // (re)run all scripts
    loop {
        if script_files_changed.load(Ordering::Relaxed) {
            script_files_changed.store(false, Ordering::Relaxed);
            log::info!("Rebuilding all rhythms...");
        }

        // build final phrase
        let load = |id: InstrumentId, file_name: &str| {
            if file_name.ends_with(".lua") {
                bindings::lua::new_rhythm_from_file_with_fallback(
                    id,
                    beat_time,
                    format!("./assets/{file_name}").as_str(),
                )
            } else {
                bindings::rhai::new_rhythm_from_file_with_fallback(
                    id,
                    beat_time,
                    format!("./assets/{file_name}").as_str(),
                )
            }
        };
        let phrase = Phrase::new(
            beat_time,
            vec![
                load(KICK, "kick.lua"),
                load(SNARE, "snare.lua"),
                load(HIHAT, "hihat.lua"),
                load(BASS, "bass.lua"),
                load(SYNTH, "synth.lua"),
                load(TONE, "tone.rhai"),
                load(FX, "fx.rhai"),
            ],
            BeatTimeStep::Bar(4.0),
        );

        // wrap phrase into a sequence
        let mut sequence = Sequence::new(beat_time, vec![phrase]);

        let reset_playback_pos = false;
        player.run_until(&mut sequence, &beat_time, reset_playback_pos, || {
            script_files_changed.load(Ordering::Relaxed)
        });
    }
}
