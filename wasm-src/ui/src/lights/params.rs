use alloc::vec;

use spring::lights::LightParams;

pub fn point(position: [f32; 3], color: [f32; 3], radius: f32) -> LightParams {
    LightParams {
        position: position.to_vec(),
        direction: vec![0.0, -1.0, 0.0],
        ambient_color: vec![0.0, 0.0, 0.0],
        diffuse_color: color.to_vec(),
        specular_color: color.to_vec(),
        intensity_weight: vec![1.0, 1.0, 1.0],
        attenuation: vec![0.0, 0.0, 1.0],
        ambient_decay_rate: vec![1.0, 1.0, 1.0],
        diffuse_decay_rate: vec![1.0, 1.0, 1.0],
        specular_decay_rate: vec![1.0, 1.0, 1.0],
        decay_function_type: vec![0.0, 0.0, 0.0],
        radius,
        fov: 180.0,
        ttl: 0,
        priority: 0,
        ignore_los: true,
        local_space: false,
    }
}
