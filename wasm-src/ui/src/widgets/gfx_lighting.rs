mod deferred_lighting;
mod lighting;
mod lights;

use core::cell::{Cell, RefCell};

use spring_addons::ui::Widget;

use self::deferred_lighting::DeferredPass;
use self::lights::Lights;
use crate::state::Global;

#[derive(Default)]
pub struct GfxLightingWidget {
    sun_applied: Cell<bool>,
    lights: RefCell<Lights>,
    pass: DeferredPass,
}

impl GfxLightingWidget {
    pub fn new(still: bool) -> Self {
        Self {
            lights: RefCell::new(Lights::new(still)),
            ..Self::default()
        }
    }
}

impl Widget<Global> for GfxLightingWidget {
    fn name(&self) -> &'static str {
        "GfxLighting"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        self.lights.borrow_mut().init(_ctx.spring());
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        unit_id: i32,
        unit_def_id: i32,
        _unit_team: i32,
        _builder_id: i32,
    ) {
        self.lights
            .borrow_mut()
            .unit_created(_ctx.spring(), unit_id, unit_def_id);
    }

    fn unit_destroyed(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        event: &spring_addons::UnitDestroyedEvent,
    ) {
        self.lights.borrow_mut().unit_destroyed(event.unit_id);
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = ctx.spring();
        lighting::apply(spring, &self.sun_applied);
        self.lights.borrow_mut().update(spring, ctx.global());
    }

    fn draw_screen_effects(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        width: i32,
        height: i32,
    ) {
        let spring = ctx.spring();
        let player_unit = ctx.global().status.access().player_unit;
        self.pass
            .draw(spring, &self.lights.borrow(), player_unit, width, height);
    }

    fn shutdown(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = _ctx.spring();
        self.pass.shutdown(spring);
    }
}
