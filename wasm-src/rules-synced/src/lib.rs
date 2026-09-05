extern crate alloc;

mod combat;
mod diagnostics;
mod gadgets;
#[cfg(feature = "headless-integration")]
mod integration;
mod lifecycle;
mod orchestration;
mod player;
mod random;
mod setup;
mod state;
mod status;
mod terrain;
mod town;
mod units;

spring_addons::export_rules_gadgets! {
    state: state::Global,
    setup: gadgets::setup,
    scratch: 4096,
}
