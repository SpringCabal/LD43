use alloc::vec::Vec;

use crate::state;

use super::params;

pub fn update() {
    let projectiles =
        spring::projectiles::get_all_projectiles(spring::projectiles::GetAllProjectilesOptions {
            exclude_weapon_projectiles: false,
            exclude_piece_projectiles: true,
        })
        .unwrap_or_default();
    let mut lights: Vec<spring::lights::LightParams> = Vec::new();
    for projectile in projectiles {
        let Some(light) = light_for(projectile) else {
            continue;
        };
        lights.push(light);
    }
    apply(lights);
}

const DEFAULT_RADIUS: f32 = 100.0;
const CANNON_BASE_RADIUS: f32 = 10.0;
const CANNON_SIZE_RADIUS: f32 = 90.0;
const LASER_SIZE_RADIUS: f32 = 150.0;
const MISSILE_SIZE_RADIUS: f32 = 150.0;
const STARBURST_RADIUS: f32 = 350.0;
const DGUN_RADIUS: f32 = 800.0;
const LIGHTNING_RADIUS_CAP: f32 = 250.0;
const BEAM_RADIUS_CAP: f32 = 150.0;
const COLOR_OFFSET: f32 = 0.2;
const ASSUMED_SIZE: f32 = 1.0;

fn light_for(projectile_id: i32) -> Option<spring::lights::LightParams> {
    let weapon_def_id = spring::projectiles::get_projectile_def_id(projectile_id).ok()?;
    let weapon = spring::weapon_defs::get_weapon_def_by_id(weapon_def_id).ok()?;
    let radius = radius_for(&weapon.info.type_, weapon.info.range);
    if radius <= 0.0 {
        return None;
    }
    let position = spring::projectiles::get_projectile_position(projectile_id).ok()?;
    Some(params::point(
        [position.x, position.y, position.z],
        [COLOR_OFFSET, COLOR_OFFSET, COLOR_OFFSET],
        radius,
    ))
}

fn radius_for(weapon_type: &str, range: f32) -> f32 {
    match weapon_type {
        "Cannon" => CANNON_BASE_RADIUS + CANNON_SIZE_RADIUS * ASSUMED_SIZE,
        "LaserCannon" => LASER_SIZE_RADIUS * ASSUMED_SIZE,
        "DGun" => DGUN_RADIUS,
        "MissileLauncher" => MISSILE_SIZE_RADIUS * ASSUMED_SIZE,
        "StarburstLauncher" => STARBURST_RADIUS,
        "LightningCannon" => range.min(LIGHTNING_RADIUS_CAP),
        "BeamLaser" => range.min(BEAM_RADIUS_CAP),
        _ => DEFAULT_RADIUS,
    }
}

fn apply(lights: Vec<spring::lights::LightParams>) {
    let handles =
        state::with_state(|ui| core::mem::take(&mut ui.projectile_lights)).unwrap_or_default();
    let mut updated: Vec<u32> = Vec::with_capacity(lights.len());
    let mut existing = handles.into_iter();
    for light in &lights {
        match existing.next() {
            Some(handle) => {
                let _ = spring::lights::update_map_light(handle, light);
                updated.push(handle);
            }
            None => {
                if let Ok(handle) = spring::lights::add_map_light(light) {
                    updated.push(handle);
                }
            }
        }
    }
    let dark = params::point([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.0);
    for handle in existing {
        let _ = spring::lights::update_map_light(handle, &dark);
        updated.push(handle);
    }
    let _ = state::with_state(|ui| ui.projectile_lights = updated);
}
