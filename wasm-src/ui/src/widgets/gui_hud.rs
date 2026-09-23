use spring_addons::ui::Widget;

use crate::hud::{self, Document};
use crate::state::Global;

/// The RmlUi HUD: health, the hovered unit, the intro and end screens.
#[derive(Default)]
pub struct GuiHudWidget {
    document: Document,
}

impl GuiHudWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for GuiHudWidget {
    fn name(&self) -> &'static str {
        "GuiHud"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let global = ctx.global();
        self.document.initialize(global);
        self.document.hide_default_layout();
        self.document.apply_bindings();
        if let Ok(geometry) = spring::display::get_view_geometry(0) {
            global
                .view
                .set((geometry.view_size_x, geometry.view_size_y));
            self.document
                .render(geometry.view_size_x, geometry.view_size_y);
        }
        self.document.show_status(global);
        hud::evaluate_endgame(global);
        hud::track_target(global);
    }
}
