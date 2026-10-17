pub mod hud;
use spring_addons::ui::Widget;

use self::hud::Document;
use crate::state::Global;

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

    fn shutdown(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = ctx.spring();
        self.document.shutdown(spring, ctx.global());
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = ctx.spring();
        let global = ctx.global();
        self.document.initialize(spring, global);
        if let Ok(geometry) = spring.display().get_view_geometry() {
            global
                .view
                .set((geometry.view_size_x, geometry.view_size_y));
            self.document
                .render(spring, geometry.view_size_x, geometry.view_size_y);
        }
        self.document.show_status(spring, global);
        hud::evaluate_endgame(spring, global);
        hud::track_target(spring, global);
    }
}
