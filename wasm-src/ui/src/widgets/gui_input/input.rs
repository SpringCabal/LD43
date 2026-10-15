use spring::mouse;
use spring_addons::event::KeyEvent;
use springcabal::menu;
use springcabal_control::hold::HoldTarget;
use ttld_shared::Command;

use crate::cursor;
use crate::state::Global;
use crate::widgets::gui_hud::hud;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HeldCommand {
    Move,
    Attack,
}

pub type Held = HoldTarget<HeldCommand>;

pub const RESEND_DISTANCE: f32 = 8.0;

pub fn mouse_press(global: &Global, held: &Held, x: i32, y: i32, button: i32) -> bool {
    if global.hud.access().modal() {
        return true;
    }
    let command = match button {
        mouse::LEFT => HeldCommand::Move,
        mouse::RIGHT => HeldCommand::Attack,
        _ => return false,
    };
    if let Some(ground) = ground_at(x, y) {
        send_held(held, command, ground);
    }
    true
}

pub fn mouse_move(button: i32) -> bool {
    matches!(button, mouse::LEFT | mouse::RIGHT)
}

pub fn mouse_release(held: &Held, button: i32) -> bool {
    clear_selection();
    if !matches!(button, mouse::LEFT | mouse::RIGHT) {
        return false;
    }
    release(held);
    true
}

pub fn clear_selection() {
    if spring::selection::get_selected_units_count(0).unwrap_or(0) > 0 {
        let _ = spring::selection::select_unit_array(&[], false);
    }
}

pub fn mouse_wheel(_up: bool, _value: f32) -> bool {
    true
}

pub fn hold_mouse(global: &Global, held: &Held) {
    let command = if global.hud.access().modal() {
        None
    } else {
        spring::input::get_mouse_state(0)
            .ok()
            .and_then(|mouse| match (mouse.left, mouse.right) {
                (true, _) => Some(HeldCommand::Move),
                (false, true) => Some(HeldCommand::Attack),
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

fn send_held(held: &Held, command: HeldCommand, ground: spring::types::Float3) {
    if held.hold(command, (ground.x, ground.z)) {
        let (x, z) = (ground.x, ground.z);
        let command = match command {
            HeldCommand::Move => Command::Move { x, z },
            HeldCommand::Attack => Command::Attack {
                x,
                z,
                move_to: false,
            },
        };
        springcabal::message::send_to_rules(&command.message());
    }
}

fn release(held: &Held) {
    if held.release() {
        springcabal::message::send_to_rules(&Command::Stop.message());
    }
}

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
