pub mod fire;
use core::cell::{Cell, RefCell};

use spring_addons::UnitDestroyedEvent;
use spring_addons::ui::Widget;

use self::fire::Fire;
use crate::state::Global;

#[derive(Default)]
pub struct GfxFireWidget {
    fire: RefCell<Fire>,
    house_def: Cell<Option<i32>>,
    #[cfg(feature = "fire-probe")]
    probe: RefCell<fire::probe::Probe>,
}

impl GfxFireWidget {
    pub fn new() -> Self {
        Self::default()
    }

    fn is_house(&self, def_id: i32) -> bool {
        self.house_def.get() == Some(def_id)
    }
}

impl Widget<Global> for GfxFireWidget {
    fn name(&self) -> &'static str {
        "GfxFire"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = _ctx.spring();
        self.house_def
            .set(spring.unit_defs().get_unit_def_id_by_name("house").ok());
        let mut fire = self.fire.borrow_mut();
        for unit_id in spring.units_query().get_all_units().unwrap_or_default() {
            if spring
                .units_info()
                .get_unit_def_id(unit_id)
                .is_ok_and(|def_id| self.is_house(def_id))
            {
                fire.add_house(unit_id);
            }
        }
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        unit_id: i32,
        unit_def_id: i32,
        _unit_team: i32,
        _builder_id: i32,
    ) {
        if self.is_house(unit_def_id) {
            self.fire.borrow_mut().add_house(unit_id);
        }
    }

    fn unit_destroyed(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        event: &UnitDestroyedEvent,
    ) {
        self.fire.borrow_mut().remove_house(event.unit_id);
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = _ctx.spring();
        if cfg!(feature = "fire-probe") {
            #[cfg(feature = "fire-probe")]
            self.probe
                .borrow_mut()
                .update(spring, &mut self.fire.borrow_mut());
        } else {
            self.fire.borrow_mut().update(spring);
        }
    }

    fn shutdown(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        self.fire.borrow().shutdown(ctx.spring());
    }

    fn draw_world(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = _ctx.spring();
        self.fire.borrow().draw(spring);
    }
}
