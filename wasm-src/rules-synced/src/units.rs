use alloc::string::String;
use alloc::vec::Vec;

use spring::Float3;
use spring::commands::Pending;
use spring::unit_control;

use crate::defs::{UnitKind, defs};

pub const PLAYER_TEAM: i32 = 0;
pub const ENEMY_TEAM: i32 = 1;
pub const NEUTRAL_TEAM: i32 = 2;

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

pub fn ground_height(spring: &spring::Spring<'_, spring::RulesSynced>, x: f32, z: f32) -> f32 {
    spring.terrain().get_ground_height(x, z).unwrap_or(0.0)
}

pub fn spawn(
    _spring: &spring::Spring<'_, spring::RulesSynced>,
    kind: UnitKind,
    x: f32,
    z: f32,
    team: i32,
    facing: i32,
) -> Pending<spring::RulesSynced, Option<i32>> {
    crate::with_addon_context(|ctx| {
        ctx.commands()
            .queue(move |spring| create(spring, kind, x, z, team, facing))
    })
}

pub fn create(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    kind: UnitKind,
    x: f32,
    z: f32,
    team: i32,
    facing: i32,
) -> Option<i32> {
    let y = ground_height(spring, x, z);
    create_at(spring, kind, Float3 { x, y, z }, team, facing)
}

pub fn create_at(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    kind: UnitKind,
    position: Float3,
    team: i32,
    facing: i32,
) -> Option<i32> {
    let unit_def = unit_control::DefRef {
        name: String::from(kind.def_name()),
        id: defs(spring).id(kind)?,
    };
    let options = unit_control::CreateUnitOptions {
        build: false,
        flatten_ground: false,
        unit_id: -1,
        builder_id: -1,
    };
    match spring
        .unit_control()
        .create_unit(&unit_def, position, facing, team, options)
    {
        Ok(unit_id) if unit_id >= 0 => Some(unit_id),
        result => {
            spring::log_warning!(
                spring,
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

pub fn position(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> Option<Float3> {
    spring
        .units_info()
        .get_unit_position(
            unit_id,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: false,
                aim_pos: false,
            },
        )
        .ok()
}

pub fn all_units(spring: &spring::Spring<'_, spring::RulesSynced>) -> Vec<i32> {
    spring.units().get_all_units().unwrap_or_default()
}

pub fn destroy_all(spring: &spring::Spring<'_, spring::RulesSynced>) {
    for unit_id in all_units(spring) {
        destroy(spring, unit_id);
    }
}

pub fn destroy(_spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) {
    let options = unit_control::DestroyUnitOptions {
        selfd: false,
        reclaimed: true,
        attacker_id: -1,
        recycle_id: false,
    };
    crate::with_addon_context(|ctx| ctx.commands().destroy_unit(unit_id, options))
        .then(move |spring, result| crate::engine::check(spring, result, "destroy_unit", unit_id));
}

pub fn has_orders(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> bool {
    spring
        .units_commands()
        .get_command_queue(unit_id, 1)
        .map(|commands| !commands.is_empty())
        .unwrap_or(true)
}

pub fn units_in_cylinder(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    x: f32,
    z: f32,
    radius: f32,
    allegiance: i32,
) -> Vec<i32> {
    spring
        .units_query()
        .get_units_in_cylinder(x, z, radius, allegiance)
        .unwrap_or_default()
}

pub fn valid(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> bool {
    spring.units_query().valid_unit_id(unit_id).unwrap_or(false)
}

pub fn def_of(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> Option<i32> {
    spring.units_info().get_unit_def_id(unit_id).ok()
}

pub fn health(spring: &spring::Spring<'_, spring::RulesSynced>, unit_id: i32) -> Option<Health> {
    let health = spring.units_info().get_unit_health(unit_id).ok()?;
    Some(Health {
        current: health.health,
        max: health.max_health,
    })
}

pub struct Health {
    pub current: f32,
    pub max: f32,
}

pub fn health_value(health: f32) -> spring::unit_control::UnitHealthValue {
    spring::unit_control::UnitHealthValue {
        health,
        capture: -1.0,
        paralyze: -1.0,
        build: -1.0,
        use_amounts: false,
    }
}
