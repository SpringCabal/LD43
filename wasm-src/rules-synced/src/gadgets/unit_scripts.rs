use spring::commands;
use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::unit_scripts;

#[derive(Default)]
pub struct UnitScriptsGadget;

impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }

    fn init(&self, _ctx: &AddonContext<'_, Global>) {
        crate::with_cus_module_or_defer(|host| host.attach_existing());
    }

    fn game_frame(&self, _ctx: &AddonContext<'_, Global>, frame: i32) {
        wake_tasks_last(frame);
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

fn wake_tasks_last(frame: i32) {
    commands::run(move || {
        if commands::is_empty() {
            crate::with_cus_module_or_defer(move |host| host.game_frame(frame));
        } else {
            wake_tasks_last(frame);
        }
    });
}
