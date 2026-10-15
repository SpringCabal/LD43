extern crate alloc;

mod cursor;
mod game;
mod gl_uniforms;
mod state;
mod widgets;

spring_addons::export_ui_widgets! {
    state: state::Global,
    setup: widgets::setup,
    scratch: 4096,
}
