pub mod behavior;
use alloc::rc::Rc;
use core::cell::RefCell;

use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use self::behavior::Villagers;
use crate::state::{Global, Phase};

#[derive(Default)]
pub struct TownGadget {
    villagers: Rc<RefCell<Villagers>>,
}

const VILLAGER_UPDATE_DIVISOR: usize = 90;

impl Gadget<Global> for TownGadget {
    fn name(&self) -> &'static str {
        "Town"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        let spring = ctx.spring();
        match ctx.global().phase(frame) {
            Phase::Loading => {}
            Phase::Setup => behavior::populate(spring, Rc::clone(&self.villagers)),
            Phase::Running => {
                let villagers = Rc::clone(&self.villagers);
                crate::with_addon_context(|ctx| {
                    ctx.commands().run(move |spring| {
                        let batch = villagers.borrow_mut().next_batch(VILLAGER_UPDATE_DIVISOR);
                        for villager in batch {
                            behavior::check_idle(spring, villager);
                        }
                    })
                });
            }
        }
    }

    fn unit_destroyed(&self, ctx: &AddonContext<'_, Global>, event: &UnitDestroyedEvent) {
        let was_villager = self.villagers.borrow_mut().remove(event.unit_id);
        if was_villager {
            ctx.global().score.access_mut().villagers_killed += 1;
        }
    }
}
