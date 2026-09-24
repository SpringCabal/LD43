use alloc::format;

use crate::state::{GameEnd, Global};

use super::model;

/// Closes the intro screen and starts the game.
pub fn close_intro(global: &Global) {
    let bindings = {
        let mut hud = global.hud.access_mut();
        if !hud.intro_open {
            return;
        }
        hud.intro_open = false;
        hud.bindings
    };
    if let Some(bindings) = bindings {
        model::set_bool(bindings.intro_visible, false);
    }
    let _ = spring::send_commands("pause 0", "");
}

/// A new round has started (its first status has arrived): closes the end
/// screen and the target panel. The status already holds the new round's
/// counters.
pub fn start_round(global: &Global) {
    let bindings = {
        let mut hud = global.hud.access_mut();
        hud.endgame_open = false;
        hud.restart_requested = false;
        hud.target = None;
        hud.bindings
    };
    if let Some(bindings) = bindings {
        model::set_bool(bindings.endgame_visible, false);
        model::set_bool(bindings.target_visible, false);
        model::set_bool(bindings.target_enemy, false);
    }
}

/// Opens the end screen once the game is won or the blood mage is dead.
pub fn evaluate(global: &Global) {
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
        open(global, victory);
    }
}

fn open(global: &Global, victory: bool) {
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
        model::set_bool(bindings.endgame_visible, true);
        model::set_bool(bindings.endgame_victory, victory);
        model::set_string(bindings.endgame_title, title);
        model::set_string(bindings.endgame_line_one, &line_one);
        model::set_string(bindings.endgame_line_two, &line_two);
    }
    let _ = spring::send_commands("endgraph 0", "");
}
