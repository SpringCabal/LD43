use spring::commands;
use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::state::Global;
use crate::unit_scripts;

/// Gives every unit its script as soon as it exists, and wakes the scripts'
/// sleeping tasks each frame.
#[derive(Default)]
pub struct UnitScriptsGadget;

impl Gadget<Global> for UnitScriptsGadget {
    fn name(&self) -> &'static str {
        "UnitScripts"
    }

    /// Units that already exist when the rules load. None do when a game
    /// starts; after a rules reload they all do.
    fn init(&self, _ctx: &AddonContext<'_, Global>) {
        crate::with_cus_module_or_defer(|host| host.attach_existing());
    }

    /// Lua wakes sleeping unit-script tasks in GameFrame, before the units
    /// update and aim; without this they would wake in the engine's CUS tick,
    /// after the unit update. This gadget runs after the others, as Lua's
    /// unit_script.lua runs after most of the game's gadgets.
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

/// Wakes the tasks once every other command of this frame has run, including
/// those that commands queued (the ork AI's and the villagers' orders): in
/// Lua all of it ran before the wake-up, and a task started before it counts
/// its sleep from the previous frame.
fn wake_tasks_last(frame: i32) {
    commands::run(move || {
        if commands::is_empty() {
            crate::with_cus_module_or_defer(move |host| host.game_frame(frame));
        } else {
            wake_tasks_last(frame);
        }
    });
}
