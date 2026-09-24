use core::cell::{Cell, RefCell};

use spring_addons::ui::Widget;

use crate::deferred_lighting::DeferredPass;
use crate::lighting;
use crate::lights::Lights;
use crate::state::Global;

/// The sun, the point lights (houses, staff, cursor, projectiles) and the
/// deferred pass that draws them.
#[derive(Default)]
pub struct GfxLightingWidget {
    sun_applied: Cell<bool>,
    lights: RefCell<Lights>,
    pass: DeferredPass,
}

impl GfxLightingWidget {
    /// With `still` lights, time doesn't vary them (the staff flicker), so
    /// captures repeat: the visual suite and the galleries.
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
