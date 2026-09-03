use alloc::vec::Vec;

use crate::state;

use super::particles::{self, Particle};

pub fn update(delta_seconds: f32) {
    let frame = state::with_state(|ui| ui.game_frame).unwrap_or(0);
    if frame != state::with_state(|ui| ui.fire_frame).unwrap_or(frame) {
        let _ = state::with_state(|ui| ui.fire_frame = frame);
        if frame % CHECK_INTERVAL == 0 {
            emit(frame);
        }
    }
    let _ = state::with_state(|ui| ui.fire.advance(delta_seconds));
}

const CHECK_INTERVAL: i32 = 6;
const WIND_SCALE: f32 = 0.09;
const DISC_RADIUS_SCALE: f32 = 0.6;
const SIZE_SCALE: f32 = 0.35;
const EMIT_CHANCE_RANGE: i32 = 400;

fn emit(frame: i32) {
    let burning = burning_units();
    if burning.is_empty() {
        return;
    }
    apply_wind();
    let count = burning.len() as i32;
    let mut seed = frame as u64;
    for unit_id in burning {
        seed = next(seed);
        if (unit_float(seed) * EMIT_CHANCE_RANGE as f32) as i32 >= EMIT_CHANCE_RANGE - count {
            continue;
        }
        let Ok(position) = spring::units_info::get_unit_position(
            unit_id,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: false,
                aim_pos: false,
            },
        ) else {
            continue;
        };
        let Ok(radius) = spring::units_info::get_unit_radius(unit_id) else {
            continue;
        };
        for _ in 0..particles::PARTICLES_PER_EMIT {
            seed = next(seed);
            spawn(position, radius, seed);
        }
    }
}

fn burning_units() -> Vec<i32> {
    spring::get_all_units()
        .unwrap_or_default()
        .into_iter()
        .filter(|unit_id| {
            spring::get_unit_rules_param_f32(*unit_id, "on_fire").unwrap_or(0.0) >= 0.5
        })
        .collect()
}

fn apply_wind() {
    let Ok(wind) = spring::game::get_wind(0) else {
        return;
    };
    let _ = state::with_state(|ui| ui.fire.set_force(0.0, wind.current * WIND_SCALE + 3.0, 0.0));
}

fn spawn(position: spring::types::Float3, radius: f32, seed: u64) {
    let alpha = unit_float(next(seed)) * core::f32::consts::TAU;
    let disc = unit_float(next(next(seed))) * DISC_RADIUS_SCALE * radius;
    let emit_angle = unit_float(next(next(next(seed))))
        * particles::EMIT_ROTATION_SPREAD
        * core::f32::consts::PI
        / 180.0;
    let emit_spin = unit_float(next(seed.wrapping_add(7))) * core::f32::consts::TAU;
    let particle = Particle {
        x: position.x + disc * alpha.sin(),
        y: position.y,
        z: position.z + disc * alpha.cos(),
        velocity_x: emit_angle.sin() * emit_spin.cos(),
        velocity_y: emit_angle.cos(),
        velocity_z: emit_angle.sin() * emit_spin.sin(),
        age: 0.0,
        life: particles::LIFE + unit_float(next(seed.wrapping_add(11))) * particles::LIFE_SPREAD,
        delay: unit_float(next(seed.wrapping_add(13))) * particles::DELAY_SPREAD,
        size: radius * SIZE_SCALE
            + unit_float(next(seed.wrapping_add(17))) * particles::SIZE_SPREAD,
        rotation: unit_float(next(seed.wrapping_add(19))) * particles::ROTATION_SPREAD,
        rotation_speed: particles::ROTATION_SPEED
            + unit_float(next(seed.wrapping_add(23))) * particles::ROTATION_SPEED_SPREAD,
    };
    let _ = state::with_state(|ui| ui.fire.spawn(particle));
}

fn next(seed: u64) -> u64 {
    seed.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

fn unit_float(seed: u64) -> f32 {
    ((seed >> 40) as u32) as f32 / 16_777_216.0
}
