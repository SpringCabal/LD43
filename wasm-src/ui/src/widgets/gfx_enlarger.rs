use spring_addons::event::EventResult;
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

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        enlarger::sweep();
    }

    fn draw_unit(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        unit_id: i32,
        draw_mode: i32,
    ) -> EventResult {
        enlarger::draw_unit(unit_id, draw_mode).into()
    }
}
