use alloc::format;

use crate::state::{GameEnd, Global};

use super::model;

pub fn close_intro(spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
    let bindings = {
        let mut hud = global.hud.access_mut();
        if !hud.intro_open {
            return;
        }
        hud.intro_open = false;
        hud.bindings
    };
    if let Some(bindings) = bindings {
        model::set_bool(spring, bindings.intro_visible, false);
    }
    let _ = spring.messages().send_commands("pause 0", "");
}

pub fn start_round(spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
    let bindings = {
        let mut hud = global.hud.access_mut();
        hud.endgame_open = false;
        hud.restart_requested = false;
        hud.target = None;
        hud.bindings
    };
    if let Some(bindings) = bindings {
        model::set_bool(spring, bindings.endgame_visible, false);
        model::set_bool(spring, bindings.target_visible, false);
        model::set_bool(spring, bindings.target_enemy, false);
    }
}

pub fn evaluate(spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
    let outcome = {
        let status = global.status.access();
        if status.game_end == GameEnd::Victory {
            Some(true)
        } else {
            (!status.player_alive).then_some(false)
        }
    };
    if let Some(victory) = outcome
        && !global.hud.access().modal()
    {
        open(spring, global, victory);
    }
}

fn open(spring: &spring::Spring<'_, spring::Ui>, global: &Global, victory: bool) {
    let (villagers, drained, orks) = {
        let status = global.status.access();
        (
            status.villagers_killed,
            status.allies_drained,
            status.orks_killed,
        )
    };
    let (title, line_one, line_two) = if victory {
        (
            "The Town is Saved!",
            format!("{villagers} townsfolk lost their lives."),
            format!("{drained} of the fallen were found mysteriously drained of blood."),
        )
    } else {
        (
            "Defeat!",
            format!("{orks} orks were slain before you succumbed."),
            "There is no hope without your aid.".into(),
        )
    };
    let bindings = {
        let mut hud = global.hud.access_mut();
        hud.endgame_open = true;
        hud.bindings
    };
    if let Some(bindings) = bindings {
        model::set_bool(spring, bindings.endgame_visible, true);
        model::set_bool(spring, bindings.endgame_victory, victory);
        model::set_string(spring, bindings.endgame_title, title);
        model::set_string(spring, bindings.endgame_line_one, &line_one);
        model::set_string(spring, bindings.endgame_line_two, &line_two);
    }
    let _ = spring.messages().send_commands("endgraph 0", "");
}
