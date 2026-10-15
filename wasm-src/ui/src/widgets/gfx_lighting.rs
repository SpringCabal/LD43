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

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        self.lights.borrow_mut().init();
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        unit_id: i32,
        unit_def_id: i32,
        _unit_team: i32,
        _builder_id: i32,
    ) {
        self.lights.borrow_mut().unit_created(unit_id, unit_def_id);
    }

    fn unit_destroyed(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        event: &spring_addons::UnitDestroyedEvent,
    ) {
        self.lights.borrow_mut().unit_destroyed(event.unit_id);
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        lighting::apply(&self.sun_applied);
        self.lights.borrow_mut().update(ctx.global());
    }

    fn draw_screen_effects(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global>,
        width: i32,
        height: i32,
    ) {
        let player_unit = ctx.global().status.access().player_unit;
        self.pass
            .draw(&self.lights.borrow(), player_unit, width, height);
    }

    fn shutdown(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        self.pass.shutdown();
    }
}
