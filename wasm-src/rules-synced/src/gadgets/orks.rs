use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::combat;
use crate::state::Global;

/// The ork AI: every ork attacks the town, and idle orks get a new target.
#[derive(Default)]
pub struct OrksGadget;

impl Gadget<Global> for OrksGadget {
    fn name(&self) -> &'static str {
        "Orks"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if !ctx.global().running(frame) {
            return;
        }
        let batch = ctx.global().orks.access_mut().next_batch();
        for unit_id in batch {
            combat::check_idle(unit_id);
        }
    }

    fn unit_created(
        &self,
        ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        team: i32,
        _builder: i32,
    ) {
        let registered = ctx.global().orks.access_mut().created(unit, def, team);
        if registered {
            combat::advance(unit);
        }
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        let was_ork = ctx.global().orks.access_mut().destroyed(event.unit_id);
        if was_ork {
            ctx.global().score.access_mut().orks_killed += 1;
        }
    }
}
