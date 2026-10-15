mod data;
mod spawn;
mod villagers;

pub use spawn::{VillagerArea, populate};
pub use villagers::{Villagers, check_idle};
