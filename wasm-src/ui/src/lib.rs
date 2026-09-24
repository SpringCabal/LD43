extern crate alloc;

mod audio;
mod camera;
mod cursor;
mod deferred_lighting;
mod diagnostics;
mod fire;
mod game;
mod gl_uniforms;
mod highlight;
mod hud;
mod input;
mod lighting;
mod lights;
mod mapedge;
mod minimap;
mod state;
mod widgets;

spring_addons::export_ui_widgets! {
    state: state::Global,
    setup: widgets::setup,
    scratch: 4096,
}
