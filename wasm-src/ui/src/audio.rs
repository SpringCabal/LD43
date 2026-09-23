use core::cell::Cell;

use crate::diagnostics;

const MUSIC_FILE: &str = "sounds/music.ogg";
const MUSIC_VOLUME: f32 = 0.20;

/// Starts the looping music, retrying until it plays.
pub fn update(started: &Cell<bool>) {
    if started.replace(true) {
        return;
    }
    match spring::sound::play_sound_stream(MUSIC_FILE, MUSIC_VOLUME, true) {
        Ok(true) => diagnostics::info("music stream started"),
        Ok(false) | Err(_) => {
            started.set(false);
            diagnostics::warning("music stream could not be started");
        }
    }
}
