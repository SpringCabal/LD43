use core::cell::{Cell, RefCell};

use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use crate::combat::{self, Story};
use crate::state::{Global, Phase};

/// Sends the ork waves on schedule until the town is saved.
#[derive(Default)]
pub struct StoryGadget {
    story: RefCell<Story>,
    /// The final wave has come, but victory isn't possible until its boss is
    /// on the map.
    boss_due: Cell<bool>,
}

impl Gadget<Global> for StoryGadget {
    fn name(&self) -> &'static str {
        "Story"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let global = ctx.global();
        match global.phase(frame) {
            Phase::Loading => return,
            Phase::Setup => {
                *self.story.borrow_mut() = Story::default();
                self.boss_due.set(false);
                return;
            }
            Phase::Running => {}
        }
        let wave = self
            .story
            .borrow_mut()
            .take_due_wave(frame, global.game_end.get());
        if let Some(wave) = wave {
            combat::spawn_wave(wave);
            self.boss_due.set(self.boss_due.get() || wave.able_to_win);
        }
        if self.boss_due.get() && combat::open_victory(global) {
            self.boss_due.set(false);
        }
    }
}
