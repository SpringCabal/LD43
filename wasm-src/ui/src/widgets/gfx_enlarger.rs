use spring_addons::ui::Widget;

use crate::enlarger;
use crate::state::Global;

#[derive(Default)]
pub struct GfxEnlargerWidget;

impl GfxEnlargerWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for GfxEnlargerWidget {
    fn name(&self) -> &'static str {
        "GfxEnlarger"
    }

    fn init(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        enlarger::reset();
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        enlarger::sweep();
    }
}
