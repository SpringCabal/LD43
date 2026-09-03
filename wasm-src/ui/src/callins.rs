use crate::audio;
use crate::camera;
use crate::enlarger;
use crate::fire;
use crate::healthbars;
use crate::highlight;
use crate::hud;
use crate::input;
use crate::lighting;
use crate::lights;
use crate::mapedge;
use crate::state;

pub fn update(delta_seconds: f32) {
    let seconds = state::with_state(|ui| {
        ui.seconds += delta_seconds.max(0.0);
        ui.seconds
    })
    .unwrap_or(0.0);
    hud::initialize();
    hud::hide_default_layout();
    hud::apply_bindings();
    lighting::apply();
    camera::apply();
    camera::track();
    audio::update();
    resize();
    hud::track_target();
    input::hold_mouse();
    lights::place_house_lights();
    lights::update_staff_light(seconds);
    lights::update_mouse_light();
    lights::update_projectile_lights();
    fire::update(delta_seconds.max(0.0));
    enlarger::sweep();
    #[cfg(feature = "e2e")]
    e2e::tick();
}

/// Drives the visual end-to-end run: dismisses the intro so gameplay is on
/// screen, then captures screenshots at fixed ticks for the runner to check.
#[cfg(feature = "e2e")]
mod e2e {
    use core::sync::atomic::{AtomicU32, Ordering};

    static TICKS: AtomicU32 = AtomicU32::new(0);

    pub fn tick() {
        let tick = TICKS.fetch_add(1, Ordering::Relaxed);
        // Tick counts, not frames: the E2E runs under software GL, where the
        // renderer manages only a handful of frames per second.
        match tick {
            5 => {
                // Capture the intro window before dismissing it.
                let _ = spring::send_commands("screenshot", "");
            }
            10 => super::hud::close_intro(),
            30 | 70 | 120 => {
                let _ = spring::send_commands("screenshot", "");
            }
            _ => {}
        }
    }
}

pub fn recv_from_synced(message: &[u8]) {
    let Ok(text) = core::str::from_utf8(message) else {
        return;
    };
    if text.starts_with("TTLD_STATUS|") {
        hud::update(text);
    }
}

pub fn mouse_move(x: i32, y: i32, _dx: i32, _dy: i32, _button: i32) -> bool {
    input::mouse_move(x, y)
}

pub fn mouse_press(x: i32, y: i32, button: i32) -> bool {
    input::mouse_press(x, y, button)
}

pub fn key_press<'a>(
    key_code: i32,
    _alt: bool,
    ctrl: bool,
    _meta: bool,
    _shift: bool,
    _is_repeat: bool,
    _label: &[u8],
    _utf32_char: i32,
    _scan_code: i32,
    _action_list: impl Iterator<Item = (&'a [u8], &'a [u8], &'a [u8])>,
) -> bool {
    input::key_press(key_code, ctrl)
}

fn resize() {
    let Ok(geometry) = spring::display::get_view_geometry(0) else {
        return;
    };
    hud::render(geometry.view_size_x, geometry.view_size_y);
}

pub fn draw_world() {
    let seconds = state::with_state(|ui| ui.seconds).unwrap_or(0.0);
    healthbars::draw(seconds);
    fire::draw();
}

pub fn draw_world_pre_unit(_unused: u8) {
    mapedge::draw();
    highlight::draw_pre_unit();
}

pub fn draw_world_refraction(_unused: u8) {
    mapedge::draw();
    highlight::draw_refraction();
}

pub fn draw_unit(unit_id: i32, draw_mode: i32) -> bool {
    enlarger::draw_unit(unit_id, draw_mode)
}
