use core::cell::RefCell;

use alloc::vec::Vec;

use crate::fire::Field;
use crate::highlight::Resources;

#[derive(Clone, Copy, Debug)]
pub struct DeferredPointLight {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub radius: f32,
}

impl DeferredPointLight {
    pub fn from_params(params: &spring::lights::LightParams) -> Self {
        Self {
            position: [
                params.position.first().copied().unwrap_or(0.0),
                params.position.get(1).copied().unwrap_or(0.0),
                params.position.get(2).copied().unwrap_or(0.0),
            ],
            color: [
                params.diffuse_color.first().copied().unwrap_or(0.0),
                params.diffuse_color.get(1).copied().unwrap_or(0.0),
                params.diffuse_color.get(2).copied().unwrap_or(0.0),
            ],
            radius: params.radius,
        }
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

pub struct UiState {
    pub context: u64,
    pub document: u64,
    pub hud: Option<HudBindings>,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub view_width: i32,
    pub view_height: i32,
    pub camera_initialized: bool,
    pub default_layout_hidden: bool,
    pub bindings_applied: bool,
    pub music_started: bool,
    pub lighting_applied: bool,
    pub intro_open: bool,
    pub endgame_open: bool,
    pub game_end: GameEnd,
    pub player_alive: bool,
    pub player_unit: i32,
    pub tracked_unit: i32,
    pub casting: bool,
    pub villagers_killed: i32,
    pub allies_drained: i32,
    pub orks_killed: i32,
    pub target_unit: i32,
    pub target_ratio: i32,
    pub house_lights_placed: bool,
    pub staff_boost: f32,
    pub house_deferred_lights: Vec<DeferredPointLight>,
    pub staff_deferred_light: Option<DeferredPointLight>,
    pub mouse_deferred_light: Option<DeferredPointLight>,
    pub projectile_deferred_lights: Vec<DeferredPointLight>,
    pub seconds: f32,
    pub game_frame: i32,
    pub cast_start_frame: i32,
    pub cast_animation_time: f32,
    pub fire: Field,
    pub fire_frame: i32,
    pub edge_shader: u32,
    pub edge_list: u32,
    pub edge_setup_failed: bool,
    pub outline: Option<Resources>,
    pub outline_setup_failed: bool,
}

thread_local! {
    static STATE: RefCell<UiState> = RefCell::new(UiState::new());
}

pub fn with_state<R>(function: impl FnOnce(&mut UiState) -> R) -> Option<R> {
    STATE.with(|state| state.try_borrow_mut().ok().map(|mut ui| function(&mut ui)))
}

impl UiState {
    fn new() -> Self {
        Self {
            context: 0,
            document: 0,
            hud: None,
            mouse_x: -1,
            mouse_y: -1,
            view_width: 0,
            view_height: 0,
            camera_initialized: false,
            default_layout_hidden: false,
            bindings_applied: false,
            music_started: false,
            lighting_applied: false,
            intro_open: true,
            endgame_open: false,
            game_end: GameEnd::Start,
            player_alive: true,
            player_unit: -1,
            tracked_unit: -1,
            casting: false,
            villagers_killed: 0,
            allies_drained: 0,
            orks_killed: 0,
            target_unit: -1,
            target_ratio: -2,
            house_lights_placed: false,
            staff_boost: 1.0,
            house_deferred_lights: Vec::new(),
            staff_deferred_light: None,
            mouse_deferred_light: None,
            projectile_deferred_lights: Vec::new(),
            seconds: 0.0,
            game_frame: 0,
            cast_start_frame: 0,
            cast_animation_time: 0.0,
            fire: Field::new(),
            fire_frame: -1,
            edge_shader: 0,
            edge_list: 0,
            edge_setup_failed: false,
            outline: None,
            outline_setup_failed: false,
        }
    }
}

#[derive(Default)]
pub struct Global;
