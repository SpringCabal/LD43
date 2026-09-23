use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::PointLight;

/// Replaces `lights` with one light per glowing projectile in flight.
pub fn update(lights: &mut Vec<PointLight>, defs: &mut BTreeMap<i32, Option<DefLight>>) {
    let projectiles =
        spring::projectiles::get_all_projectiles(spring::projectiles::GetAllProjectilesOptions {
            exclude_weapon_projectiles: false,
            exclude_piece_projectiles: true,
        })
        .unwrap_or_default();
    lights.clear();
    lights.extend(
        projectiles
            .into_iter()
            .filter_map(|projectile| light_for(projectile, defs)),
    );
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

/// The radius and colour of the light a weapon def's projectiles carry, or
/// `None` for projectiles without a light.
#[derive(Clone, Copy)]
pub struct DefLight {
    radius: f32,
    color: [f32; 3],
}

fn light_for(projectile_id: i32, defs: &mut BTreeMap<i32, Option<DefLight>>) -> Option<PointLight> {
    let weapon_def_id = spring::projectiles::get_projectile_def_id(projectile_id).ok()?;
    let light = *defs
        .entry(weapon_def_id)
        .or_insert_with(|| def_light(weapon_def_id));
    let light = light?;
    let position = spring::projectiles::get_projectile_position(projectile_id).ok()?;
    Some(PointLight {
        position: [position.x, position.y, position.z],
        color: light.color,
        radius: light.radius,
    })
}

fn def_light(weapon_def_id: i32) -> Option<DefLight> {
    let weapon = spring::weapon_defs::get_weapon_def_by_id(weapon_def_id).ok()?;
    let custom_radius =
        spring::weapon_defs::get_weapon_def_custom_param(weapon_def_id, "light_radius")
            .ok()
            .and_then(|value| value.parse::<f32>().ok());
    let radius = custom_radius.unwrap_or_else(|| radius_for(&weapon.info.type_, weapon.info.range));
    if radius <= 0.0 {
        return None;
    }
    let color = if custom_radius.is_some() {
        // The blood mage's fireball carries its own warm light in the desktop
        // widget.  Keep that information in the weapon definition so the WASM
        // path does not reduce it to the generic grey cannon light.
        [1.0, 0.25, 0.03]
    } else {
        [COLOR_OFFSET, COLOR_OFFSET, COLOR_OFFSET]
    };
    Some(DefLight { radius, color })
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
