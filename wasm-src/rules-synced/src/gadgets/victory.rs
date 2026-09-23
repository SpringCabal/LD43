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
            combat::declare_if_won(ctx.global());
        }
    }
}
