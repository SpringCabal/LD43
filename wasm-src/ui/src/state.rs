//! State the widgets share. Everything else lives in the widget that owns it.

use core::cell::Cell;

use spring::types::Float3;
use spring_addons::Resource;
pub use ttld_shared::GameEnd;
use ttld_shared::Status;

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

/// The rules' latest [`Status`], as the widgets read it.
#[derive(Clone, Copy)]
pub struct GameStatus {
    /// Bumped whenever a message arrives, so readers can tell it changed.
    pub revision: u32,
    /// The rules' round: 1, then one more per restart; 0 before the first
    /// status.
    pub round: u32,
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
            round: 0,
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
    /// Takes a new status from the rules.
    pub fn apply(&mut self, status: Status) {
        self.round = status.round;
        self.player_unit = status.player_unit;
        self.player_alive = status.player_alive();
        self.health = status.health;
        self.max_health = status.max_health;
        self.casting = status.casting;
        self.cast_start_frame = status.cast_start;
        self.cast_animation_time = status.cast_animation_frames as f32;
        self.villagers_killed = status.villagers_killed;
        self.allies_drained = status.allies_drained;
        self.orks_killed = status.orks_killed;
        self.game_end = status.game_end;
        self.revision = self.revision.wrapping_add(1);
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
    /// A restart was sent and its round hasn't started yet.
    pub restart_requested: bool,
    /// The hovered unit and health percentage the target panel shows.
    pub target: Option<(i32, i32)>,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            bindings: None,
            intro_open: true,
            endgame_open: false,
            restart_requested: false,
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
