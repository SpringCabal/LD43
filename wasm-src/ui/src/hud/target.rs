use alloc::format;

use crate::state;

use super::model;

pub fn track_target() {
    let Some((x, y)) = mouse_position() else {
        clear();
        return;
    };
    let Some(unit_id) = unit_under_cursor(x, y) else {
        clear();
        return;
    };
    let Ok(health) = spring::units_info::get_unit_health(unit_id) else {
        clear();
        return;
    };
    let current = health.health.max(0.0);
    let max = health.max_health.max(1.0);
    let ratio = (100.0 * current / max) as i32;
    let name = human_name(unit_id);
    let enemy = is_enemy(unit_id);
    let caption = if current <= 0.0 {
        alloc::string::String::from("DEAD")
    } else {
        format!("{ratio}%")
    };
    let _ = state::with_state(|ui| {
        let Some(hud) = ui.hud else {
            return;
        };
        if ui.target_unit != unit_id {
            ui.target_unit = unit_id;
            model::set_string(hud.target_name, &name);
        }
        model::set_bool(hud.target_enemy, enemy);
        if ui.target_ratio != ratio {
            ui.target_ratio = ratio;
            model::set_percent(hud.target_percent, ratio as f32);
            model::set_string(hud.target_caption, &caption);
        }
        model::set_bool(hud.target_visible, true);
    });
}

const HIT_UNIT: i32 = 1;

fn clear() {
    let _ = state::with_state(|ui| {
        if ui.target_unit == -1 {
            return;
        }
        ui.target_unit = -1;
        ui.target_ratio = -2;
        if let Some(hud) = ui.hud {
            model::set_bool(hud.target_visible, false);
            model::set_bool(hud.target_enemy, false);
        }
    });
}

fn mouse_position() -> Option<(i32, i32)> {
    let mouse = spring::input::get_mouse_state(0).ok()?;
    (!mouse.offscreen).then(|| (mouse.x.round() as i32, mouse.y.round() as i32))
}

fn unit_under_cursor(x: i32, y: i32) -> Option<i32> {
    let trace = spring::camera::trace_screen_ray(
        x as f32,
        y as f32,
        spring::camera::TraceScreenRayOptions {
            only_coords: false,
            use_minimap: false,
            include_sky: false,
            ignore_water: false,
            height_offset: 0.0,
        },
    )
    .ok()?;
    if trace.hit_type == HIT_UNIT {
        return Some(trace.hit_id);
    }

    None
}

fn is_enemy(unit_id: i32) -> bool {
    let Some(team) = spring::units_info::get_unit_team(unit_id).ok() else {
        return false;
    };
    spring::player::get_local_team_id(0)
        .map(|local| team != local)
        .unwrap_or(false)
}

fn human_name(unit_id: i32) -> alloc::string::String {
    let Ok(unit_def_id) = spring::units_info::get_unit_def_id(unit_id) else {
        return alloc::string::String::new();
    };
    spring::unit_defs::get_unit_def_human_name_bytes(unit_def_id)
        .ok()
        .and_then(|bytes| alloc::string::String::from_utf8(bytes).ok())
        .unwrap_or_default()
}
