//! Gallery mode, for the verification galleries
//! (`verification/fixtures/gallery/gallery.lua` in springcabal-games).  They
//! set the `gallery` modoption, place the camera themselves and capture single
//! units and effects, so only the widgets that draw what a player sees in the
//! world stay: no HUD, RmlUi document, minimap, input, camera or music.

use spring_addons::register_widgets;
use spring_addons::ui::{HideDefaultInterface, Widget, WidgetHandler};

use super::*;
use crate::cursor;
use crate::state::{Cursor, Global};

/// Whether the `gallery` modoption is set.
pub fn active() -> bool {
    spring::game_options::get_mod_option("gallery")
        .ok()
        .flatten()
        .is_some_and(|mode| !mode.is_empty())
}

/// The lighting pass (sun, house, staff, cursor and projectile lights, bloom),
/// burning houses, the map edge and the hover highlight. The staff light's
/// flicker stays still: it follows real time, which differs between runs.
pub fn setup(handler: &mut WidgetHandler<Global>) {
    register_widgets!(
        handler,
        [
            HideDefaultInterface::new(setup::HIDDEN_INTERFACE),
            GalleryPointerWidget,
            ApiSyncWidget::new(),
            GfxLightingWidget::new(true),
            GfxFireWidget::new(),
            GfxMapEdgeWidget::new(),
            GfxHighlightWidget::new(),
        ]
    );
    spring::log_notice!(
        "ttld-ui",
        "gallery mode: HideDefaultInterface GalleryPointer ApiSync GfxLighting GfxFire GfxMapEdge GfxHighlight"
    );
}

/// Stands in for the player's pointer, which the input and HUD widgets track
/// in the game: the view size, and a cursor resting on the middle of the view,
/// where the gallery puts its subject.  The cursor light and the highlight
/// follow it.
struct GalleryPointerWidget;

impl Widget<Global> for GalleryPointerWidget {
    fn name(&self) -> &'static str {
        "GalleryPointer"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let Ok(geometry) = spring::display::get_view_geometry(0) else {
            return;
        };
        let (width, height) = (geometry.view_size_x, geometry.view_size_y);
        let (x, y) = (width as f32 / 2.0, height as f32 / 2.0);
        let global = ctx.global();
        global.view.set((width, height));
        global.cursor.set(Cursor {
            screen: Some((x, y)),
            ground: cursor::ground_at(x, y),
            unit: cursor::unit_at(x, y),
        });
    }
}
