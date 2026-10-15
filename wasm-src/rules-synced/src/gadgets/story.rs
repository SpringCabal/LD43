pub(crate) mod behavior;

use alloc::rc::Rc;
use core::cell::{Cell, RefCell};

use spring::commands;
use spring_addons::AddonContext;
use spring_addons::rules::Gadget;

use self::behavior::Story;
use crate::gadgets::victory;
use crate::state::{Global, Phase};

#[derive(Default)]
pub struct StoryGadget {
    story: RefCell<Story>,
    boss_due: Rc<Cell<bool>>,
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
            behavior::spawn_wave(wave);
            self.boss_due.set(self.boss_due.get() || wave.able_to_win);
        }
        if self.boss_due.get() {
            let boss_due = Rc::clone(&self.boss_due);
            commands::run(move || {
                if crate::with_global(victory::open_victory) {
                    boss_due.set(false);
                }
            });
        }
    }
}
