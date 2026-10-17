use core::cell::Cell;

use spring::Float3;
use spring_addons::Resource;
pub use ttld_shared::GameEnd;
use ttld_shared::Status;

#[derive(Default)]
pub struct Global {
    pub status: Resource<GameStatus>,
    pub hud: Resource<Hud>,
    pub view: Cell<(i32, i32)>,
    pub cursor: Cell<Cursor>,
}

#[derive(Clone, Copy)]
pub struct GameStatus {
    pub revision: u32,
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
    pub bindings: Option<HudBindings>,
    pub intro_open: bool,
    pub endgame_open: bool,
    pub restart_requested: bool,
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
    pub fn modal(&self) -> bool {
        self.intro_open || self.endgame_open
    }
}

#[derive(Clone, Copy, Default)]
pub struct Cursor {
    pub screen: Option<(f32, f32)>,
    pub ground: Option<Float3>,
    pub unit: Option<i32>,
}
