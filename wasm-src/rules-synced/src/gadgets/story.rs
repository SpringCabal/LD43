use spring_addons::rules::Gadget;

use crate::gadgets::setup::frame_consumed;
use crate::state::Global;

#[derive(Default)]
pub struct StoryGadget;

impl StoryGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for StoryGadget {
    fn name(&self) -> &'static str {
        "Story"
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        if frame_consumed() {
            return;
        }
        crate::orchestration::advance_story(frame);
    }
}
