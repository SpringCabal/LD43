use alloc::format;

use crate::state;

use super::model;

pub fn close_intro() {
    let _ = state::with_state(|ui| {
        if !ui.intro_open {
            return;
        }
        ui.intro_open = false;
        if let Some(hud) = ui.hud {
            model::set_bool(hud.intro_visible, false);
        }
    });
    let _ = spring::send_commands("pause 0", "");
}

pub fn open_endgame(victory: bool) {
    let (villagers, drained, orks) =
        state::with_state(|ui| (ui.villagers_killed, ui.allies_drained, ui.orks_killed))
            .unwrap_or((0, 0, 0));
    let (line_one, line_two) = if victory {
        (
            format!("{villagers} townsfolk lost their lives."),
            format!("{drained} of the fallen were found mysteriously drained of blood."),
        )
    } else {
        (
            format!("{orks} orks were slain before you succumbed."),
            "There is no hope without your aid.".into(),
        )
    };
    let title = if victory {
        "The Town is Saved!"
    } else {
        "Defeat!"
    };
    let _ = state::with_state(|ui| {
        if ui.endgame_open {
            return;
        }
        ui.endgame_open = true;
        let Some(hud) = ui.hud else {
            return;
        };
        model::set_bool(hud.endgame_visible, true);
        model::set_bool(hud.endgame_victory, victory);
        model::set_string(hud.endgame_title, title);
        model::set_string(hud.endgame_line_one, &line_one);
        model::set_string(hud.endgame_line_two, &line_two);
    });
    let _ = spring::send_commands("endgraph 0", "");
}

pub fn evaluate() {
    let outcome = state::with_state(|ui| {
        if ui.intro_open || ui.endgame_open {
            return None;
        }
        if ui.game_end == "victory" {
            return Some(true);
        }
        (!ui.player_alive).then_some(false)
    })
    .flatten();
    if let Some(victory) = outcome {
        open_endgame(victory);
    }
}
