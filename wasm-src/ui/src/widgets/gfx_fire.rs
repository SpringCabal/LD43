use spring_addons::ui::Widget;

use crate::fire;
use crate::state::Global;

#[derive(Default)]
pub struct GfxFireWidget;

impl GfxFireWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for GfxFireWidget {
    fn name(&self) -> &'static str {
        "GfxFire"
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, dt: f32) {
        fire::update(dt.max(0.0));
    }

    fn draw_world(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        fire::draw();
    }
}
