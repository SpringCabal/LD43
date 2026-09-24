use core::cell::RefCell;

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::combat::{self, Story};
use crate::state::{GameEnd, Global};

/// Sends the ork waves on schedule until the town is saved.
#[derive(Default)]
pub struct StoryGadget {
    story: RefCell<Story>,
}

impl Gadget<Global> for StoryGadget {
    fn name(&self) -> &'static str {
        "Story"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        if !global.running(frame) {
            return;
        }
        let wave = self
            .story
            .borrow_mut()
            .take_due_wave(frame, global.game_end.get());
        if let Some(wave) = wave {
            if wave.able_to_win {
                global.game_end.set(GameEnd::VictoryPossible);
            }
            combat::spawn_wave(wave);
        }
    }
}
