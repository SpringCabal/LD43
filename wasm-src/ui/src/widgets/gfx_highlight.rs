use spring_addons::ui::Widget;

use crate::highlight::Highlight;
use crate::state::Global;

#[derive(Default)]
pub struct GfxHighlightWidget {
    highlight: Highlight,
}

impl GfxHighlightWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GfxHighlightWidget {
    fn name(&self) -> &'static str {
        "GfxHighlight"
    }

    fn draw_world_pre_unit(&self, ctx: &spring_addons::AddonContext<'_, Global>) {
        self.highlight.draw_pre_unit(ctx.global());
    }

    fn draw_world_refraction(&self, ctx: &spring_addons::AddonContext<'_, Global>) {
        self.highlight.draw_refraction(ctx.global());
    }
}
