use crate::game;
use crate::state::GameStatus;

use super::PointLight;

/// The glow at the tip of the blood mage's staff, brighter while casting.
pub fn light(status: &GameStatus, boost: &mut Boost, seconds: f32) -> Option<PointLight> {
    if status.player_unit < 0 {
        return None;
    }
    let position = staff_position(status.player_unit)?;
    let boost = boost.update(status);
    let flicker = (seconds * FLICKER_RATE).sin() / core::f32::consts::PI * STATIC_FLICKER_RATIO;
    let multiplier = COLOR_MULTIPLIER * boost * (1.0 + flicker);
    Some(PointLight {
        position: [position.x, position.y + HEIGHT, position.z],
        color: [RED * multiplier, GREEN * multiplier, BLUE * multiplier],
        radius: RADIUS,
    })
}

/// The staff light multiplier: it rises during a cast and decays back to 1.
pub struct Boost(f32);

impl Default for Boost {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Boost {
    fn update(&mut self, status: &GameStatus) -> f32 {
        if status.cast_animation_time > 0.0 {
            let elapsed = (game::status_frame() - status.cast_start_frame) as f32;
            let animation = status.cast_animation_time;
            if elapsed <= animation * 0.4 {
                self.0 = (self.0 + BOOST_RISE_FAST).min(BOOST_MAXIMUM);
            } else if elapsed <= animation * 0.55 {
                self.0 += BOOST_RISE_SLOW;
            }
        }
        if self.0 > 1.0 {
            self.0 = (self.0 - BOOST_DECAY).max(1.0);
        }
        self.0
    }
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

fn staff_position(unit_id: i32) -> Option<spring::types::Float3> {
    // Match ui_unit_light.lua: GetUnitViewPosition is the model's rendered
    // origin, and GetUnitPiecePosition supplies the absolute piece offset.
    // Do not fall back to the unit centre: that creates a light which Lua
    // would not draw and makes the staff pool visibly drift from the model.
    let unit = spring::unit_rendering::get_unit_view_position(unit_id, false).ok()?;
    let piece = spring::typed::unit_piece_position_by_name(unit_id, STAFF_PIECE)?;
    Some(spring::types::Float3 {
        x: unit.x + piece.x,
        y: unit.y + piece.y,
        z: unit.z + piece.z,
    })
}
