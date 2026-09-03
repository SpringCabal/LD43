use crate::state;

pub fn arm(unit_id: i32) {
    if scale(unit_id).is_none() {
        return;
    }
    let _ = spring::unsynced_ctrl::set_unit_lua_draw(unit_id, true);
}

pub fn draw_unit(unit_id: i32, _draw_mode: i32) -> bool {
    let Some((horizontal, vertical)) = scale(unit_id) else {
        return false;
    };
    let _ = spring::gfx::scale(horizontal, vertical, horizontal);
    false
}

pub fn sweep() {
    let armed = state::with_state(|ui| ui.enlarger_armed).unwrap_or(true);
    if armed {
        return;
    }
    let units = spring::get_all_units().unwrap_or_default();
    if units.is_empty() {
        return;
    }
    for unit_id in units {
        arm(unit_id);
    }
    let _ = state::with_state(|ui| ui.enlarger_armed = true);
}

/// Draw scales come straight from the unit def's custom params, the same place
/// the Lua build reads them from.
fn scale(unit_id: i32) -> Option<(f32, f32)> {
    let def_id: i32 = spring::get_unit_def_id(unit_id).ok()?.into();
    let horizontal = custom_scale(def_id, "hscale");
    let vertical = custom_scale(def_id, "vscale");
    if horizontal.is_none() && vertical.is_none() {
        return None;
    }
    Some((horizontal.unwrap_or(1.0), vertical.unwrap_or(1.0)))
}

fn custom_scale(def_id: i32, key: &str) -> Option<f32> {
    spring::get_unit_def_custom_param(def_id, key)
        .ok()?
        .trim()
        .parse()
        .ok()
        .filter(|value: &f32| *value != 0.0)
}
