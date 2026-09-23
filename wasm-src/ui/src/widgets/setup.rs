use spring_addons::register_widgets;
use spring_addons::ui::WidgetHandler;

use super::*;
use crate::state::Global;

pub fn setup(handler: &mut WidgetHandler<Global>) {
    register_widgets!(
        handler,
        [
            ApiSyncWidget::new(),
            GuiHudWidget::new(),
            GfxLightingWidget::new(),
            GuiCameraWidget::new(),
            SndMusicWidget::new(),
            GuiHudViewWidget::new(),
            GuiMinimapWidget::new(),
            GuiInputWidget::new(),
            GfxFireWidget::new(),
            GfxMapEdgeWidget::new(),
            GfxHighlightWidget::new(),
        ]
    );
    #[cfg(feature = "e2e")]
    register_widgets!(handler, [DbgE2eWidget::new()]);
}
