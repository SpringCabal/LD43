use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::combat::{self, Effect, Modifier};
use crate::state::Global;
use crate::units::PLAYER_TEAM;

/// Unit attribute effects: the town's permanent half speed, and the timed
/// effects of spells wearing off.
#[derive(Default)]
pub struct CombatGadget;

impl Gadget<Global> for CombatGadget {
    fn name(&self) -> &'static str {
        "Combat"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if ctx.global().running(frame) {
            combat::update_statuses(ctx.global(), frame);
        }
    }

    fn unit_created(
        &self,
        _ctx: &AddonContext<'_, Global>,
        unit: i32,
        _def: i32,
        team: i32,
        _builder: i32,
    ) {
        // A dying unit's Killed script adds its death paralysis after
        // UnitDestroyed has already cleared its effects, so a reused unit id
        // can still carry it.
        combat::forget(unit);
        if team == PLAYER_TEAM {
            combat::add_effect(
                unit,
                Effect::HalfSpeed,
                Modifier {
                    move_factor: 0.5,
                    reload_factor: 1.0,
                },
            );
        }
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        combat::forget(event.unit_id);
        ctx.global().statuses.access_mut().forget(event.unit_id);
    }
}
