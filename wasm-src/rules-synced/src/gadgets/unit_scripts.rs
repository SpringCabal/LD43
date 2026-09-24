use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::unit_scripts;

/// Gives every unit its script as soon as it exists.
#[derive(Default)]
pub struct UnitScriptsGadget;

impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }

    /// Units that already exist when the rules load (after a rules reload).
    fn init(&self, _ctx: &AddonContext<'_, Global>) {
        crate::with_cus_module_or_defer(|host| host.attach_existing());
    }

    fn unit_created(
        &self,
        _ctx: &AddonContext<'_, Global>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        unit_scripts::attach(unit, def);
    }
}
