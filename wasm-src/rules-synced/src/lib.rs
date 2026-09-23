extern crate alloc;

mod cmd;
mod combat;
mod defs;
mod diagnostics;
mod engine;
mod gadgets;
#[cfg(feature = "headless-integration")]
mod integration;
mod lifecycle;
mod orchestration;
mod player;
#[cfg(feature = "pose-probe")]
mod pose_probe;
mod random;
#[cfg(feature = "e2e")]
mod render_test;
mod setup;
mod state;
mod status;
mod terrain;
mod town;
mod unit_scripts;
mod units;

spring_addons::export_rules_gadgets! {
    state: state::Global,
    setup: gadgets::setup,
    scratch: 4096,
}

spring::export_core_cus!(unit_scripts::TtldCusModule);
