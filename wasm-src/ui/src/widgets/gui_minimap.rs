use spring_addons::ui::Widget;

use crate::minimap;
use crate::state::Global;

#[derive(Default)]
pub struct GuiMinimapWidget;

impl GuiMinimapWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for GuiMinimapWidget {
    fn name(&self) -> &'static str {
        "GuiMinimap"
    }

    fn draw_screen(&self, _ctx: &spring_addons::AddonContext<'_, Global>, width: i32, height: i32) {
        minimap::draw(width, height);
    }

    fn shutdown(&self, _ctx: &spring_addons::AddonContext<'_, Global>) {
        minimap::shutdown();
    }
}
