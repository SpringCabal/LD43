use alloc::format;
use core::cell::Cell;

use crate::state::Global;
use crate::{cursor, game, hud};

pub fn mouse_press(global: &Global, x: i32, y: i32, button: i32) -> bool {
    if global.hud.access().modal() {
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

/// Drags with a steering button held are the game's, not the engine's
/// selection box.
pub fn mouse_move(button: i32) -> bool {
    matches!(button, BUTTON_LEFT | BUTTON_RIGHT)
}

pub fn mouse_release(button: i32) -> bool {
    clear_selection();
    matches!(button, BUTTON_LEFT | BUTTON_RIGHT)
}

/// Like the Lua build's RPG controller, which empties the selection in
/// SelectionChanged: units are hover targets, never an RTS selection. Checked
/// every update, which also covers selecting through the minimap or keys.
pub fn clear_selection() {
    if spring::selection::get_selected_units_count(0).unwrap_or(0) > 0 {
        let _ = spring::selection::select_unit_array(&[], false);
    }
}

pub fn mouse_wheel(_up: bool, _value: f32) -> bool {
    // This game is intentionally locked to the close third-person camera.
    // Returning handled prevents Spring's default camera controller from
    // changing its distance or panning in response to the wheel.
    true
}

/// While a button is held, keeps steering the blood mage toward the cursor,
/// once per game frame (`last_frame` is the frame last steered on).
pub fn hold_mouse(global: &Global, last_frame: &Cell<i32>) {
    if global.hud.access().modal() {
        return;
    }
    let Ok(mouse) = spring::input::get_mouse_state(0) else {
        return;
    };
    let command = if mouse.left {
        MOVE
    } else if mouse.right {
        ATTACK
    } else {
        return;
    };
    let Some(ground) = global.cursor.get().ground else {
        return;
    };
    let frame = game::frame();
    if last_frame.replace(frame) != frame {
        send_ground_command(command, ground);
    }
}

/// Samples what the cursor points at, for every widget this update.
pub fn sample_cursor(global: &Global) {
    let screen = cursor::screen();
    global.cursor.set(crate::state::Cursor {
        screen,
        ground: screen.and_then(|(x, y)| cursor::ground_at(x, y)),
        unit: screen.and_then(|(x, y)| cursor::unit_at(x, y)),
    });
}

pub fn key_press(global: &Global, key: i32, ctrl: bool) -> bool {
    if ctrl && matches!(key, KEY_Q | KEY_Q_UPPER) {
        hud::exit_to_menu();
        return true;
    }
    let (intro_open, endgame_open) = {
        let hud = global.hud.access();
        (hud.intro_open, hud.endgame_open)
    };
    if intro_open {
        if key == KEY_RETURN {
            hud::close_intro(global);
            return true;
        }
        return false;
    }
    if endgame_open {
        if matches!(key, KEY_RETURN | KEY_R | KEY_R_UPPER) {
            hud::restart(global);
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
    if let Some(position) = ground_at(x, y) {
        send_ground_command(command, position);
    }
}

fn send_ground_command(command: &str, position: spring::types::Float3) {
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
    cursor::screen().map(|(x, y)| (x.round() as i32, y.round() as i32))
}

fn ground_at(x: i32, y: i32) -> Option<spring::types::Float3> {
    cursor::ground_at(x as f32, y as f32)
}
