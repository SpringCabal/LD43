use crate::state::{Global, HudBindings};
use ttld_shared::Command;

pub fn create(
    spring: &spring::Spring<'_, spring::Ui>,
    context: u64,
    intro_open: bool,
) -> Option<HudBindings> {
    let model = spring
        .rml_ui()
        .context_create_data_model(context, "ttld")
        .ok()?;
    if !model.success || model.data_model_handle == 0 {
        return None;
    }
    let model = model.data_model_handle;
    bind_event(spring, model, "introPlay", super::close_intro)?;
    bind_event(spring, model, "introExit", |spring, _| {
        springcabal::menu::quit_to_menu(spring)
    })?;
    bind_event(spring, model, "endgameRestart", restart)?;
    bind_event(spring, model, "endgameExit", |spring, _| {
        springcabal::menu::quit_to_menu(spring)
    })?;
    Some(HudBindings {
        health: bind_int(spring, model, "health", 0)?,
        health_percent: bind_percent(spring, model, "healthPercent", 100.0)?,
        health_caption: bind_string(spring, model, "healthCaption", "100%")?,
        target_visible: bind_bool(spring, model, "targetVisible", false)?,
        target_enemy: bind_bool(spring, model, "targetEnemy", false)?,
        target_name: bind_string(spring, model, "targetName", "")?,
        target_percent: bind_percent(spring, model, "targetPercent", 100.0)?,
        target_caption: bind_string(spring, model, "targetCaption", "")?,
        villagers_killed: bind_int(spring, model, "villagersKilled", 0)?,
        allies_drained: bind_int(spring, model, "alliesDrained", 0)?,
        orks_killed: bind_int(spring, model, "orksKilled", 0)?,
        intro_visible: bind_bool(spring, model, "introVisible", intro_open)?,
        endgame_visible: bind_bool(spring, model, "endgameVisible", false)?,
        endgame_victory: bind_bool(spring, model, "endgameVictory", false)?,
        endgame_title: bind_string(spring, model, "endgameTitle", "")?,
        endgame_line_one: bind_string(spring, model, "endgameLineOne", "")?,
        endgame_line_two: bind_string(spring, model, "endgameLineTwo", "")?,
    })
}

pub fn set_int(spring: &spring::Spring<'_, spring::Ui>, handle: u64, value: i32) {
    if handle != 0 {
        let _ = spring.rml_ui().data_model_set_int(handle, value);
    }
}

pub fn set_percent(spring: &spring::Spring<'_, spring::Ui>, handle: u64, value: f32) {
    if handle != 0 {
        let _ = spring
            .rml_ui()
            .data_model_set_percent(handle, value.clamp(0.0, 100.0));
    }
}

pub fn set_bool(spring: &spring::Spring<'_, spring::Ui>, handle: u64, value: bool) {
    if handle != 0 {
        let _ = spring.rml_ui().data_model_set_bool(handle, value);
    }
}

pub fn set_string(spring: &spring::Spring<'_, spring::Ui>, handle: u64, value: &str) {
    if handle != 0 {
        let _ = spring.rml_ui().data_model_set_string(handle, value);
    }
}

fn bind_event(
    spring: &spring::Spring<'_, spring::Ui>,
    model: u64,
    name: &str,
    handler: impl Fn(&spring::Spring<'_, spring::Ui>, &Global) + spring::RetainedCapture + 'static,
) -> Option<()> {
    spring
        .rml_ui()
        .data_model_bind_event(model, name, &[], move |spring, _| {
            crate::with_global(|global| handler(spring, global));
            Ok(())
        })
        .is_ok_and(|registration| registration.success)
        .then_some(())
}

pub fn restart(spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
    if global.hud.access().restart_requested {
        return;
    }
    if !springcabal::message::send_to_rules(spring, &Command::Restart.message()) {
        return;
    }
    global.hud.access_mut().restart_requested = true;
}

fn bind_int(
    spring: &spring::Spring<'_, spring::Ui>,
    model: u64,
    name: &str,
    initial: i32,
) -> Option<u64> {
    let binding = spring
        .rml_ui()
        .data_model_bind_int(model, name, initial)
        .ok()?;
    (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
}

fn bind_percent(
    spring: &spring::Spring<'_, spring::Ui>,
    model: u64,
    name: &str,
    initial: f32,
) -> Option<u64> {
    let binding = spring
        .rml_ui()
        .data_model_bind_percent(model, name, initial)
        .ok()?;
    (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
}

fn bind_bool(
    spring: &spring::Spring<'_, spring::Ui>,
    model: u64,
    name: &str,
    initial: bool,
) -> Option<u64> {
    let binding = spring
        .rml_ui()
        .data_model_bind_bool(model, name, initial)
        .ok()?;
    (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
}

fn bind_string(
    spring: &spring::Spring<'_, spring::Ui>,
    model: u64,
    name: &str,
    initial: &str,
) -> Option<u64> {
    let binding = spring
        .rml_ui()
        .data_model_bind_string(model, name, initial)
        .ok()?;
    (binding.success && binding.variable_handle != 0).then_some(binding.variable_handle)
}
