//! State the widgets share. Everything else lives in the widget that owns it.

use core::cell::Cell;

use spring::types::Float3;
use spring_addons::Resource;

#[derive(Default)]
pub struct Global {
    /// The latest status from the rules.
    pub status: Resource<GameStatus>,
    /// The RmlUi HUD: its data bindings and which screens are open. Shared
    /// because the HUD's own buttons and the keyboard both open and close
    /// screens.
    pub hud: Resource<Hud>,
    /// The view size in pixels.
    pub view: Cell<(i32, i32)>,
    /// What the cursor pointed at when this update started.
    pub cursor: Cell<Cursor>,
}

/// How far the story has got, as the rules publish it in `gameEnd`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameEnd {
    #[default]
    Start,
    VictoryPossible,
    Victory,
}

impl GameEnd {
    pub fn parse(value: &str) -> Self {
        match value {
            "victory" => Self::Victory,
            "victoryPossible" => Self::VictoryPossible,
            _ => Self::Start,
        }
    }
}

/// The rules' `TTLD_STATUS` message.
#[derive(Clone, Copy)]
pub struct GameStatus {
    /// Bumped whenever a message arrives, so readers can tell it changed.
    pub revision: u32,
    pub player_unit: i32,
    pub player_alive: bool,
    pub health: i32,
    pub max_health: i32,
    pub casting: bool,
    pub cast_start_frame: i32,
    pub cast_animation_time: f32,
    pub villagers_killed: i32,
    pub allies_drained: i32,
    pub orks_killed: i32,
    pub game_end: GameEnd,
}

impl Default for GameStatus {
    fn default() -> Self {
        Self {
            revision: 0,
            player_unit: -1,
            player_alive: true,
            health: 0,
            max_health: 1000,
            casting: false,
            cast_start_frame: 0,
            cast_animation_time: 0.0,
            villagers_killed: 0,
            allies_drained: 0,
            orks_killed: 0,
            game_end: GameEnd::Start,
        }
    }
}

impl GameStatus {
    /// Applies the `key=value` fields of a `TTLD_STATUS` message.
    pub fn apply<'a>(&mut self, fields: impl Iterator<Item = (&'a str, &'a str)>) {
        let (mut health, mut max_health) = (0, 1000);
        for (key, value) in fields {
            match key {
                "health" => health = value.parse().unwrap_or(0),
                "maxHealth" => max_health = value.parse().unwrap_or(1000),
                "villagersKilled" => parse_into(value, &mut self.villagers_killed),
                "alliesDrained" => parse_into(value, &mut self.allies_drained),
                "orksKilled" => parse_into(value, &mut self.orks_killed),
                "gameEnd" => self.game_end = GameEnd::parse(value),
                "playerAlive" => self.player_alive = value != "0",
                "playerUnit" => self.player_unit = value.parse().unwrap_or(-1),
                "castStart" => self.cast_start_frame = value.parse().unwrap_or(0),
                "castAnimTime" => self.cast_animation_time = value.parse().unwrap_or(0.0),
                "casting" => self.casting = value != "0",
                _ => {}
            }
        }
        self.health = health;
        self.max_health = max_health;
        self.revision = self.revision.wrapping_add(1);
    }
}

fn parse_into(value: &str, target: &mut i32) {
    if let Ok(value) = value.parse() {
        *target = value;
    }
}

/// Handles of the RmlUi data-model variables.
#[derive(Clone, Copy, Default)]
pub struct HudBindings {
    pub health: u64,
    pub health_percent: u64,
    pub health_caption: u64,
    pub target_visible: u64,
    pub target_enemy: u64,
    pub target_name: u64,
    pub target_percent: u64,
    pub target_caption: u64,
    pub villagers_killed: u64,
    pub allies_drained: u64,
    pub orks_killed: u64,
    pub intro_visible: u64,
    pub endgame_visible: u64,
    pub endgame_victory: u64,
    pub endgame_title: u64,
    pub endgame_line_one: u64,
    pub endgame_line_two: u64,
}

pub struct Hud {
    /// `None` until the HUD document has loaded.
    pub bindings: Option<HudBindings>,
    pub intro_open: bool,
    pub endgame_open: bool,
    /// The hovered unit and health percentage the target panel shows.
    pub target: Option<(i32, i32)>,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            bindings: None,
            intro_open: true,
            endgame_open: false,
            target: None,
        }
    }
}

impl Hud {
    /// The intro or end screen is up and takes the input.
    pub fn modal(&self) -> bool {
        self.intro_open || self.endgame_open
    }
}

#[derive(Clone, Copy, Default)]
pub struct Cursor {
    /// Screen position; `None` while the cursor is off the window.
    pub screen: Option<(f32, f32)>,
    /// The ground under the cursor.
    pub ground: Option<Float3>,
    /// The unit under the cursor.
    pub unit: Option<i32>,
}
