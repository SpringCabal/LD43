mod control;
mod drain;
mod messages;
mod spells;

#[cfg(feature = "headless-integration")]
pub use control::PlayerCommand;
pub use control::{Order, Player};
pub use messages::parse;
pub use spells::{Spellbook, cast, fireball_exploded, resolve_pending};
