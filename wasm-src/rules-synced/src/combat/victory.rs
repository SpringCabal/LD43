use crate::defs::{UnitKind, defs};
use crate::state::{GameEnd, Global};
use crate::units::ENEMY_TEAM;

use super::orks;
use super::story;

/// Opens the way to victory once the final wave's boss is on the map; false
/// while it isn't. A boss that failed to spawn would otherwise count as
/// already killed, and the wave would win the game on arrival, so another is
/// spawned and the caller asks again next frame.
pub fn open_victory(global: &Global) -> bool {
    if !boss_alive() {
        spring::log_warning!("ttld-wasm", "the final wave has no boss; spawning another");
        story::spawn_boss();
        return false;
    }
    global.game_end.set(GameEnd::VictoryPossible);
    true
}

/// Declares victory once the final wave has come and the boss is dead, and
/// sends every ork fleeing. Returns whether it declared victory.
pub fn declare_if_won(global: &Global) -> bool {
    if global.game_end.get() != GameEnd::VictoryPossible || boss_alive() {
        return false;
    }
    global.game_end.set(GameEnd::Victory);
    spring::log_info!("ttld-wasm", "victory");
    let fleeing = {
        let mut orks = global.orks.access_mut();
        orks.start_fleeing();
        orks.units().to_vec()
    };
    for unit_id in &fleeing {
        orks::flee(global, *unit_id);
    }
    spring::log_info!("ttld-wasm", "{} orks fleeing", fleeing.len());
    true
}

pub fn boss_alive() -> bool {
    let Some(def_id) = defs().id(UnitKind::OrkBoss) else {
        return true;
    };
    spring::get_team_unit_def_count(ENEMY_TEAM, def_id)
        .map(|count| count > 0)
        .unwrap_or(true)
}
