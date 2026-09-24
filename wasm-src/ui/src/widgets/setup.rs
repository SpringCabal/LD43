use spring::default_interface::{
    CLOCK, CONSOLE, END_GRAPH, PLAYER_INFO, RESOURCE_BAR, TOOLTIP, UNIT_ICONS,
};
use spring_addons::register_widgets;
use spring_addons::ui::{HideDefaultInterface, WidgetHandler};
use springcabal::music::MusicLoop;

use super::*;
use crate::state::Global;

const MUSIC_FILE: &str = "sounds/music.ogg";
const MUSIC_VOLUME: f32 = 0.20;
/// The track's length, in case the engine can't report it.
const MUSIC_SECONDS: f32 = 270.0;

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
            MusicLoop::new(MUSIC_FILE, MUSIC_VOLUME).length_hint(MUSIC_SECONDS),
            GuiMinimapWidget::new(),
            GfxFireWidget::new(),
            GfxMapEdgeWidget::new(),
            GfxHighlightWidget::new(),
        ]
    );
    #[cfg(feature = "e2e")]
    register_widgets!(handler, [DbgE2eWidget::new()]);
}
