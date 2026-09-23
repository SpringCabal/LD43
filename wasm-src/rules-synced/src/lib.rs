extern crate alloc;

mod cmd;
mod combat;
mod defs;
mod diagnostics;
mod engine;
mod fx;
mod gadgets;
#[cfg(feature = "headless-integration")]
mod integration;
mod player;
#[cfg(feature = "pose-probe")]
mod pose_probe;
mod random;
mod state;
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
