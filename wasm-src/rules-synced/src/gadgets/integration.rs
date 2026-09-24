use core::cell::RefCell;

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::integration;
use crate::state::Global;

/// Runs the headless gameplay checks.
#[derive(Default)]
pub struct IntegrationGadget {
    tracker: RefCell<integration::Tracker>,
}

impl Gadget<Global> for IntegrationGadget {
    fn name(&self) -> &'static str {
        "Integration"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if ctx.global().running(frame) {
            integration::update(&self.tracker, ctx.global(), frame);
        }
    }
}
