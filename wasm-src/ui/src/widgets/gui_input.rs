use spring_addons::event::{EventResult, KeyEvent};
use spring_addons::ui::Widget;

use crate::state::{self, Global};
use crate::{input, lights};

#[derive(Default)]
pub struct GuiInputWidget;

impl GuiInputWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<Global> for GuiInputWidget {
    fn name(&self) -> &'static str {
        "GuiInput"
    }

    fn update(&self, _ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let seconds = state::with_state(|ui| ui.seconds).unwrap_or(0.0);
        input::hold_mouse();
        lights::place_house_lights();
        lights::update_staff_light(seconds);
        lights::update_mouse_light();
        lights::update_projectile_lights();
    }

    fn mouse_move(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        x: i32,
        y: i32,
        _dx: i32,
        _dy: i32,
        _button: i32,
    ) -> EventResult {
        input::mouse_move(x, y).into()
    }

    fn mouse_wheel(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        up: bool,
        value: f32,
    ) -> EventResult {
        input::mouse_wheel(up, value).into()
    }

    fn mouse_press(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        x: i32,
        y: i32,
        button: i32,
    ) -> EventResult {
        input::mouse_press(x, y, button).into()
    }

    fn key_press(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global>,
        event: &KeyEvent<'_>,
    ) -> EventResult {
        input::key_press(event.key_code, event.ctrl).into()
    }
}
