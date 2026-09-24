use alloc::vec::Vec;
use core::cell::{Cell, RefCell};

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::state::Global;

/// Gives every unit its script as soon as it exists.
#[derive(Default)]
pub struct UnitScriptsGadget {
    swept: Cell<bool>,
    /// Units the unit-script module could not take yet, attached on the next
    /// frame: those created while a unit script was running further up the
    /// stack, and those that existed before the engine initialized the module.
    pending: RefCell<Vec<(i32, i32)>>,
}

/// Attaches the unit's script; false if the unit-script module is busy.
fn attach(unit: i32, def: i32) -> bool {
    crate::with_cus_module(|module| module.attach(unit, def)).is_some()
}

impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }

    fn game_frame(&self, _ctx: &AddonContext<'_, Global>, _frame: i32) {
        let mut units = core::mem::take(&mut *self.pending.borrow_mut());
        // Units created before this module loaded never raised UnitCreated here.
        if !self.swept.replace(true) {
            units.extend(
                spring::get_all_units()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|unit| Some((unit, spring::get_unit_def_id(unit).ok()?.0))),
            );
        }
        for (unit, def) in units {
            // A unit that died meanwhile may have handed its id to a new one.
            let same_unit = spring::get_unit_def_id(unit).is_ok_and(|current| current.0 == def);
            if same_unit && !attach(unit, def) {
                self.pending.borrow_mut().push((unit, def));
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
        if !attach(unit, def) {
            self.pending.borrow_mut().push((unit, def));
        }
    }
}
