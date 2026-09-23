use spring_addons::rules::Gadget;

use crate::gadgets::setup::frame_consumed;
use crate::state::Global;

#[derive(Default)]
pub struct VictoryGadget;

impl VictoryGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for VictoryGadget {
    fn name(&self) -> &'static str {
        "Victory"
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _frame: i32) {
        if frame_consumed() {
            return;
        }
        crate::orchestration::check_victory();
    }
}
