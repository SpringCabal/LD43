use crate::state::{Global, HudBindings};
use ttld_shared::Command;

pub fn create(context: u64) -> Option<HudBindings> {
    let model = spring::context_create_data_model(context, "ttld").ok()?;
    if !model.success || model.data_model_handle == 0 {
        return None;
    }
    let model = model.data_model_handle;
    bind_event(model, "introPlay", super::close_intro);
    bind_event(model, "introExit", |_| springcabal::menu::quit_to_menu());
    bind_event(model, "endgameRestart", restart);
    bind_event(model, "endgameExit", |_| springcabal::menu::quit_to_menu());
    Some(HudBindings {
        health: bind_int(model, "health", 0)?,
        health_percent: bind_percent(model, "healthPercent", 100.0)?,
        health_caption: bind_string(model, "healthCaption", "100%")?,
        target_visible: bind_bool(model, "targetVisible", false)?,
        target_enemy: bind_bool(model, "targetEnemy", false)?,
        target_name: bind_string(model, "targetName", "")?,
        target_percent: bind_percent(model, "targetPercent", 100.0)?,
        target_caption: bind_string(model, "targetCaption", "")?,
        villagers_killed: bind_int(model, "villagersKilled", 0)?,
        allies_drained: bind_int(model, "alliesDrained", 0)?,
        orks_killed: bind_int(model, "orksKilled", 0)?,
        intro_visible: bind_bool(model, "introVisible", true)?,
        endgame_visible: bind_bool(model, "endgameVisible", false)?,
        endgame_victory: bind_bool(model, "endgameVictory", false)?,
        endgame_title: bind_string(model, "endgameTitle", "")?,
        endgame_line_one: bind_string(model, "endgameLineOne", "")?,
        endgame_line_two: bind_string(model, "endgameLineTwo", "")?,
    })
}

pub fn set_int(handle: u64, value: i32) {
    if handle != 0 {
        let _ = spring::data_model_set_int(handle, value);
    }
}

pub fn set_percent(handle: u64, value: f32) {
    if handle != 0 {
        let _ = spring::data_model_set_percent(handle, value.clamp(0.0, 100.0));
    }
}

pub fn set_bool(handle: u64, value: bool) {
    if handle != 0 {
        let _ = spring::data_model_set_bool(handle, value);
    }
}

pub fn set_string(handle: u64, value: &str) {
    if handle != 0 {
        let _ = spring::data_model_set_string(handle, value);
    }
}

fn bind_event(model: u64, name: &str, handler: impl FnMut(&Global) + 'static) {
    let callback = crate::register_callback(handler);
    let bound = spring::data_model_bind_event(model, name, callback, &[])
        .is_ok_and(|registration| registration.success);
    if !bound {
        crate::unregister_callback(callback);
    }
}

/// Asks the rules for a new round (`ui_endgame.lua` reloads the Lua rules,
/// which doesn't reload the Wasm rules module). The end screen stays up
/// until the new round's first status arrives; see
/// [`start_round`](super::endgame::start_round).
pub fn restart(global: &Global) {
    if global.hud.access().restart_requested {
        return;
    }
    if !springcabal::message::send_to_rules(&Command::Restart.message()) {
        return;
    }
    global.hud.access_mut().restart_requested = true;
}

fn bind_int(model: u64, name: &str, initial: i32) -> Option<u64> {
    let binding = spring::data_model_bind_int(model, name, initial).ok()?;
    binding.success.then_some(binding.variable_handle)
}

fn bind_percent(model: u64, name: &str, initial: f32) -> Option<u64> {
    let binding = spring::data_model_bind_percent(model, name, initial).ok()?;
    binding.success.then_some(binding.variable_handle)
}

fn bind_bool(model: u64, name: &str, initial: bool) -> Option<u64> {
    let binding = spring::data_model_bind_bool(model, name, initial).ok()?;
    binding.success.then_some(binding.variable_handle)
}

fn bind_string(model: u64, name: &str, initial: &str) -> Option<u64> {
    let binding = spring::data_model_bind_string(model, name, initial).ok()?;
    binding.success.then_some(binding.variable_handle)
}
