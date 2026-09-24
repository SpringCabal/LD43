use spring_addons::event::KeyEvent;
use springcabal::menu;
use springcabal::message::Encoder;
use springcabal_control::hold::HoldTarget;

use crate::state::Global;
use crate::{cursor, hud};

/// The move or attack the player holds a mouse button for, as last sent to
/// the rules.
pub type Held = HoldTarget<&'static str>;

/// A held command's target is sent again once it has moved this far.
pub const RESEND_DISTANCE: f32 = 8.0;

pub fn mouse_press(global: &Global, held: &Held, x: i32, y: i32, button: i32) -> bool {
    if global.hud.access().modal() {
        return true;
    }
    let command = match button {
        BUTTON_LEFT => MOVE,
        BUTTON_RIGHT => ATTACK,
        _ => return false,
    };
    if let Some(ground) = ground_at(x, y) {
        send_held(held, command, ground);
    }
    true
}

/// Drags with a steering button held are the game's, not the engine's
/// selection box.
pub fn mouse_move(button: i32) -> bool {
    matches!(button, BUTTON_LEFT | BUTTON_RIGHT)
}

/// Letting go of a steering button stops the blood mage, as in the Lua build.
pub fn mouse_release(held: &Held, button: i32) -> bool {
    clear_selection();
    if !matches!(button, BUTTON_LEFT | BUTTON_RIGHT) {
        return false;
    }
    release(held);
    true
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

/// While a button is held, keeps steering the blood mage toward the cursor:
/// the rules hear the command again whenever its target moves, and a stop once
/// the button is up or a screen takes over the input.
pub fn hold_mouse(global: &Global, held: &Held) {
    let command = if global.hud.access().modal() {
        None
    } else {
        spring::input::get_mouse_state(0)
            .ok()
            .and_then(|mouse| match (mouse.left, mouse.right) {
                (true, _) => Some(MOVE),
                (false, true) => Some(ATTACK),
                (false, false) => None,
            })
    };
    let Some(command) = command else {
        release(held);
        return;
    };
    if let Some(ground) = global.cursor.get().ground {
        send_held(held, command, ground);
    }
}

fn send_held(held: &Held, command: &'static str, ground: spring::types::Float3) {
    if held.hold(command, (ground.x, ground.z)) {
        send_rules_msg(
            &Encoder::new(command)
                .field(ground.x)
                .field(ground.z)
                .finish(),
        );
    }
}

fn release(held: &Held) {
    if held.release() {
        send_rules_msg(STOP);
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

pub fn key_press(global: &Global, event: &KeyEvent<'_>) -> bool {
    if menu::is_quit_to_menu(event) {
        menu::quit_to_menu();
        return true;
    }
    let key = event.key_code;
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
        send_rules_msg(STOP);
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
const STOP: &str = "stop";
const SPELL: &str = "spell";
const KEY_RETURN: i32 = 13;
const KEY_S: i32 = 115;
const KEY_1: i32 = 49;
const KEY_5: i32 = 53;
const KEY_R: i32 = 114;
const KEY_R_UPPER: i32 = 82;

fn send_rules_msg(message: &str) {
    if let Err(error) = spring::send_lua_rules_msg(message) {
        spring::log_warning!("ttld-wasm", "input `{message}` not sent: {error:?}");
    }
}

fn cast_spell(index: i32) {
    let Some((x, y)) = cursor() else {
        return;
    };
    let Some(position) = ground_at(x, y) else {
        return;
    };
    send_rules_msg(
        &Encoder::new(SPELL)
            .field(index)
            .field(position.x)
            .field(position.y)
            .field(position.z)
            .finish(),
    );
}

fn cursor() -> Option<(i32, i32)> {
    cursor::screen().map(|(x, y)| (x.round() as i32, y.round() as i32))
}

fn ground_at(x: i32, y: i32) -> Option<spring::types::Float3> {
    cursor::ground_at(x as f32, y as f32)
}
