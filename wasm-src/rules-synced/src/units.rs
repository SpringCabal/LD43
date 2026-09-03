use alloc::string::String;
use alloc::vec::Vec;

use spring::generated::owned::unit_control;
use spring::types::Float3;

pub const PLAYER_TEAM: i32 = 0;
pub const ENEMY_TEAM: i32 = 1;
pub const NEUTRAL_TEAM: i32 = 2;

pub const BLOOD_MAGE: &str = "bloodmage";
pub const HOUSE: &str = "house";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VillagerKind {
    Swordsman,
    Crossbowman,
    Coward,
    Peasant,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OrkKind {
    Small,
    Big,
    Boss,
}

impl VillagerKind {
    pub fn def_name(self) -> &'static str {
        match self {
            Self::Swordsman => "swordsman",
            Self::Crossbowman => "crossbowman",
            Self::Coward => "coward",
            Self::Peasant => "peasant",
        }
    }
}

impl OrkKind {
    pub fn def_name(self) -> &'static str {
        match self {
            Self::Small => "orksmall",
            Self::Big => "orkbig",
            Self::Boss => "orkboss",
        }
    }
}

pub fn def_id(name: &str) -> Option<i32> {
    spring::get_unit_def_id_by_name(name).ok()
}

pub fn ground_height(x: f32, z: f32) -> f32 {
    spring::get_ground_height(x, z).unwrap_or(0.0)
}

pub fn create(name: &str, x: f32, z: f32, team: i32, facing: i32) -> Option<i32> {
    let def_id = def_id(name)?;
    let y = ground_height(x, z);
    create_at(name, def_id, Float3 { x, y, z }, team, facing)
}

pub fn create_at(name: &str, def_id: i32, position: Float3, team: i32, facing: i32) -> Option<i32> {
    let unit_def = unit_control::DefRef {
        name: String::from(name),
        id: def_id,
    };
    let options = unit_control::CreateUnitOptions {
        build: false,
        flatten_ground: false,
        unit_id: -1,
        builder_id: -1,
    };
    let unit_id = unit_control::create_unit(&unit_def, position, facing, team, options).ok()?;
    (unit_id >= 0).then_some(unit_id)
}

pub fn position(unit_id: i32) -> Option<Float3> {
    spring::units_info::get_unit_position(
        unit_id,
        spring::units_info::GetUnitPositionOptions {
            mid_pos: false,
            aim_pos: false,
        },
    )
    .ok()
}

pub fn all_units() -> Vec<i32> {
    spring::get_all_units().unwrap_or_default()
}
