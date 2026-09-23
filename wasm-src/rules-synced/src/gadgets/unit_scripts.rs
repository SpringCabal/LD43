use core::cell::Cell;

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::unit_scripts;

/// Hands every unit to the Core-CUS module, which attaches its script on the
/// next CUS tick.
#[derive(Default)]
pub struct UnitScriptsGadget {
    swept: Cell<bool>,
}

impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }

    /// Units created before this module loaded never raised UnitCreated here.
    fn game_frame(&self, _ctx: &AddonContext<'_, Global>, _frame: i32) {
        if self.swept.replace(true) {
            return;
        }
        for unit in spring::get_all_units().unwrap_or_default() {
            if let Ok(def) = spring::get_unit_def_id(unit) {
                unit_scripts::queue_unit_created(unit, def.0);
            }
        }
    }

    fn unit_created(
        &self,
        _ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        unit_scripts::queue_unit_created(unit, def);
    }
}
