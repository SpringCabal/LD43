extern crate alloc;

mod audio;
mod camera;
mod diagnostics;
mod enlarger;
mod fire;
mod healthbars;
mod highlight;
mod hud;
mod input;
mod lighting;
mod lights;
mod mapedge;
mod state;
mod widgets;

spring_addons::export_ui_widgets! {
    state: state::Global,
    setup: widgets::setup,
    scratch: 4096,
}
