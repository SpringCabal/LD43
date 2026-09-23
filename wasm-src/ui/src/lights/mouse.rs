use crate::{cursor, state};

use super::params;

pub fn update() {
    let Some(position) = cursor::screen().and_then(|(x, y)| cursor::ground_at(x, y)) else {
        return;
    };
    let light = params::point(
        [position.x, position.y + HEIGHT, position.z],
        [RED, GREEN, BLUE],
        RADIUS,
    );
    let deferred_light = state::DeferredPointLight::from_params(&light);
    let _ = state::with_state(|ui| ui.mouse_deferred_light = Some(deferred_light));
}

const HEIGHT: f32 = 100.0;
const RADIUS: f32 = 5000.0;
const RED: f32 = 0.9;
const GREEN: f32 = 0.9;
const BLUE: f32 = 0.9;
