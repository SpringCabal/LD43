use spring_addons::rules::Gadget;

use crate::combat;
use crate::gadgets::setup::frame_consumed;
use crate::state::Global;

#[derive(Default)]
pub struct CombatGadget;

impl CombatGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for CombatGadget {
    fn name(&self) -> &'static str {
        "Combat"
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        if frame_consumed() {
            return;
        }
        combat::status_update(frame);
    }
}
