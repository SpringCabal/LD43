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
            crate::with_addon_context(|ctx| {
                ctx.commands().run(|spring| {
                    crate::with_global(|global| declare_if_won(spring, global));
                })
            });
        }
    }
}

pub fn open_victory(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) -> bool {
    if !boss_alive(spring) {
        spring::log_warning!(
            spring,
            "ttld-wasm",
            "the final wave has no boss; spawning another"
        );
        story::spawn_boss(spring);
        return false;
    }
    global.game_end.set(GameEnd::VictoryPossible);
    true
}

pub fn declare_if_won(spring: &spring::Spring<'_, spring::RulesSynced>, global: &Global) -> bool {
    if global.game_end.get() != GameEnd::VictoryPossible || boss_alive(spring) {
        return false;
    }
    global.game_end.set(GameEnd::Victory);
    spring::log_info!(spring, "ttld-wasm", "victory");
    let fleeing = {
        let mut orks = global.orks.access_mut();
        orks.start_fleeing();
        orks.units().to_vec()
    };
    for unit_id in &fleeing {
        orks::flee(spring, global, *unit_id);
    }
    spring::log_info!(spring, "ttld-wasm", "{} orks fleeing", fleeing.len());
    true
}

pub fn boss_alive(spring: &spring::Spring<'_, spring::RulesSynced>) -> bool {
    let Some(def_id) = defs(spring).id(UnitKind::OrkBoss) else {
        return true;
    };
    spring
        .units_query()
        .get_team_unit_def_count(ENEMY_TEAM, def_id)
        .map(|count| count > 0)
        .unwrap_or(true)
}
