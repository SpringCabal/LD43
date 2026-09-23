use alloc::vec;
use core::cell::Cell;

/// Sets the sun lighting of the Lua build's gfx_day_night.lua, once.
pub fn apply(applied: &Cell<bool>) {
    if applied.get() {
        return;
    }
    let params = spring::unsynced_ctrl::SunLightingParams {
        ground_ambient_color: Some(vec![GROUND_AMBIENT, GROUND_AMBIENT, GROUND_AMBIENT, 1.0]),
        ground_diffuse_color: Some(vec![GROUND_DIFFUSE, GROUND_DIFFUSE, GROUND_DIFFUSE, 1.0]),
        ground_specular_color: Some(vec![1.0, 1.0, 1.0, 1.0]),
        model_ambient_color: Some(vec![MODEL_AMBIENT, MODEL_AMBIENT, MODEL_AMBIENT, 1.0]),
        model_diffuse_color: Some(vec![MODEL_DIFFUSE, MODEL_DIFFUSE, MODEL_DIFFUSE, 1.0]),
        model_specular_color: Some(vec![1.0, 1.0, 1.0, 1.0]),
        specular_exponent: None,
        ground_shadow_density: Some(GROUND_SHADOW_DENSITY),
        model_shadow_density: Some(MODEL_SHADOW_DENSITY),
    };
    if spring::unsynced_ctrl::set_sun_lighting(params).is_ok() {
        applied.set(true);
        let _ = spring::send_commands("shadows 1", "");
    }
}

const GROUND_AMBIENT: f32 = 0.3;
const GROUND_DIFFUSE: f32 = 0.1;
const MODEL_AMBIENT: f32 = 0.5;
const MODEL_DIFFUSE: f32 = 0.4;
const GROUND_SHADOW_DENSITY: f32 = 0.940_000_63;
const MODEL_SHADOW_DENSITY: f32 = 0.970_000_24;
