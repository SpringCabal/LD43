use spring_addons::register_gadgets;
use spring_addons::rules::{Gadget, GadgetHandler};

use super::*;
use crate::state::{self, Global};

#[derive(Default)]
pub struct SetupGadget;

impl SetupGadget {
    pub fn new() -> Self {
        Self
    }
}

impl Gadget<Global> for SetupGadget {
    fn name(&self) -> &'static str {
        "Setup"
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        let _ = state::with_world(|world| {
            world.frame = frame;
            world.frame_consumed = false;
        });
        if crate::setup::take_setup_flag() {
            crate::setup::setup();
            let _ = state::with_world(|world| world.frame_consumed = true);
        }
    }
}

pub fn setup(handler: &mut GadgetHandler<Global>) {
    register_gadgets!(
        handler,
        [
            SetupGadget::new(),
            CombatGadget::new(),
            PlayerGadget::new(),
            StoryGadget::new(),
            TownGadget::new(),
            VictoryGadget::new(),
            StatusGadget::new(),
        ]
    );

    #[cfg(feature = "headless-integration")]
    register_gadgets!(handler, [IntegrationGadget::new()]);

    register_gadgets!(handler, [LifecycleGadget::new(), InputGadget::new()]);
}

pub fn frame_consumed() -> bool {
    state::with_world(|world| world.frame_consumed).unwrap_or(false)
}
