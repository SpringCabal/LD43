use spring::commands;
use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::defs::{UnitKind, defs};
use crate::state::{GameEnd, Global};
use crate::units::ENEMY_TEAM;

use crate::gadgets::orks::behavior as orks;
use crate::gadgets::story::behavior as story;

#[derive(Default)]
pub struct VictoryGadget;

impl Gadget<Global> for VictoryGadget {
    fn name(&self) -> &'static str {
        "Victory"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if ctx.global().running(frame) {
            commands::run(|| {
                crate::with_global(declare_if_won);
            });
        }
    }
}

pub fn open_victory(global: &Global) -> bool {
    if !boss_alive() {
        spring::log_warning!("ttld-wasm", "the final wave has no boss; spawning another");
        story::spawn_boss();
        return false;
    }
    global.game_end.set(GameEnd::VictoryPossible);
    true
}

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
