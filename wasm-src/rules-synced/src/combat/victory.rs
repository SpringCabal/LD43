use alloc::format;

use crate::defs::{UnitKind, defs};
use crate::diagnostics;
use crate::state::{GameEnd, Global};
use crate::units::ENEMY_TEAM;

use super::orks;

/// Declares victory once the final wave has come and the boss is dead, and
/// sends every ork fleeing. Returns whether it declared victory.
pub fn declare_if_won(global: &Global) -> bool {
    if global.game_end.get() != GameEnd::VictoryPossible || boss_alive() {
        return false;
    }
    global.game_end.set(GameEnd::Victory);
    diagnostics::info("victory");
    let fleeing = {
        let mut orks = global.orks.access_mut();
        orks.start_fleeing();
        orks.units().to_vec()
    };
    for unit_id in &fleeing {
        orks::flee(global, *unit_id);
    }
    diagnostics::info(&format!("{} orks fleeing", fleeing.len()));
    true
}

fn boss_alive() -> bool {
    let Some(def_id) = defs().id(UnitKind::OrkBoss) else {
        return true;
    };
    spring::get_team_unit_def_count(ENEMY_TEAM, def_id)
        .map(|count| count > 0)
        .unwrap_or(true)
}
