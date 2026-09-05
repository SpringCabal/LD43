use spring_addons::ui::Widget;

use crate::lighting;
use crate::state::Global;

#[derive(Default)]
pub struct GfxLightingWidget;

impl GfxLightingWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for GfxLightingWidget {
    fn name(&self) -> &'static str {
        "GfxLighting"
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        lighting::apply();
    }
}
