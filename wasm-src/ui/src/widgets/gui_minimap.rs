mod minimap;

use spring_addons::ui::Widget;

use self::minimap::Minimap;
use crate::state::Global;

#[derive(Default)]
pub struct GuiMinimapWidget {
    minimap: Minimap,
}

impl GuiMinimapWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GuiMinimapWidget {
    fn name(&self) -> &'static str {
        "GuiMinimap"
    }

    fn draw_screen(
        &self,
        ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        width: i32,
        height: i32,
    ) {
        let spring = ctx.spring();
        let player_unit = ctx.global().status.access().player_unit;
        self.minimap.draw(spring, player_unit, width, height);
    }

    fn shutdown(&self, _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = _ctx.spring();
        self.minimap.shutdown(spring);
    }
}
