use crate::state;

use super::params;

pub fn update() {
    let Some(position) = cursor_ground() else {
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

fn cursor_ground() -> Option<spring::types::Float3> {
    let mouse = spring::input::get_mouse_state(0).ok()?;
    if mouse.offscreen {
        return None;
    }
    let trace = spring::camera::trace_screen_ray(
        mouse.x,
        mouse.y,
        spring::camera::TraceScreenRayOptions {
            only_coords: true,
            use_minimap: false,
            include_sky: false,
            ignore_water: false,
            height_offset: 0.0,
        },
    )
    .ok()?;
    Some(trace.hit_pos)
}
