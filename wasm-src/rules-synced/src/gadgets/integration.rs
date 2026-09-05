use spring_addons::rules::Gadget;

use crate::gadgets::setup::frame_consumed;
use crate::integration;
use crate::state::Global;

#[derive(Default)]
pub struct IntegrationGadget;

impl IntegrationGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for IntegrationGadget {
    fn name(&self) -> &'static str {
        "Integration"
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        if frame_consumed() {
            return;
        }
        integration::update(frame);
    }
}
