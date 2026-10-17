use alloc::rc::Rc;
use core::cell::RefCell;

use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitPreDamagedEvent};

use crate::defs::{UnitKind, defs};
use crate::integration;
use crate::state::Global;

#[derive(Default)]
pub struct IntegrationGadget {
    tracker: Rc<RefCell<integration::Tracker>>,
}

impl Gadget<Global> for IntegrationGadget {
    fn name(&self) -> &'static str {
        "Integration"
    }

    fn game_frame(&self, ctx: &AddonContext<'_, Global>, frame: i32) {
        if ctx.global().running(frame) {
            let tracker = Rc::clone(&self.tracker);
            crate::with_addon_context(|ctx| {
                ctx.commands().run(move |spring| {
                    crate::with_global(|global| {
                        integration::update(spring, &tracker, global, frame)
                    });
                })
            });
        }
    }

    fn unit_pre_damaged(
        &self,
        _ctx: &AddonContext<'_, Global>,
        event: &UnitPreDamagedEvent,
    ) -> Option<spring::DamageResult> {
        let spring = _ctx.spring();
        let mut tracker = self.tracker.borrow_mut();
        if defs(spring).fireball == Some(event.weapon_def_id) {
            integration::fireball_hit(&mut tracker, event.unit_id, event.damage);
        }
        if event.attacker_id >= 0 && defs(spring).is(event.attacker_def_id, UnitKind::OrkSmall) {
            integration::bite(
                &mut tracker,
                integration::Bite {
                    attacker: event.attacker_id,
                    frame: spring.game().current_frame().unwrap_or(0),
                },
            );
        }
        None
    }
}
