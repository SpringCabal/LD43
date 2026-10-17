use super::particles::{self, Field, Particle};

const WIND_SCALE: f32 = 0.09;
const DISC_RADIUS_SCALE: f32 = 0.6;
const SIZE_SCALE: f32 = 0.35;
const EMIT_CHANCE_RANGE: i32 = 400;

pub fn on_fire(spring: &spring::Spring<'_, spring::Ui>, unit_id: i32) -> bool {
    spring
        .rules_params()
        .get_unit_rules_param_f32(unit_id, "on_fire")
        .unwrap_or(0.0)
        >= 0.5
}

pub fn emit(spring: &spring::Spring<'_, spring::Ui>, field: &mut Field, burning: &[i32]) {
    if burning.is_empty() {
        return;
    }
    apply_wind(spring, field);
    let count = burning.len() as i32;
    for &unit_id in burning {
        if (spring.unsynced_random().float().expect("unsynced random") * EMIT_CHANCE_RANGE as f32)
            as i32
            >= EMIT_CHANCE_RANGE - count
        {
            continue;
        }
        let Ok(position) = spring.units_info().get_unit_position(
            unit_id,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: false,
                aim_pos: false,
            },
        ) else {
            continue;
        };
        let Ok(radius) = spring.units_info().get_unit_radius(unit_id) else {
            continue;
        };
        for _ in 0..particles::PARTICLES_PER_EMIT {
            spawn(spring, field, position, radius);
        }
    }
}

fn apply_wind(spring: &spring::Spring<'_, spring::Ui>, field: &mut Field) {
    let Ok(wind) = spring.game().get_wind() else {
        return;
    };
    field.set_force(0.0, wind.current * WIND_SCALE + 3.0, 0.0);
}

fn spawn(
    spring: &spring::Spring<'_, spring::Ui>,
    field: &mut Field,
    position: spring::Float3,
    radius: f32,
) {
    let alpha = spring.unsynced_random().float().expect("unsynced random") * core::f32::consts::TAU;
    let disc =
        spring.unsynced_random().float().expect("unsynced random") * DISC_RADIUS_SCALE * radius;
    let emit_angle = spring.unsynced_random().float().expect("unsynced random")
        * particles::EMIT_ROTATION_SPREAD
        * core::f32::consts::PI
        / 180.0;
    let emit_spin =
        spring.unsynced_random().float().expect("unsynced random") * core::f32::consts::TAU;
    let particle = Particle {
        x: position.x + disc * alpha.sin(),
        y: position.y,
        z: position.z + disc * alpha.cos(),
        velocity_x: emit_angle.sin() * emit_spin.cos(),
        velocity_y: emit_angle.cos(),
        velocity_z: emit_angle.sin() * emit_spin.sin(),
        age: 0.0,
        life: particles::LIFE
            + spring.unsynced_random().float().expect("unsynced random") * particles::LIFE_SPREAD,
        delay: spring.unsynced_random().float().expect("unsynced random") * particles::DELAY_SPREAD,
        size: radius * SIZE_SCALE
            + spring.unsynced_random().float().expect("unsynced random") * particles::SIZE_SPREAD,
        rotation: spring.unsynced_random().float().expect("unsynced random")
            * particles::ROTATION_SPREAD,
        rotation_speed: particles::ROTATION_SPEED
            + spring.unsynced_random().float().expect("unsynced random")
                * particles::ROTATION_SPEED_SPREAD,
    };
    field.spawn(particle);
}
