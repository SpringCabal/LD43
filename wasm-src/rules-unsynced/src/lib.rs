//! Client-side unit rendering hooks from the original LuaRules unsynced gadget.

fn unit_scale(unit_id: i32) -> Option<(f32, f32)> {
    let horizontal = spring::get_unit_rules_param_f32(unit_id, "scale_horizontal").ok();
    let vertical = spring::get_unit_rules_param_f32(unit_id, "scale_vertical").ok();
    if horizontal.is_none() && vertical.is_none() {
        return None;
    }
    Some((horizontal.unwrap_or(1.0), vertical.unwrap_or(1.0)))
}

fn draw_unit(unit_id: i32, _draw_mode: i32) -> bool {
    let Some((horizontal, vertical)) = unit_scale(unit_id) else {
        return false;
    };
    let _ = spring::gfx::scale(horizontal, vertical, horizontal);
    false
}

spring::export_environment_mask!(spring::rules_unsynced::ENVIRONMENT_MASK);
spring::export_draw_unit!(draw_unit);
