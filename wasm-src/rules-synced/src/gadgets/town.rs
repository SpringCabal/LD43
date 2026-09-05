use spring_addons::rules::Gadget;

use crate::gadgets::setup::frame_consumed;
use crate::orchestration;
use crate::state::Global;

#[derive(Default)]
pub struct TownGadget;

impl TownGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for TownGadget {
    fn name(&self) -> &'static str {
        "Town"
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        if frame_consumed() {
            return;
        }
        orchestration::steer_orks();
        orchestration::steer_villagers();
    }
}
