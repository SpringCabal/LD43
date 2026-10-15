use core::cell::Cell;

use spring_addons::Resource;

use crate::combat::{Attributes, Statuses};
use crate::gadgets::orks::behavior::OrkAi;
use crate::gadgets::player::behavior::{Player, Spellbook};

pub use ttld_shared::GameEnd;

#[derive(Default)]
pub struct Global {
    setup_frame: Cell<Option<i32>>,
    round: Cell<u32>,
    pub game_end: Cell<GameEnd>,
    pub score: Resource<Score>,
    pub player: Resource<Player>,
    pub spells: Resource<Spellbook>,
    pub orks: Resource<OrkAi>,
    pub statuses: Resource<Statuses>,
    pub attributes: Resource<Attributes>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Loading,
    Setup,
    Running,
}

impl Global {
    pub fn begin_round(&self, frame: i32) {
        self.setup_frame.set(Some(frame));
        self.round.set(self.round.get() + 1);
    }

    pub fn forget_round(&self) {
        self.game_end.set(GameEnd::default());
        *self.score.access_mut() = Score::default();
        *self.player.access_mut() = Player::default();
        *self.spells.access_mut() = Spellbook::default();
        *self.orks.access_mut() = OrkAi::default();
        *self.statuses.access_mut() = Statuses::default();
        *self.attributes.access_mut() = Attributes::default();
    }

    pub fn restart(&self) {
        self.setup_frame.set(None);
    }

    pub fn round(&self) -> u32 {
        self.round.get()
    }

    #[cfg(feature = "headless-integration")]
    pub fn setup_frame(&self) -> Option<i32> {
        self.setup_frame.get()
    }

    pub fn phase(&self, frame: i32) -> Phase {
        match self.setup_frame.get() {
            None => Phase::Loading,
            Some(setup) if setup == frame => Phase::Setup,
            Some(_) => Phase::Running,
        }
    }

    pub fn running(&self, frame: i32) -> bool {
        self.phase(frame) == Phase::Running
    }
}

#[derive(Default)]
pub struct Score {
    pub villagers_killed: i32,
    pub allies_drained: i32,
    pub orks_killed: i32,
}
