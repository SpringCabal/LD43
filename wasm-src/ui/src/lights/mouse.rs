use spring::types::Float3;

use super::PointLight;

/// The light that follows the cursor over the ground.
pub fn light(ground: Float3) -> PointLight {
    PointLight {
        position: [ground.x, ground.y + HEIGHT, ground.z],
        color: [RED, GREEN, BLUE],
        radius: RADIUS,
    }
}

const HEIGHT: f32 = 100.0;
const RADIUS: f32 = 5000.0;
const RED: f32 = 0.9;
const GREEN: f32 = 0.9;
const BLUE: f32 = 0.9;
