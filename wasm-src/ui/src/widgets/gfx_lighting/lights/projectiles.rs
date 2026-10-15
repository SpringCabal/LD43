use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::PointLight;

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
const DEFAULT_COLOR: [f32; 3] = [1.2, 0.7, 0.2];
const DEFAULT_SIZE_BASE: f32 = 2.0;
const DEFAULT_SIZE_PER_DAMAGE: f32 = 0.0025;
const DEFAULT_SIZE_PER_AREA: f32 = 0.1;

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
    let custom = |key| spring::weapon_defs::get_weapon_def_custom_param(weapon_def_id, key).ok();
    if custom("fake_weapon").is_some() {
        return None;
    }
    let weapon = spring::weapon_defs::get_weapon_def_by_id(weapon_def_id).ok()?;
    let size = custom("projectile_size")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or_else(|| {
            DEFAULT_SIZE_BASE
                + (weapon.info.damage * DEFAULT_SIZE_PER_DAMAGE)
                    .min(weapon.info.area_of_effect * DEFAULT_SIZE_PER_AREA)
        });
    let radius = custom("light_radius")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or_else(|| radius_for(&weapon.info.type_, weapon.info.range, size));
    if radius <= 0.0 {
        return None;
    }
    let color = custom("light_color")
        .and_then(|rgb| parse_color(&rgb))
        .or_else(|| {
            custom("projectile_rgb")
                .and_then(|rgb| parse_color(&rgb))
                .map(|rgb| rgb.map(|component| component + COLOR_OFFSET))
        })
        .unwrap_or(DEFAULT_COLOR);
    Some(DefLight { radius, color })
}

fn parse_color(value: &str) -> Option<[f32; 3]> {
    let mut components = value.split_whitespace().map(str::parse::<f32>);
    Some([
        components.next()?.ok()?,
        components.next()?.ok()?,
        components.next()?.ok()?,
    ])
}

fn radius_for(weapon_type: &str, range: f32, size: f32) -> f32 {
    match weapon_type {
        "Cannon" => CANNON_BASE_RADIUS + CANNON_SIZE_RADIUS * size,
        "LaserCannon" => LASER_SIZE_RADIUS * size,
        "DGun" => DGUN_RADIUS,
        "MissileLauncher" => MISSILE_SIZE_RADIUS * size,
        "StarburstLauncher" => STARBURST_RADIUS,
        "LightningCannon" => range.min(LIGHTNING_RADIUS_CAP),
        "BeamLaser" => range.min(BEAM_RADIUS_CAP),
        _ => DEFAULT_RADIUS,
    }
}
