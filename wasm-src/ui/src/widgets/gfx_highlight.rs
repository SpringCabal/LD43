mod highlight;

use spring_addons::ui::Widget;

use self::highlight::Highlight;
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

    fn draw_world_pre_unit(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = ctx.spring();
        self.highlight.draw_pre_unit(spring, ctx.global());
    }

    fn draw_world_refraction(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = ctx.spring();
        self.highlight.draw_refraction(spring, ctx.global());
    }
}
