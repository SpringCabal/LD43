use spring::default_interface::{
    CLOCK, CONSOLE, END_GRAPH, PLAYER_INFO, RESOURCE_BAR, TOOLTIP, UNIT_ICONS,
};
use spring_addons::register_widgets;
use spring_addons::ui::{HideDefaultInterface, WidgetHandler};

use super::*;
use crate::state::Global;

/// Registers the widgets in the order they update and draw.
pub fn setup(handler: &mut WidgetHandler<Global>) {
    register_widgets!(
        handler,
        [
            // The minimap stays: the minimap widget draws the engine's.
            HideDefaultInterface::new(
                CONSOLE | RESOURCE_BAR | TOOLTIP | CLOCK | PLAYER_INFO | UNIT_ICONS | END_GRAPH
            ),
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
