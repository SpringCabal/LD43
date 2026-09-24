use core::cell::RefCell;

use alloc::vec::Vec;

use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitPreDamagedEvent};

use crate::defs::defs;
use crate::integration;
use crate::state::Global;

/// Runs the headless gameplay checks.
#[derive(Default)]
pub struct IntegrationGadget {
    tracker: RefCell<integration::Tracker>,
    /// Fireball damage seen since the last frame. Kept apart from the
    /// tracker, which is borrowed while checks run and can be when damage
    /// arrives.
    fireball_hits: RefCell<Vec<(i32, f32)>>,
}

impl Gadget<Global> for IntegrationGadget {
    fn name(&self) -> &'static str {
        "Integration"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if ctx.global().running(frame) {
            let hits = self.fireball_hits.take();
            {
                let mut tracker = self.tracker.borrow_mut();
                for (unit_id, damage) in hits {
                    integration::fireball_hit(&mut tracker, unit_id, damage);
                }
            }
            integration::update(&self.tracker, ctx.global(), frame);
        }
    }

    fn unit_pre_damaged(
        &self,
        _ctx: &AddonContext<'_, Global>,
        event: &UnitPreDamagedEvent,
    ) -> Option<spring::DamageResult> {
        if defs().fireball == Some(event.weapon_def_id) {
            self.fireball_hits
                .borrow_mut()
                .push((event.unit_id, event.damage));
        }
        None
    }
}
