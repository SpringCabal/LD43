//! State the gadgets share. Everything else lives in the gadget that owns it.

use core::cell::Cell;

use spring_addons::Resource;

use crate::combat::{Attributes, OrkAi, Statuses};
use crate::player::{Player, Spellbook};

#[derive(Default)]
pub struct Global {
    /// The frame the world was set up on; see [`Global::phase`].
    setup_frame: Cell<Option<i32>>,
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

/// Where a game frame falls relative to world setup.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Before setup, which happens on the first game frame.
    Loading,
    /// The frame that builds the town. Gadgets that set up their own part of
    /// the world do it now; the rest start on the next frame.
    Setup,
    Running,
}

impl Global {
    /// Marks `frame` as the setup frame. Only the setup gadget calls this, on
    /// its first game frame.
    pub fn begin_setup(&self, frame: i32) {
        self.setup_frame.set(Some(frame));
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
