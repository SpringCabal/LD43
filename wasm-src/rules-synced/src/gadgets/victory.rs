use spring::commands;
use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::combat;
use crate::state::Global;

/// Declares victory once the boss of the final wave is dead.
#[derive(Default)]
pub struct VictoryGadget;

impl Gadget<Global> for VictoryGadget {
    fn name(&self) -> &'static str {
        "Victory"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if ctx.global().running(frame) {
            // After the story's and the ork AI's commands, whose results it
            // reads and whose orders its retreat replaces.
            commands::run(|| {
                crate::with_global(combat::declare_if_won);
            });
        }
    }
}
