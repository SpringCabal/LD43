pub(crate) mod behavior;

use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::state::Global;

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
        crate::with_addon_context(|ctx| {
            ctx.commands().run(|spring| {
                let batch = crate::with_global(|global| global.orks.access_mut().next_batch());
                for unit_id in batch {
                    behavior::check_idle(spring, unit_id);
                }
            })
        });
    }

    fn unit_created(
        &self,
        ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        team: i32,
        _builder: i32,
    ) {
        let registered = ctx
            .global()
            .orks
            .access_mut()
            .created(ctx.spring(), unit, def, team);
        if registered {
            behavior::advance(ctx.spring(), unit);
        }
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        let was_ork = ctx.global().orks.access_mut().destroyed(event.unit_id);
        if was_ork {
            ctx.global().score.access_mut().orks_killed += 1;
        }
    }
}
