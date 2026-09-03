use crate::diagnostics;
use crate::state;

const MUSIC_FILE: &str = "sounds/music.ogg";
const MUSIC_VOLUME: f32 = 0.20;

pub fn update() {
    let should_start = state::with_state(|ui| {
        if ui.music_started {
            false
        } else {
            ui.music_started = true;
            true
        }
    })
    .unwrap_or(false);
    if !should_start {
        return;
    }
    match spring::sound::play_sound_stream(MUSIC_FILE, MUSIC_VOLUME, true) {
        Ok(true) => diagnostics::info("music stream started"),
        Ok(false) | Err(_) => {
            let _ = state::with_state(|ui| ui.music_started = false);
            diagnostics::warning("music stream could not be started");
        }
    }
}
