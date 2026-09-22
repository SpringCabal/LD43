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
    let deferred_lights = lights
        .iter()
        .map(state::DeferredPointLight::from_params)
        .collect();
    let _ = state::with_state(|ui| ui.projectile_deferred_lights = deferred_lights);
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
    let custom_radius =
        spring::weapon_defs::get_weapon_def_custom_param(weapon_def_id, "light_radius")
            .ok()
            .and_then(|value| value.parse::<f32>().ok());
    let radius = custom_radius.unwrap_or_else(|| radius_for(&weapon.info.type_, weapon.info.range));
    if radius <= 0.0 {
        return None;
    }
    let position = spring::projectiles::get_projectile_position(projectile_id).ok()?;
    let color = if custom_radius.is_some() {
        // The blood mage's fireball carries its own warm light in the desktop
        // widget.  Keep that information in the weapon definition so the WASM
        // path does not reduce it to the generic grey cannon light.
        [1.0, 0.25, 0.03]
    } else {
        [COLOR_OFFSET, COLOR_OFFSET, COLOR_OFFSET]
    };
    Some(params::point(
        [position.x, position.y, position.z],
        color,
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
