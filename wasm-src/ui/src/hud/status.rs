use crate::state::{self, GameEnd};

use super::endgame;
use super::model;

pub fn update(message: &str) {
    let mut health = 0;
    let mut max_health = 1000;
    for field in message.split('|').skip(1) {
        let Some((key, value)) = field.split_once('=') else {
            continue;
        };
        match key {
            "health" => health = value.parse().unwrap_or(0),
            "maxHealth" => max_health = value.parse().unwrap_or(1000),
            "villagersKilled" => set_counter(value, Counter::VillagersKilled),
            "alliesDrained" => set_counter(value, Counter::AlliesDrained),
            "orksKilled" => set_counter(value, Counter::OrksKilled),
            "gameEnd" => set_game_end(value),
            "playerAlive" => set_player_alive(value != "0"),
            "playerUnit" => {
                let unit_id = value.parse().unwrap_or(-1);
                let _ = state::with_state(|ui| ui.player_unit = unit_id);
            }
            "frame" => {
                let frame = value.parse().unwrap_or(0);
                let _ = state::with_state(|ui| ui.game_frame = frame);
            }
            "castStart" => {
                let frame = value.parse().unwrap_or(0);
                let _ = state::with_state(|ui| ui.cast_start_frame = frame);
            }
            "castAnimTime" => {
                let time = value.parse().unwrap_or(0.0);
                let _ = state::with_state(|ui| ui.cast_animation_time = time);
            }
            "casting" => {
                let _ = state::with_state(|ui| ui.casting = value != "0");
            }
            _ => {}
        }
    }
    apply_health(health, max_health);
    endgame::evaluate();
}

enum Counter {
    VillagersKilled,
    AlliesDrained,
    OrksKilled,
}

fn set_counter(value: &str, counter: Counter) {
    let Ok(value) = value.parse::<i32>() else {
        return;
    };
    let _ = state::with_state(|ui| {
        let Some(hud) = ui.hud else {
            return;
        };
        match counter {
            Counter::VillagersKilled => {
                ui.villagers_killed = value;
                model::set_int(hud.villagers_killed, value);
            }
            Counter::AlliesDrained => {
                ui.allies_drained = value;
                model::set_int(hud.allies_drained, value);
            }
            Counter::OrksKilled => {
                ui.orks_killed = value;
                model::set_int(hud.orks_killed, value);
            }
        }
    });
}

fn set_game_end(value: &str) {
    let _ = state::with_state(|ui| ui.game_end = GameEnd::parse(value));
}

fn set_player_alive(alive: bool) {
    let _ = state::with_state(|ui| ui.player_alive = alive);
}

fn apply_health(health: i32, max_health: i32) {
    let max_health = max_health.max(1);
    let ratio = (health as f32 / max_health as f32).clamp(0.0, 1.0);
    let caption = if health <= 0 {
        alloc::string::String::from("DEAD")
    } else {
        alloc::format!("{}%", (ratio * 100.0) as i32)
    };
    let _ = state::with_state(|ui| {
        let Some(hud) = ui.hud else {
            return;
        };
        model::set_int(hud.health, health);
        model::set_percent(hud.health_percent, ratio * 100.0);
        model::set_string(hud.health_caption, &caption);
    });
}
