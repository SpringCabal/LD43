//! What the mouse cursor points at.

use spring::camera::{TraceScreenRayOptions, trace_screen_ray};
use spring::types::Float3;

/// `hit_type` of a screen-ray trace that hit a unit.
const HIT_UNIT: i32 = 1;

/// The cursor's screen position, or `None` while it is off the window.
pub fn screen() -> Option<(f32, f32)> {
    let mouse = spring::input::get_mouse_state(0).ok()?;
    (!mouse.offscreen).then_some((mouse.x, mouse.y))
}

/// The ground point under screen position `(x, y)`.
pub fn ground_at(x: f32, y: f32) -> Option<Float3> {
    let trace = trace_screen_ray(x, y, options(true)).ok()?;
    Some(trace.hit_pos)
}

/// The unit under screen position `(x, y)`.
pub fn unit_at(x: f32, y: f32) -> Option<i32> {
    let trace = trace_screen_ray(x, y, options(false)).ok()?;
    (trace.hit_type == HIT_UNIT).then_some(trace.hit_id)
}

fn options(only_coords: bool) -> TraceScreenRayOptions {
    TraceScreenRayOptions {
        only_coords,
        use_minimap: false,
        include_sky: false,
        ignore_water: false,
        height_offset: 0.0,
    }
}
