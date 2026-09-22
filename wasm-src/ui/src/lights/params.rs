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
        // Non-zero selects multiplicative decay; a rate of 1 keeps a live
        // light steady. Zero selects subtractive decay and kills it on the
        // first engine update.
        decay_function_type: vec![1.0, 1.0, 1.0],
        radius,
        fov: 180.0,
        // TTL=0 is the engine's unused/dead-light marker.
        ttl: u32::MAX,
        priority: 0,
        ignore_los: true,
        local_space: false,
    }
}
