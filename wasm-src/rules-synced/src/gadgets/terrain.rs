use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::state::Global;
use crate::terrain;

/// Makes the ground under buildings impassable while they stand.
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
        terrain::block(unit, def);
    }

    fn unit_destroyed(&self, _ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        terrain::unblock(event.unit_id, event.unit_def_id);
    }
}
