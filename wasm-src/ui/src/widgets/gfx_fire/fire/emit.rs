use super::particles::{Effect, Field, Texture};

/// Exactly representable arithmetic in Spring's f32 Lua. Both widgets own it,
/// so native CEG randomness is independent of custom fire polling.
pub struct Random(u32);
impl Default for Random {
    fn default() -> Self {
        Self(1)
    }
}
impl Random {
    pub fn sample(&mut self) -> f32 {
        self.0 = (self.0 * 251 + 1) % 65521;
        self.0 as f32 / 65521.0
    }
}
pub fn on_fire(spring: &spring::Spring<'_, spring::Ui>, unit_id: i32) -> bool {
    spring
        .rules_params()
        .get_unit_rules_param_f32(unit_id, "on_fire")
        .unwrap_or(0.0)
        == 1.0
}
pub fn emit(
    spring: &spring::Spring<'_, spring::Ui>,
    field: &mut Field,
    burning: &[i32],
    random: &mut Random,
) {
    if burning.is_empty() {
        return;
    }
    let wind = spring.game().get_wind_vector().unwrap_or_default();
    for &unit_id in burning {
        // Lua math.random(400) is one based and uses a strict comparison.
        if (random.sample() * 400.0) as i32 + 1 >= 400 - burning.len() as i32 {
            continue;
        }
        let Ok(position) = spring
            .units_info()
            .get_unit_position(unit_id, Default::default())
        else {
            continue;
        };
        let Ok(radius) = spring.units_info().get_unit_radius(unit_id) else {
            continue;
        };
        let effect = effect(radius, [wind.x, wind.y, wind.z]);
        springcabal::particles::emit_effect(
            field,
            &effect,
            [position.x, position.y, position.z],
            &mut || random.sample(),
        );
    }
}
pub(super) fn effect(radius: f32, wind: [f32; 3]) -> Effect {
    Effect {
        count: 5,
        life: 30.0,
        life_spread: 10.0,
        delay_spread: 20.0,
        force: [wind[0] * 0.09, wind[1] * 0.09 + 3.0, wind[2] * 0.09],
        speed: 0.65,
        speed_spread: 0.0,
        speed_exp: 1.0,
        size: radius * 0.35,
        size_spread: 1.0,
        size_growth: 0.9,
        size_exp: 1.0,
        rotation_speed: 1.0,
        rotation_speed_spread: -2.0,
        rotation_spread: 360.0,
        rotation_exp: 1.0,
        distortion: false,
        distortion_strength: 0.0,
        distortion_scale: 0.0,
        distortion_heat: 0.0,
        emit_rotation_spread: 60.0,
        emit_vector: [0.0, 1.0, 0.0],
        start_radius: radius * 0.6,
        colormap: &COLORS,
        texture: Texture::Flame,
    }
}
const COLORS: [[f32; 4]; 4] = [
    [0.0, 0.0, 0.0, 0.0],
    [0.4, 0.4, 0.4, 0.01],
    [0.35, 0.15, 0.15, 0.20],
    [0.0, 0.0, 0.0, 0.0],
];
