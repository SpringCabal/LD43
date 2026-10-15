mod attributes;
mod status;

pub use attributes::{
    Attributes, Effect, Modifier, add_effect, forget, remove_turn_inertia, update_paused,
};
pub use status::{Statuses, adrenaline, stun, update as update_statuses};
