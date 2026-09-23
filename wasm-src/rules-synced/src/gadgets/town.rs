use core::cell::RefCell;

use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitDestroyedEvent};

use crate::state::{Global, Phase};
use crate::town::{self, Villagers};

/// Builds the town on the setup frame, then keeps its villagers at their posts.
#[derive(Default)]
pub struct TownGadget {
    villagers: RefCell<Villagers>,
}

/// Every villager is checked once every this many frames.
const VILLAGER_UPDATE_DIVISOR: usize = 90;

impl Gadget<Global> for TownGadget {
    fn name(&self) -> &'static str {
        "Town"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        match ctx.global().phase(frame) {
            Phase::Loading => {}
            Phase::Setup => {
                // Populating creates units, whose UnitDestroyed/UnitCreated
                // must find the villagers unborrowed.
                let villagers = town::populate();
                *self.villagers.borrow_mut() = Villagers::new(villagers);
            }
            Phase::Running => {
                let batch = self
                    .villagers
                    .borrow_mut()
                    .next_batch(VILLAGER_UPDATE_DIVISOR);
                for villager in batch {
                    town::check_idle(villager);
                }
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
