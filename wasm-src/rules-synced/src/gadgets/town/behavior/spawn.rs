use alloc::rc::Rc;
use core::cell::RefCell;

use spring::ResultExt;
use spring::commands;
use spring::generated::owned::unit_control;
use spring::types::Float3;

use super::villagers::{Villager, Villagers};
use crate::defs::{UnitKind, defs};
use crate::random;
use crate::units::{self, PLAYER_TEAM, VillagerKind};

use super::data::{HOUSE_POSITIONS, VILLAGER_AREAS};

pub struct VillagerArea {
    pub x: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
    pub units: &'static [(VillagerKind, i32)],
}

const BLOOD_MAGE_X: f32 = 5580.0;
const BLOOD_MAGE_Z: f32 = 7100.0;
const HOUSE_GRID: f32 = 16.0;
const PLACEMENT_TRIES: i32 = 8;
const BLOCKED: i32 = 0;
const FREE: i32 = 2;

pub fn populate(villagers: Rc<RefCell<Villagers>>) {
    let list = Rc::clone(&villagers);
    commands::run(move || *list.borrow_mut() = Villagers::default());
    spawn_houses();
    spawn_blood_mage();
    fill_villager_areas(&villagers);
    commands::run(move || {
        spring::log_info!(
            "ttld-wasm",
            "town populated: {} houses, {} villagers",
            HOUSE_POSITIONS.len(),
            villagers.borrow().count()
        );
    });
}

fn spawn_houses() {
    for (x, z) in HOUSE_POSITIONS {
        let x = (x / HOUSE_GRID).floor() * HOUSE_GRID;
        let z = (z / HOUSE_GRID).floor() * HOUSE_GRID;
        commands::run(move || {
            units::create(UnitKind::House, x, z, PLAYER_TEAM, random::facing());
        });
    }
}

fn spawn_blood_mage() {
    units::spawn(
        UnitKind::BloodMage,
        BLOOD_MAGE_X,
        BLOOD_MAGE_Z,
        PLAYER_TEAM,
        0,
    )
    .then(|unit| {
        if unit.is_none() {
            spring::log_warning!("ttld-wasm", "blood mage failed to spawn");
        }
    });
}

fn fill_villager_areas(villagers: &Rc<RefCell<Villagers>>) {
    for area in &VILLAGER_AREAS {
        for (kind, count) in area.units {
            let kind = kind.unit();
            let Some(def_id) = defs().id(kind) else {
                spring::log_warning!("ttld-wasm", "unknown villager def {}", kind.def_name());
                continue;
            };
            for _ in 0..*count {
                let villagers = Rc::clone(villagers);
                commands::queue(move || spawn_villager(area, kind, def_id)).then(move |unit_id| {
                    if let Some(unit_id) = unit_id {
                        villagers.borrow_mut().push(Villager { unit_id, area });
                    }
                });
            }
        }
    }
}

fn spawn_villager(area: &'static VillagerArea, kind: UnitKind, def_id: i32) -> Option<i32> {
    let facing = random::facing();
    let position = find_placement(area, def_id, facing);
    let unit_id = units::create_at(kind, position, PLAYER_TEAM, facing)?;
    unit_control::set_unit_rotation(
        unit_id,
        Float3 {
            x: 0.0,
            y: random::range(0.0, core::f32::consts::PI),
            z: 0.0,
        },
    )
    .log_err_once("villager rotation");
    Some(unit_id)
}

fn find_placement(area: &VillagerArea, def_id: i32, facing: i32) -> Float3 {
    let mut position = sample(area);
    for tries in 1..=PLACEMENT_TRIES {
        let blocking = test_build_order(def_id, position, facing);
        let acceptable = if tries < 4 {
            blocking == FREE
        } else {
            blocking != BLOCKED
        };
        if acceptable {
            break;
        }
        position = sample(area);
    }
    position
}

fn sample(area: &VillagerArea) -> Float3 {
    let x = area.x + random::centered(area.width);
    let z = area.z + random::centered(area.height);
    Float3 {
        x,
        y: units::ground_height(x, z),
        z,
    }
}

fn test_build_order(def_id: i32, position: Float3, facing: i32) -> i32 {
    spring::utils::test_build_order(def_id, position, facing)
        .map(|result| result.status)
        .unwrap_or(BLOCKED)
}
