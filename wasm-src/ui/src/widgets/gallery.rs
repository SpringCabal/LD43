use spring_addons::ui::{HideDefaultInterface, Widget, WidgetHandler};
use springcabal::verification::WorldWidgets;

use super::*;
use crate::cursor;
use crate::state::{Cursor, Global};

pub fn setup(handler: &mut WidgetHandler<Global>) {
    WorldWidgets::new(handler)
        .widget(HideDefaultInterface::new(setup::HIDDEN_INTERFACE))
        .widget(GalleryPointerWidget)
        .widget(ApiSyncWidget::new())
        .widget(GfxLightingWidget::new(true))
        .widget(GfxFireWidget::new())
        .widget(GfxMapEdgeWidget::new())
        .widget(GfxHighlightWidget::new())
        .announce("ttld-ui");
}

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
