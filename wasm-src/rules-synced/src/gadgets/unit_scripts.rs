use core::cell::Cell;

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::unit_scripts;

/// Gives every unit its script as soon as it exists.
#[derive(Default)]
pub struct UnitScriptsGadget {
    started: Cell<bool>,
}

impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }

    /// Units created before the game starts (by the start script, or before a
    /// rules reload) get their scripts on the first frame. Before it,
    /// `CusHost` would start their clocks at frame -1 read as `u32::MAX`, and
    /// a task that sleeps would never wake (engine TODO I.1).
    fn game_frame(&self, _ctx: &AddonContext<'_, Global>, _frame: i32) {
        if !self.started.replace(true) {
            crate::with_cus_module_or_defer(|host| host.attach_existing());
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
        // Not `started`: the first frame's own units (the town) are created
        // by gadgets that run before this one's game_frame.
        if spring::game::current_frame() >= 0 {
            unit_scripts::attach(unit, def);
        }
    }
}
