use spring_addons::ui::Widget;

use crate::highlight;
use crate::state::Global;

#[derive(Default)]
pub struct GfxHighlightWidget;

impl GfxHighlightWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for GfxHighlightWidget {
    fn name(&self) -> &'static str {
        "GfxHighlight"
    }

    fn draw_world_pre_unit(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        highlight::draw_pre_unit();
    }

    fn draw_world_refraction(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        highlight::draw_refraction();
    }
}
