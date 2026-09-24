//! State the gadgets share. Everything else lives in the gadget that owns it.

use core::cell::Cell;

use spring_addons::Resource;

use crate::combat::{Attributes, OrkAi, Statuses};
use crate::player::{Player, Spellbook};

#[derive(Default)]
pub struct Global {
    /// The frame the current round was set up on; see [`Global::phase`].
    setup_frame: Cell<Option<i32>>,
    /// Rounds set up so far: 1 in the first round, one more per restart.
    round: Cell<u32>,
    pub game_end: Cell<GameEnd>,
    pub score: Resource<Score>,
    /// The blood mage: read by the input, status and spell code.
    pub player: Resource<Player>,
    /// Spells cast but not yet resolved.
    pub spells: Resource<Spellbook>,
    /// Every living ork: steered by the ork AI, sent fleeing on victory.
    pub orks: Resource<OrkAi>,
    /// Timed status effects (adrenaline, stun) from spells.
    pub statuses: Resource<Statuses>,
    /// Speed and reload modifiers on every unit.
    pub attributes: Resource<Attributes>,
}

/// Where a game frame falls relative to the setup of the round.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Before setup, which happens on the first game frame and on the first
    /// frame after a restart.
    Loading,
    /// The frame that builds the town. Gadgets that set up their own part of
    /// the world do it now; the rest start on the next frame.
    Setup,
    Running,
}

impl Global {
    /// Starts a round on `frame`: forgets the previous round's shared state
    /// and marks `frame` as the setup frame. Only the setup gadget calls
    /// this, once the previous round's units are gone.
    pub fn begin_round(&self, frame: i32) {
        self.game_end.set(GameEnd::default());
        *self.score.access_mut() = Score::default();
        *self.player.access_mut() = Player::default();
        *self.spells.access_mut() = Spellbook::default();
        *self.orks.access_mut() = OrkAi::default();
        *self.statuses.access_mut() = Statuses::default();
        *self.attributes.access_mut() = Attributes::default();
        self.setup_frame.set(Some(frame));
        self.round.set(self.round.get() + 1);
    }

    /// Asks for a new round: the setup gadget builds it on the next frame.
    pub fn restart(&self) {
        self.setup_frame.set(None);
    }

    pub fn round(&self) -> u32 {
        self.round.get()
    }

    /// The frame the current round was set up on.
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

/// How far the story has got.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum GameEnd {
    #[default]
    Start,
    /// The final wave has come; killing the boss wins.
    VictoryPossible,
    Victory,
}

impl GameEnd {
    /// The value published in the `gameEnd` rules param.
    pub fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::VictoryPossible => "victoryPossible",
            Self::Victory => "victory",
        }
    }
}

/// The end-screen counters.
#[derive(Default)]
pub struct Score {
    pub villagers_killed: i32,
    pub allies_drained: i32,
    pub orks_killed: i32,
}
