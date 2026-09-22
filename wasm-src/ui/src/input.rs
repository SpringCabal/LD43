use alloc::format;

use crate::hud;
use crate::state;

pub fn mouse_press(x: i32, y: i32, button: i32) -> bool {
    let _ = state::with_state(|ui| {
        ui.mouse_x = x;
        ui.mouse_y = y;
    });
    if state::with_state(|ui| ui.intro_open || ui.endgame_open).unwrap_or(false) {
        return true;
    }
    let command = match button {
        BUTTON_LEFT => MOVE,
        BUTTON_RIGHT => ATTACK_MOVE,
        _ => return false,
    };
    send_world_command(command, x, y);
    true
}

pub fn mouse_move(x: i32, y: i32) -> bool {
    let _ = state::with_state(|ui| {
        ui.mouse_x = x;
        ui.mouse_y = y;
    });
    false
}

pub fn mouse_wheel(_up: bool, _value: f32) -> bool {
    // This game is intentionally locked to the close third-person camera.
    // Returning handled prevents Spring's default camera controller from
    // changing its distance or panning in response to the wheel.
    true
}

pub fn hold_mouse() {
    if state::with_state(|ui| ui.intro_open || ui.endgame_open).unwrap_or(false) {
        return;
    }
    let Ok(mouse) = spring::input::get_mouse_state(0) else {
        return;
    };
    if mouse.offscreen {
        return;
    }
    let x = mouse.x.round() as i32;
    let y = mouse.y.round() as i32;
    if mouse.left {
        send_world_command(MOVE, x, y);
    } else if mouse.right {
        send_world_command(ATTACK, x, y);
    }
}

pub fn key_press(key: i32, ctrl: bool) -> bool {
    if ctrl && matches!(key, KEY_Q | KEY_Q_UPPER) {
        hud::quit();
        return true;
    }
    if state::with_state(|ui| ui.intro_open).unwrap_or(false) {
        if key == KEY_RETURN {
            hud::close_intro();
            return true;
        }
        return false;
    }
    if state::with_state(|ui| ui.endgame_open).unwrap_or(false) {
        if matches!(key, KEY_RETURN | KEY_R | KEY_R_UPPER) {
            hud::restart();
        }
        return true;
    }
    if key == KEY_S {
        let _ = spring::send_lua_rules_msg("stop");
        return true;
    }
    if (KEY_1..=KEY_5).contains(&key) {
        cast_spell(key - KEY_1 + 1);
        return true;
    }
    false
}

const BUTTON_LEFT: i32 = 1;
const BUTTON_RIGHT: i32 = 3;
const MOVE: &str = "movement";
const ATTACK: &str = "attack";
const ATTACK_MOVE: &str = "attmove";
const KEY_RETURN: i32 = 13;
const KEY_S: i32 = 115;
const KEY_1: i32 = 49;
const KEY_5: i32 = 53;
const KEY_Q: i32 = 113;
const KEY_Q_UPPER: i32 = 81;
const KEY_R: i32 = 114;
const KEY_R_UPPER: i32 = 82;

fn send_world_command(command: &str, x: i32, y: i32) {
    let Some(position) = ground_at(x, y) else {
        return;
    };
    let _ = spring::send_lua_rules_msg(&format!("{command}|{}|{}", position.x, position.z));
}

fn cast_spell(index: i32) {
    let Some((x, y)) = cursor() else {
        return;
    };
    let Some(position) = ground_at(x, y) else {
        return;
    };
    let _ = spring::send_lua_rules_msg(&format!(
        "spell|{index}|{}|{}|{}",
        position.x, position.y, position.z
    ));
}

fn cursor() -> Option<(i32, i32)> {
    let mouse = spring::input::get_mouse_state(0).ok()?;
    (!mouse.offscreen).then(|| (mouse.x.round() as i32, mouse.y.round() as i32))
}

fn ground_at(x: i32, y: i32) -> Option<spring::types::Float3> {
    let trace = spring::camera::trace_screen_ray(
        x as f32,
        y as f32,
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
