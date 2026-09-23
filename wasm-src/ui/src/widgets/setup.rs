use spring_addons::register_widgets;
use spring_addons::ui::WidgetHandler;

use super::*;
use crate::state::Global;

/// Registers the widgets in the order they update and draw.
pub fn setup(handler: &mut WidgetHandler<Global>) {
    register_widgets!(
        handler,
        [
            GuiInputWidget::new(),
            ApiSyncWidget::new(),
            GuiHudWidget::new(),
            GfxLightingWidget::new(),
            GuiCameraWidget::new(),
            SndMusicWidget::new(),
            GuiMinimapWidget::new(),
            GfxFireWidget::new(),
            GfxMapEdgeWidget::new(),
            GfxHighlightWidget::new(),
        ]
    );
    #[cfg(feature = "e2e")]
    register_widgets!(handler, [DbgE2eWidget::new()]);
}
