mod document;
mod endgame;
mod model;
mod target;

pub use document::Document;
pub use endgame::{close_intro, evaluate as evaluate_endgame};
pub use model::{exit_to_menu, restart};
pub use target::track as track_target;
