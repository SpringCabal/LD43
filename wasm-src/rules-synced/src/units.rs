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
    match unit_control::create_unit(&unit_def, position, facing, team, options) {
        Ok(unit_id) if unit_id >= 0 => Some(unit_id),
        result => {
            spring::log_warning!(
                "ttld-wasm",
                "creating {} at ({:.0}, {:.0}) failed: {result:?}",
                kind.def_name(),
                position.x,
                position.z
            );
            None
        }
    }
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

/// Removes every unit at once, without death sequences, as
/// unit_town_handler.lua's Initialize does.
pub fn destroy_all() {
    for unit_id in all_units() {
        crate::engine::check(
            unit_control::destroy_unit(
                unit_id,
                unit_control::DestroyUnitOptions {
                    selfd: false,
                    reclaimed: true,
                    attacker_id: -1,
                    recycle_id: false,
                },
            ),
            "destroy_unit",
            unit_id,
        );
    }
}

/// Removes a unit without a death sequence.
#[cfg(feature = "headless-integration")]
pub fn destroy(unit_id: i32) {
    crate::engine::check(
        unit_control::destroy_unit(
            unit_id,
            unit_control::DestroyUnitOptions {
                selfd: false,
                reclaimed: true,
                attacker_id: -1,
                recycle_id: false,
            },
        ),
        "destroy_unit",
        unit_id,
    );
}

/// Whether the unit has anything queued.
pub fn has_orders(unit_id: i32) -> bool {
    // Ask for one command: a limit of zero returns an empty queue for every
    // unit, which made every idle check re-issue the same order.
    spring::units_commands::get_command_queue(unit_id, 1)
        .map(|commands| !commands.is_empty())
        .unwrap_or(true)
}

pub fn units_in_cylinder(x: f32, z: f32, radius: f32, allegiance: i32) -> Vec<i32> {
    spring::get_units_in_cylinder(x, z, radius, allegiance).unwrap_or_default()
}

pub fn valid(unit_id: i32) -> bool {
    spring::valid_unit_id(unit_id).unwrap_or(false)
}

pub fn def_of(unit_id: i32) -> Option<i32> {
    spring::units_info::get_unit_def_id(unit_id).ok()
}

pub fn health(unit_id: i32) -> Option<Health> {
    let health = spring::units_info::get_unit_health(unit_id).ok()?;
    Some(Health {
        current: health.health,
        max: health.max_health,
    })
}

pub struct Health {
    pub current: f32,
    pub max: f32,
}

pub fn health_value(health: f32) -> spring::types::UnitHealthValue {
    spring::types::UnitHealthValue {
        health,
        capture: -1.0,
        paralyze: -1.0,
        build: -1.0,
        use_amounts: false,
    }
}
