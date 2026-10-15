pub mod behavior;
use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::state::Global;

#[derive(Default)]
pub struct TerrainGadget;

impl Gadget<Global> for TerrainGadget {
    fn name(&self) -> &'static str {
        "Terrain"
    }

    fn unit_created(
        &self,
        _ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        behavior::block(unit, def);
    }

    fn unit_destroyed(&self, _ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        behavior::unblock(event.unit_id, event.unit_def_id);
    }
}
