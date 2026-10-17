use alloc::string::String;
use spring::ResultExt;

use spring::Float3;
use spring::effects_control::DefRef;

use crate::units;

const CEG_RADIUS: f32 = 20.0;
const CEG_DAMAGE: f32 = 20.0;

pub fn spawn_effect(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    unit_id: i32,
    ceg_name: &str,
) {
    let Some(position) = units::position(spring, unit_id) else {
        return;
    };
    spawn_effect_at(spring, position, ceg_name);
}

pub fn spawn_effect_at(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    position: Float3,
    ceg_name: &str,
) {
    spawn_ceg(spring, position, &ceg(ceg_name));
}

pub fn ceg(name: &str) -> DefRef {
    DefRef {
        name: String::from(name),
        id: -1,
    }
}

pub fn spawn_ceg(spring: &spring::Spring<'_, spring::RulesSynced>, position: Float3, ceg: &DefRef) {
    spring
        .effects_control()
        .spawn_ceg(
            ceg,
            position,
            Float3::default(),
            CEG_RADIUS,
            CEG_DAMAGE,
            0.0,
        )
        .log_err_once(spring, &ceg.name);
}

pub fn play_sound(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    name: &str,
    volume: f32,
    unit_id: i32,
) {
    let Some(position) = units::position(spring, unit_id) else {
        return;
    };
    play_sound_at(spring, name, volume, position);
}

pub fn play_sound_at(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    name: &str,
    volume: f32,
    position: Float3,
) {
    spring
        .sound()
        .play_sound_file(name, volume, position, Float3::default(), 0)
        .log_err_once(spring, name);
}
