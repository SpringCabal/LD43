//! Sounds and CEGs at units and points.

use alloc::string::String;
use spring::ResultExt;

use spring::generated::owned::effects_control::DefRef;
use spring::types::Float3;

use crate::units;

/// The radius and damage every effect CEG is spawned with.
const CEG_RADIUS: f32 = 20.0;
const CEG_DAMAGE: f32 = 20.0;

pub fn spawn_effect(unit_id: i32, ceg_name: &str) {
    let Some(position) = units::position(unit_id) else {
        return;
    };
    spawn_effect_at(position, ceg_name);
}

pub fn spawn_effect_at(position: Float3, ceg_name: &str) {
    spawn_ceg(position, &ceg(ceg_name));
}

pub fn ceg(name: &str) -> DefRef {
    // A non-negative id takes precedence over the name, so -1 makes the
    // engine look the CEG up by name.
    DefRef {
        name: String::from(name),
        id: -1,
    }
}

pub fn spawn_ceg(position: Float3, ceg: &DefRef) {
    spring::generated::owned::effects_control::spawn_ceg(
        ceg,
        position,
        Float3::default(),
        CEG_RADIUS,
        CEG_DAMAGE,
        0.0,
    )
    .log_err_once(&ceg.name);
}

/// Plays a sound at a unit, at `volume` (Lua's `GG.PlaySound` volume).
pub fn play_sound(name: &str, volume: f32, unit_id: i32) {
    let Some(position) = units::position(unit_id) else {
        return;
    };
    play_sound_at(name, volume, position);
}

pub fn play_sound_at(name: &str, volume: f32, position: Float3) {
    spring::sound::play_sound_file(name, volume, position, Float3::default(), 0).log_err_once(name);
}
