use super::particles::{Batch, Field, Texture};
use spring::gl;
pub fn draw(spring: &spring::Spring<'_, spring::Ui>, field: &Field, batch: &Batch) {
    if field.gpu_particles(Some(Texture::Flame)).is_empty() {
        return;
    }
    let _ = spring.gfx().push_attrib(gl::ALL_ATTRIB_BITS);
    let _ = spring.gfx().bind_texture(Texture::Flame.path(), 0, true);
    let _ = spring.gfx().blending(true);
    let _ = spring.gfx().blend_func(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
    let _ = spring.gfx().depth_test(spring::gfx::GfxDepthTestOptions {
        enable: true,
        set_func: true,
        func: gl::LEQUAL,
    });
    let _ = spring.gfx().depth_mask(false);
    let _ = spring.gfx().alpha_test(true, gl::GREATER, 0.0);
    if !batch
        .gpu
        .draw(spring, field, Some(Texture::Flame), 0.0, [0.0; 2])
        && let Ok(camera) = spring.unit_rendering().get_camera_vectors()
    {
        let mut vertices = batch.vertices.borrow_mut();
        vertices.clear();
        for particle in field.visible(Texture::Flame) {
            if particle.color[3] > 0.0 {
                springcabal::particles::geometry::quad(
                    &mut vertices,
                    &particle,
                    camera.up,
                    camera.right,
                );
            }
        }
        batch.draw(spring, &vertices);
    }
    let _ = spring.gfx().bind_texture(Texture::Flame.path(), 0, false);
    let _ = spring.gfx().pop_attrib();
}
