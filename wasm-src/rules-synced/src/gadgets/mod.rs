pub(crate) mod combat;
#[cfg(feature = "headless-integration")]
pub(crate) mod integration;
pub(crate) mod orks;
pub(crate) mod player;
#[cfg(feature = "e2e")]
pub(crate) mod render_test;
pub(crate) mod setup;
pub(crate) mod status;
pub(crate) mod story;
pub(crate) mod terrain;
pub(crate) mod town;
pub(crate) mod unit_scripts;
pub(crate) mod victory;

pub use setup::setup;
