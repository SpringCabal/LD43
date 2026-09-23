use alloc::vec::Vec;

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub age: f32,
    pub life: f32,
    pub delay: f32,
    pub size: f32,
    pub rotation: f32,
    pub rotation_speed: f32,
}

pub struct Field {
    particles: Vec<Particle>,
    force_x: f32,
    force_y: f32,
    force_z: f32,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            particles: Vec::new(),
            force_x: 0.0,
            force_y: FORCE_BASE_Y,
            force_z: 0.0,
        }
    }
}

impl Field {
    pub fn set_force(&mut self, x: f32, y: f32, z: f32) {
        self.force_x = x;
        self.force_y = y;
        self.force_z = z;
    }

    pub fn spawn(&mut self, particle: Particle) {
        if self.particles.len() >= MAX_PARTICLES {
            return;
        }
        self.particles.push(particle);
    }

    pub fn advance(&mut self, delta: f32) {
        let (fx, fy, fz) = (self.force_x, self.force_y, self.force_z);
        for particle in &mut self.particles {
            particle.age += delta;
            if particle.age < particle.delay {
                continue;
            }
            particle.velocity_x += fx * delta;
            particle.velocity_y += fy * delta;
            particle.velocity_z += fz * delta;
            particle.x += particle.velocity_x * delta * SPEED;
            particle.y += particle.velocity_y * delta * SPEED;
            particle.z += particle.velocity_z * delta * SPEED;
            particle.rotation += particle.rotation_speed * delta;
            particle.size += SIZE_GROWTH * delta;
        }
        self.particles
            .retain(|particle| particle.age < particle.delay + particle.life);
    }

    pub fn visible(&self) -> impl Iterator<Item = &Particle> {
        self.particles
            .iter()
            .filter(|particle| particle.age >= particle.delay)
    }

    pub fn count(&self) -> usize {
        self.particles.len()
    }
}

pub const SPEED: f32 = 0.65;
pub const LIFE: f32 = 30.0;
pub const LIFE_SPREAD: f32 = 10.0;
pub const DELAY_SPREAD: f32 = 20.0;
pub const ROTATION_SPEED: f32 = 1.0;
pub const ROTATION_SPEED_SPREAD: f32 = -2.0;
pub const ROTATION_SPREAD: f32 = 360.0;
pub const SIZE_SPREAD: f32 = 1.0;
pub const SIZE_GROWTH: f32 = 0.9;
pub const EMIT_ROTATION_SPREAD: f32 = 60.0;
pub const PARTICLES_PER_EMIT: i32 = 5;
pub const TEXTURE: &str = "bitmaps/GPL/flame.png";

const FORCE_BASE_Y: f32 = 3.0;
const MAX_PARTICLES: usize = 4096;

pub const COLORMAP: [[f32; 4]; 4] = [
    [0.0, 0.0, 0.0, 0.0],
    [0.4, 0.4, 0.4, 0.01],
    [0.35, 0.15, 0.15, 0.20],
    [0.0, 0.0, 0.0, 0.0],
];

pub fn color_at(fraction: f32) -> [f32; 4] {
    let count = COLORMAP.len();
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction >= 1.0 {
        return COLORMAP[count - 1];
    }
    let position = fraction * (count as f32 - 1.0);
    let index = position.floor() as usize;
    let blend = position - index as f32;
    let inverse = 1.0 - blend;
    let low = COLORMAP[index];
    let high = COLORMAP[(index + 1).min(count - 1)];
    [
        low[0] * inverse + high[0] * blend,
        low[1] * inverse + high[1] * blend,
        low[2] * inverse + high[2] * blend,
        low[3] * inverse + high[3] * blend,
    ]
}
