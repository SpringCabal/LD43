use spring::mouse;
use spring_addons::event::KeyEvent;
use springcabal::menu;
use springcabal_control::hold::HoldTarget;
use ttld_shared::Command;

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
        mouse::LEFT => MOVE,
        mouse::RIGHT => ATTACK,
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
    matches!(button, mouse::LEFT | mouse::RIGHT)
}

/// Letting go of a steering button stops the blood mage, as in the Lua build.
pub fn mouse_release(held: &Held, button: i32) -> bool {
    clear_selection();
    if !matches!(button, mouse::LEFT | mouse::RIGHT) {
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
        let (x, z) = (ground.x, ground.z);
        let command = if command == ATTACK {
            Command::Attack {
                x,
                z,
                move_to: false,
            }
        } else {
            Command::Move { x, z }
        };
        springcabal::message::send_to_rules(&command.message());
    }
}

fn release(held: &Held) {
    if held.release() {
        springcabal::message::send_to_rules(&Command::Stop.message());
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
    let (intro_open, endgame_open) = {
        let hud = global.hud.access();
        (hud.intro_open, hud.endgame_open)
    };
    if intro_open {
        if event.matches("enter") {
            hud::close_intro(global);
            return true;
        }
        return false;
    }
    if endgame_open {
        if event.matches("enter") || event.matches("r") {
            hud::restart(global);
        }
        return true;
    }
    if event.matches("s") {
        springcabal::message::send_to_rules(&Command::Stop.message());
        return true;
    }
    if let Some(index) = SPELL_KEYS.iter().position(|key| event.matches(key)) {
        cast_spell(index as i32 + 1);
        return true;
    }
    false
}

/// What a held mouse button does.
const MOVE: &str = "move";
const ATTACK: &str = "attack";
/// The keys of spells 1 to 5.
const SPELL_KEYS: [&str; 5] = ["1", "2", "3", "4", "5"];

fn cast_spell(index: i32) {
    let Some((x, y)) = cursor() else {
        return;
    };
    let Some(position) = ground_at(x, y) else {
        return;
    };
    springcabal::message::send_to_rules(
        &Command::Spell {
            index,
            x: position.x,
            y: position.y,
            z: position.z,
        }
        .message(),
    );
}

fn cursor() -> Option<(i32, i32)> {
    cursor::screen().map(|(x, y)| (x.round() as i32, y.round() as i32))
}

fn ground_at(x: i32, y: i32) -> Option<spring::types::Float3> {
    cursor::ground_at(x as f32, y as f32)
}
