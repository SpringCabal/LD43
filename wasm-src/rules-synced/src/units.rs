use alloc::string::String;
use alloc::vec::Vec;

use spring::generated::owned::unit_control;
use spring::types::Float3;

use crate::defs::{UnitKind, defs};

pub const PLAYER_TEAM: i32 = 0;
pub const ENEMY_TEAM: i32 = 1;
pub const NEUTRAL_TEAM: i32 = 2;

/// The point the orks attack and flee from.
pub const TOWN_CENTRE_X: f32 = 5650.0;
pub const TOWN_CENTRE_Z: f32 = 5720.0;

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
    pub fn unit(self) -> UnitKind {
        match self {
            Self::Swordsman => UnitKind::Swordsman,
            Self::Crossbowman => UnitKind::Crossbowman,
            Self::Coward => UnitKind::Coward,
            Self::Peasant => UnitKind::Peasant,
        }
    }
}

impl OrkKind {
    pub fn unit(self) -> UnitKind {
        match self {
            Self::Small => UnitKind::OrkSmall,
            Self::Big => UnitKind::OrkBig,
            Self::Boss => UnitKind::OrkBoss,
        }
    }
}

pub fn ground_height(x: f32, z: f32) -> f32 {
    spring::get_ground_height(x, z).unwrap_or(0.0)
}

pub fn create(kind: UnitKind, x: f32, z: f32, team: i32, facing: i32) -> Option<i32> {
    let y = ground_height(x, z);
    create_at(kind, Float3 { x, y, z }, team, facing)
}

pub fn create_at(kind: UnitKind, position: Float3, team: i32, facing: i32) -> Option<i32> {
    let unit_def = unit_control::DefRef {
        name: String::from(kind.def_name()),
        id: defs().id(kind)?,
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

/// Whether the unit has anything queued.
pub fn has_orders(unit_id: i32) -> bool {
    // Ask for one command: a limit of zero returns an empty queue for every
    // unit, which made every idle check re-issue the same order.
    spring::units_commands::get_command_queue(unit_id, 1)
        .map(|commands| !commands.is_empty())
        .unwrap_or(true)
}
