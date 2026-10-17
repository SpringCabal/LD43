use super::particles::{self, Field};

pub fn draw(spring: &spring::Spring<'_, spring::Ui>, field: &Field) {
    if field.count() == 0 {
        return;
    }
    if !spring
        .gfx()
        .bind_texture(particles::TEXTURE, 0, true)
        .unwrap_or(false)
    {
        return;
    }
    let _ = spring
        .gfx()
        .blend_func(spring::gl::SRC_ALPHA, spring::gl::ONE_MINUS_SRC_ALPHA);
    let _ = spring.gfx().blending(true);
    let _ = spring.gfx().depth_mask(false);
    for particle in field.visible() {
        let fraction = (particle.age - particle.delay) / particle.life;
        let color = particles::color_at(fraction);
        if color[3] <= 0.0 {
            continue;
        }
        let _ = spring.gfx().color(color[0], color[1], color[2], color[3]);
        let _ = spring.gfx().push_matrix();
        let _ = spring.gfx().translate(particle.x, particle.y, particle.z);
        let _ = spring.gfx().billboard();
        let _ = spring.gfx().rotate(particle.rotation, 0.0, 0.0, 1.0);
        let _ = spring
            .gfx()
            .rect(-particle.size, -particle.size, particle.size, particle.size);
        let _ = spring.gfx().pop_matrix();
    }
    let _ = spring.gfx().color(1.0, 1.0, 1.0, 1.0);
    let _ = spring.gfx().depth_mask(true);
    let _ = spring.gfx().blending(false);
    let _ = spring.gfx().bind_texture(particles::TEXTURE, 0, false);
}
