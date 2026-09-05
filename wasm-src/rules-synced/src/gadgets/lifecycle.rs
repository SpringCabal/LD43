use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::lifecycle;
use crate::state::Global;

#[derive(Default)]
pub struct LifecycleGadget;

impl LifecycleGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for LifecycleGadget {
    fn name(&self) -> &'static str {
        "Lifecycle"
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        team: i32,
        builder: i32,
    ) {
        lifecycle::unit_created(unit, def, team, builder);
    }

    fn unit_destroyed(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        event: &UnitDestroyedEvent,
    ) {
        lifecycle::unit_destroyed(
            event.unit_id,
            event.unit_def_id,
            event.unit_team,
            event.attacker_id,
            event.attacker_def_id,
            event.attacker_team,
            event.weapon_def_id,
        );
    }
}
