use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::combat::{self, Effect, Modifier};
use crate::state::Global;
use crate::units::PLAYER_TEAM;

#[derive(Default)]
pub struct CombatGadget;

impl Gadget<Global> for CombatGadget {
    fn name(&self) -> &'static str {
        "Combat"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        combat::update_paused(spring, ctx.global(), frame);
        if ctx.global().running(frame) {
            combat::update_statuses(spring, ctx.global(), frame);
        }
    }

    fn unit_created(
        &self,
        ctx: &AddonContext<'_, Global>,
        unit: i32,
        _def: i32,
        team: i32,
        _builder: i32,
    ) {
        let spring = ctx.spring();
        combat::forget(ctx.global(), unit);
        if team == PLAYER_TEAM {
            combat::add_effect(
                spring,
                ctx.global(),
                unit,
                Effect::HalfSpeed,
                Modifier {
                    move_factor: 0.5,
                    reload_factor: 1.0,
                },
            );
        }
        combat::remove_turn_inertia(spring, unit);
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        combat::forget(ctx.global(), event.unit_id);
        ctx.global().statuses.access_mut().forget(event.unit_id);
    }
}
