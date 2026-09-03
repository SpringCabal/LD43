use crate::state;

use super::params;

pub fn update(seconds: f32) {
    let unit_id = state::with_state(|ui| ui.player_unit).unwrap_or(-1);
    if unit_id < 0 {
        release();
        return;
    }
    let Some(position) = staff_position(unit_id) else {
        return;
    };
    let boost = update_boost();
    let flicker = (seconds * FLICKER_RATE).sin() / core::f32::consts::PI * STATIC_FLICKER_RATIO;
    let multiplier = COLOR_MULTIPLIER * boost * (1.0 + flicker);
    let light = params::point(
        [position.x, position.y + HEIGHT, position.z],
        [RED * multiplier, GREEN * multiplier, BLUE * multiplier],
        RADIUS,
    );
    let handle = state::with_state(|ui| ui.staff_light).unwrap_or(0);
    if handle == 0 {
        if let Ok(handle) = spring::lights::add_model_light(&light) {
            let _ = state::with_state(|ui| ui.staff_light = handle);
        }
        return;
    }
    let _ = spring::lights::update_model_light(handle, &light);
}

const STATIC_FLICKER_RATIO: f32 = 0.5;
const FLICKER_RATE: f32 = 3.0;
const HEIGHT: f32 = 15.0;
const RADIUS: f32 = 1000.0;
const RED: f32 = 4.0;
const GREEN: f32 = 1.0;
const BLUE: f32 = 1.0;
const COLOR_MULTIPLIER: f32 = 2.0;
const BOOST_MAXIMUM: f32 = 3.0;
const BOOST_RISE_FAST: f32 = 0.45;
const BOOST_RISE_SLOW: f32 = 0.18;
const BOOST_DECAY: f32 = 0.18;
const STAFF_PIECE: &str = "Hand_Right";

fn release() {
    let handle = state::with_state(|ui| ui.staff_light).unwrap_or(0);
    if handle != 0 {
        let dark = params::point([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.0);
        let _ = spring::lights::update_model_light(handle, &dark);
    }
}

fn staff_position(unit_id: i32) -> Option<spring::types::Float3> {
    let unit = spring::units_info::get_unit_position(
        unit_id,
        spring::units_info::GetUnitPositionOptions {
            mid_pos: false,
            aim_pos: false,
        },
    )
    .ok()?;
    let piece = spring::typed::unit_piece_position_by_name(unit_id, STAFF_PIECE)?;
    Some(spring::types::Float3 {
        x: unit.x + piece.x,
        y: unit.y + piece.y,
        z: unit.z + piece.z,
    })
}

fn update_boost() -> f32 {
    state::with_state(|ui| {
        if ui.cast_animation_time > 0.0 {
            let elapsed = (ui.game_frame - ui.cast_start_frame) as f32;
            let animation = ui.cast_animation_time;
            if elapsed <= animation * 0.4 {
                ui.staff_boost = (ui.staff_boost + BOOST_RISE_FAST).min(BOOST_MAXIMUM);
            } else if elapsed <= animation * 0.55 {
                ui.staff_boost += BOOST_RISE_SLOW;
            }
        }
        if ui.staff_boost > 1.0 {
            ui.staff_boost = (ui.staff_boost - BOOST_DECAY).max(1.0);
        }
        ui.staff_boost
    })
    .unwrap_or(1.0)
}
