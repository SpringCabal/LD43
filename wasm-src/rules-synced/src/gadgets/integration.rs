use core::cell::RefCell;

use alloc::vec::Vec;

use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitPreDamagedEvent};

use crate::defs::{UnitKind, defs};
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
    /// Small orks' bites since the last frame, kept apart for the same reason.
    bites: RefCell<Vec<integration::Bite>>,
}

impl Gadget<Global> for IntegrationGadget {
    fn name(&self) -> &'static str {
        "Integration"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if ctx.global().running(frame) {
            let hits = self.fireball_hits.take();
            let bites = self.bites.take();
            {
                let mut tracker = self.tracker.borrow_mut();
                for (unit_id, damage) in hits {
                    integration::fireball_hit(&mut tracker, unit_id, damage);
                }
                for bite in bites {
                    integration::bite(&mut tracker, bite);
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
        if event.attacker_id >= 0 && defs().is(event.attacker_def_id, UnitKind::OrkSmall) {
            self.bites.borrow_mut().push(integration::Bite {
                attacker: event.attacker_id,
                frame: spring::game::current_frame(),
            });
        }
        None
    }
}
