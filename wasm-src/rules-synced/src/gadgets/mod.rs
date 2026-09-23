//! The rules gadgets. Each owns its part of the game and handles its own
//! callins; `setup` registers them in the order they run each frame.

mod combat;
#[cfg(feature = "headless-integration")]
mod integration;
mod orks;
mod player;
#[cfg(feature = "e2e")]
mod render_test;
mod setup;
mod status;
mod story;
mod terrain;
mod town;
mod unit_scripts;
mod victory;

pub use setup::setup;
