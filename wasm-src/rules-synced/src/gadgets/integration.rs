use alloc::rc::Rc;
use core::cell::RefCell;

use spring::commands;
use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitPreDamagedEvent};

use crate::defs::{UnitKind, defs};
use crate::integration;
use crate::state::Global;

/// Runs the headless gameplay checks.
#[derive(Default)]
pub struct IntegrationGadget {
    /// Shared with the commands the checks run as.
    tracker: Rc<RefCell<integration::Tracker>>,
}

impl Gadget<Global> for IntegrationGadget {
    fn name(&self) -> &'static str {
        "Integration"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if ctx.global().running(frame) {
            // The checks see the frame as the other gadgets' commands left it.
            let tracker = Rc::clone(&self.tracker);
            commands::run(move || {
                crate::with_global(|global| integration::update(&tracker, global, frame));
            });
        }
    }

    fn unit_pre_damaged(
        &self,
        _ctx: &AddonContext<'_, Global>,
        event: &UnitPreDamagedEvent,
    ) -> Option<spring::DamageResult> {
        let mut tracker = self.tracker.borrow_mut();
        if defs().fireball == Some(event.weapon_def_id) {
            integration::fireball_hit(&mut tracker, event.unit_id, event.damage);
        }
        if event.attacker_id >= 0 && defs().is(event.attacker_def_id, UnitKind::OrkSmall) {
            integration::bite(
                &mut tracker,
                integration::Bite {
                    attacker: event.attacker_id,
                    frame: spring::game::current_frame(),
                },
            );
        }
        None
    }
}
