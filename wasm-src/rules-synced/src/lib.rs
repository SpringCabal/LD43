extern crate alloc;

mod callins;
mod combat;
mod diagnostics;
#[cfg(feature = "headless-integration")]
mod integration;
mod player;
mod random;
mod state;
mod status;
mod terrain;
mod town;
mod units;

spring::export_callin_scratch!(4096);
spring::export_game_frame!(callins::game_frame);
spring::export_handle_lua_msg!(callins::handle_lua_msg);
spring::export_unit_created!(callins::unit_created);
spring::export_unit_destroyed!(callins::unit_destroyed);
spring::export_environment_mask!(spring::rules_synced::ENVIRONMENT_MASK);
