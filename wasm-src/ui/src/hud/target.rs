use alloc::format;
use alloc::string::String;

use crate::state::Global;

use super::model;

/// Shows the name and health of the unit under the cursor.
pub fn track(global: &Global) {
    let Some(unit_id) = global.cursor.get().unit else {
        clear(global);
        return;
    };
    let Ok(health) = spring::units_info::get_unit_health(unit_id) else {
        clear(global);
        return;
    };
    let current = health.health.max(0.0);
    let max = health.max_health.max(1.0);
    let ratio = (100.0 * current / max) as i32;
    let enemy = is_enemy(unit_id);
    let (bindings, shown) = {
        let mut hud = global.hud.access_mut();
        let Some(bindings) = hud.bindings else {
            return;
        };
        (bindings, hud.target.replace((unit_id, ratio)))
    };
    let (shown_unit, shown_ratio) = shown.unwrap_or((-1, -2));
    if shown_unit != unit_id {
        model::set_string(bindings.target_name, &human_name(unit_id));
    }
    model::set_bool(bindings.target_enemy, enemy);
    if shown_ratio != ratio {
        model::set_percent(bindings.target_percent, ratio as f32);
        let caption = if current <= 0.0 {
            String::from("DEAD")
        } else {
            format!("{ratio}%")
        };
        model::set_string(bindings.target_caption, &caption);
    }
    model::set_bool(bindings.target_visible, true);
}

fn clear(global: &Global) {
    let bindings = {
        let mut hud = global.hud.access_mut();
        if hud.target.take().is_none() {
            return;
        }
        hud.bindings
    };
    if let Some(bindings) = bindings {
        model::set_bool(bindings.target_visible, false);
        model::set_bool(bindings.target_enemy, false);
    }
}

fn is_enemy(unit_id: i32) -> bool {
    let Some(team) = spring::units_info::get_unit_team(unit_id).ok() else {
        return false;
    };
    spring::player::get_local_team_id(0)
        .map(|local| team != local)
        .unwrap_or(false)
}

fn human_name(unit_id: i32) -> String {
    let Ok(unit_def_id) = spring::units_info::get_unit_def_id(unit_id) else {
        return String::new();
    };
    spring::unit_defs::get_unit_def_human_name_bytes(unit_def_id)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default()
}
