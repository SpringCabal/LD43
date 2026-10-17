use alloc::string::String;
use core::cell::{Cell, RefCell};

use spring::gl;

use super::lights::{Lights, PointLight};
use crate::{game, gl_uniforms};

const VERTEX: &str = r#"
void main(void)
{
    gl_TexCoord[0] = gl_MultiTexCoord0;
    gl_Position = gl_Vertex;
}
"#;

const FRAGMENT: &str = include_str!("../../shaders/deferred_lighting.frag");
const BLOOM_BRIGHT_FRAGMENT: &str = include_str!("../../shaders/bloom_bright.frag");
const BLOOM_BLUR_H_FRAGMENT: &str = include_str!("../../shaders/bloom_blur_h.frag");
const BLOOM_BLUR_V_FRAGMENT: &str = include_str!("../../shaders/bloom_blur_v.frag");
const BLOOM_COMBINE_FRAGMENT: &str = include_str!("../../shaders/bloom_combine.frag");
const BLOOM_ILLUMINATION_THRESHOLD: f32 = 0.85;
const BLOOM_MAX_BRIGHTNESS: f32 = 0.35;
const BLOOM_KERNEL_RADIUS: f32 = 32.0;

#[derive(Clone, Copy)]
struct LightResources {
    shader: u32,
    light_position: i32,
    light_color: i32,
    eye_position: i32,
    view_projection_inverse: i32,
}

#[derive(Clone, Copy)]
struct BrightResources {
    shader: u32,
    illumination_threshold: i32,
}

#[derive(Clone, Copy)]
struct BlurResources {
    shader: u32,
    inverse_resolution: i32,
    kernel_radius: i32,
}

#[derive(Clone, Copy)]
struct CombineResources {
    shader: u32,
    max_brightness: i32,
}

struct Resources {
    light: LightResources,
    bright: BrightResources,
    blur_h: BlurResources,
    blur_v: BlurResources,
    combine: CombineResources,
    screen_hdr: String,
    bright_texture: String,
    blur_texture: String,
    width: i32,
    height: i32,
}

#[derive(Default)]
pub struct DeferredPass {
    resources: RefCell<Option<Resources>>,
    failed_size: Cell<Option<(i32, i32)>>,
    reported: Cell<bool>,
}

impl DeferredPass {
    pub fn draw(
        &self,
        spring: &spring::Spring<'_, spring::Ui>,
        lights: &Lights,
        player_unit: i32,
        width: i32,
        height: i32,
    ) {
        if width <= 1 || height <= 1 {
            return;
        }
        if !self.ensure_resources(spring, width, height) {
            return;
        }
        let resources = self.resources.borrow();
        let Some(resources) = resources.as_ref() else {
            return;
        };
        self.report(spring, lights, player_unit);

        if spring
            .gfx()
            .copy_to_texture(&resources.screen_hdr, 0, 0, 0, 0, width, height, 0, 0)
            .is_err()
        {
            return;
        }

        if lights.iter().next().is_some() {
            let Ok(camera) = spring.camera().get_camera_position() else {
                return;
            };
            let Ok(view_projection_inverse) = spring.gfx().get_view_projection_matrix_inverse()
            else {
                return;
            };
            let screen_ratio = height as f32 / width as f32;
            let _ = spring
                .gfx()
                .render_to_texture(&resources.screen_hdr, |spring| {
                    draw_light_pass(
                        spring,
                        &resources.light,
                        lights,
                        &camera,
                        width,
                        height,
                        screen_ratio,
                        &view_projection_inverse,
                    );
                });
        }

        draw_bloom(spring, resources);
    }

    pub fn shutdown(&self, spring: &spring::Spring<'_, spring::Ui>) {
        if let Some(resources) = self.resources.borrow_mut().take() {
            destroy_resources(spring, &resources);
        }
        self.failed_size.set(None);
        self.reported.set(false);
    }

    fn report(&self, spring: &spring::Spring<'_, spring::Ui>, lights: &Lights, player_unit: i32) {
        let frame = game::status_frame(spring);
        if self.reported.get() || frame <= 0 {
            return;
        }
        let (staff, mouse, projectiles) = lights.summary();
        spring::log_info!(
            spring,
            "ttld-ui",
            "deferred light pass active: {} lights at game frame {}, player={}, staff={}, mouse={}, projectiles={}",
            lights.len(),
            frame,
            player_unit,
            staff,
            mouse,
            projectiles
        );
        self.reported.set(true);
    }

    fn ensure_resources(
        &self,
        spring: &spring::Spring<'_, spring::Ui>,
        width: i32,
        height: i32,
    ) -> bool {
        let mut slot = self.resources.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|resources| resources.width == width && resources.height == height)
        {
            return true;
        }
        if self.failed_size.get() == Some((width, height)) {
            return false;
        }
        if let Some(old) = slot.take() {
            destroy_resources(spring, &old);
        }
        *slot = create_resources(spring, width, height);
        if slot.is_none() {
            spring::log_warning!(
                spring,
                "ttld-ui",
                "deferred lighting is off: its shaders or targets could not be made for {width}x{height}"
            );
            self.failed_size.set(Some((width, height)));
        }
        slot.is_some()
    }
}

fn create_resources(
    spring: &spring::Spring<'_, spring::Ui>,
    width: i32,
    height: i32,
) -> Option<Resources> {
    let clip_control = spring
        .gfx()
        .has_extension("GL_ARB_clip_control")
        .unwrap_or(false);
    let definitions = if clip_control {
        "#version 120\n#define BEAM_LIGHT 0\n#define CLIP_CONTROL 1\n"
    } else {
        "#version 120\n#define BEAM_LIGHT 0\n#define CLIP_CONTROL 0\n"
    };
    let light_shader = spring
        .gfx()
        .create_shader(
            definitions,
            VERTEX,
            "",
            "",
            "",
            FRAGMENT,
            "",
            Default::default(),
        )
        .and_then(|shader| {
            spring.gfx().use_shader(shader.shader_id)?;
            spring.gfx().use_shader(0)?;
            Ok(shader)
        })
        .ok()?
        .shader_id;
    let light = LightResources {
        shader: light_shader,
        light_position: gl_uniforms::location(spring, light_shader, "lightpos")?,
        light_color: gl_uniforms::location(spring, light_shader, "lightcolor")?,
        eye_position: gl_uniforms::location(spring, light_shader, "eyePos")?,
        view_projection_inverse: gl_uniforms::location(spring, light_shader, "viewProjectionInv")?,
    };

    let _ = spring.gfx().use_shader(light_shader);
    let _ = gl_uniforms::set_int(spring, light_shader, "modelnormals", 0);
    let _ = gl_uniforms::set_int(spring, light_shader, "modeldepths", 1);
    let _ = gl_uniforms::set_int(spring, light_shader, "mapnormals", 2);
    let _ = gl_uniforms::set_int(spring, light_shader, "mapdepths", 3);
    let _ = gl_uniforms::set_int(spring, light_shader, "modelExtra", 4);
    let _ = spring.gfx().use_shader(0);

    let screen_hdr = create_texture(spring, width, height, gl::RGBA32F_ARB, true)?;
    let bright_texture = create_texture(spring, width / 2, height / 2, gl::RGBA8, true)?;
    let blur_texture = create_texture(spring, width / 2, height / 2, gl::RGBA8, true)?;

    let bright_shader = effect_shader(spring, BLOOM_BRIGHT_FRAGMENT)?;
    let bright = BrightResources {
        shader: bright_shader,
        illumination_threshold: gl_uniforms::location(
            spring,
            bright_shader,
            "illuminationThreshold",
        )?,
    };
    let blur_h_shader = effect_shader(spring, BLOOM_BLUR_H_FRAGMENT)?;
    let blur_h = BlurResources {
        shader: blur_h_shader,
        inverse_resolution: gl_uniforms::location(spring, blur_h_shader, "inverseRX")?,
        kernel_radius: gl_uniforms::location(spring, blur_h_shader, "fragKernelRadius")?,
    };
    let blur_v_shader = effect_shader(spring, BLOOM_BLUR_V_FRAGMENT)?;
    let blur_v = BlurResources {
        shader: blur_v_shader,
        inverse_resolution: gl_uniforms::location(spring, blur_v_shader, "inverseRY")?,
        kernel_radius: gl_uniforms::location(spring, blur_v_shader, "fragKernelRadius")?,
    };
    let combine_shader = effect_shader(spring, BLOOM_COMBINE_FRAGMENT)?;
    let combine = CombineResources {
        shader: combine_shader,
        max_brightness: gl_uniforms::location(spring, combine_shader, "fragMaxBrightness")?,
    };

    let _ = spring.gfx().use_shader(bright_shader);
    let _ = gl_uniforms::set_int(spring, bright_shader, "texture0", 0);
    let _ = spring.gfx().use_shader(blur_h_shader);
    let _ = gl_uniforms::set_int(spring, blur_h_shader, "texture0", 0);
    let _ = spring.gfx().use_shader(blur_v_shader);
    let _ = gl_uniforms::set_int(spring, blur_v_shader, "texture0", 0);
    let _ = spring.gfx().use_shader(combine_shader);
    let _ = gl_uniforms::set_int(spring, combine_shader, "texture0", 0);
    let _ = gl_uniforms::set_int(spring, combine_shader, "texture1", 1);
    let _ = spring.gfx().use_shader(0);

    let resources = Resources {
        light,
        bright,
        blur_h,
        blur_v,
        combine,
        screen_hdr,
        bright_texture,
        blur_texture,
        width,
        height,
    };
    Some(resources)
}

fn create_texture(
    spring: &spring::Spring<'_, spring::Ui>,
    width: i32,
    height: i32,
    format: u32,
    fbo: bool,
) -> Option<alloc::string::String> {
    (width > 0 && height > 0).then(|| {
        spring
            .gfx()
            .create_texture(
                width,
                height,
                0,
                spring::gfx::GfxTextureParams {
                    format,
                    min_filter: gl::LINEAR,
                    mag_filter: gl::LINEAR,
                    wrap_s: gl::CLAMP,
                    wrap_t: gl::CLAMP,
                    fbo,
                    ..Default::default()
                },
            )
            .ok()
    })?
}

fn effect_shader(spring: &spring::Spring<'_, spring::Ui>, fragment: &str) -> Option<u32> {
    let shader = spring
        .gfx()
        .create_shader(
            "#version 120\n",
            "",
            "",
            "",
            "",
            fragment,
            "",
            Default::default(),
        )
        .and_then(|shader| {
            spring.gfx().use_shader(shader.shader_id)?;
            spring.gfx().use_shader(0)?;
            Ok(shader)
        })
        .ok()?
        .shader_id;
    (shader != 0).then_some(shader)
}

fn destroy_resources(spring: &spring::Spring<'_, spring::Ui>, resources: &Resources) {
    for shader in [
        resources.light.shader,
        resources.bright.shader,
        resources.blur_h.shader,
        resources.blur_v.shader,
        resources.combine.shader,
    ] {
        let _ = spring.gfx().delete_shader(shader);
    }
    let _ = spring.gfx().delete_texture_fbo(&resources.screen_hdr);
    let _ = spring.gfx().delete_texture_fbo(&resources.bright_texture);
    let _ = spring.gfx().delete_texture_fbo(&resources.blur_texture);
}

fn bind_gbuffers(spring: &spring::Spring<'_, spring::Ui>) -> bool {
    let names = [
        "$model_gbuffer_normtex",
        "$model_gbuffer_zvaltex",
        "$map_gbuffer_normtex",
        "$map_gbuffer_zvaltex",
        "$model_gbuffer_spectex",
    ];
    names.iter().enumerate().all(|(unit, name)| {
        spring
            .gfx()
            .bind_texture(name, unit as i32, true)
            .unwrap_or(false)
    })
}

fn unbind_gbuffers(spring: &spring::Spring<'_, spring::Ui>) {
    for (unit, name) in [
        "$model_gbuffer_normtex",
        "$model_gbuffer_zvaltex",
        "$map_gbuffer_normtex",
        "$map_gbuffer_zvaltex",
        "$model_gbuffer_spectex",
    ]
    .iter()
    .enumerate()
    {
        let _ = spring.gfx().bind_texture(name, unit as i32, false);
    }
}

fn draw_light_pass(
    spring: &spring::Spring<'_, spring::Ui>,
    resources: &LightResources,
    lights: &Lights,
    camera: &spring::Float3,
    width: i32,
    height: i32,
    screen_ratio: f32,
    view_projection_inverse: &[f32; 16],
) {
    if !bind_gbuffers(spring) {
        return;
    }

    let _ = spring.gfx().blending(true);
    let _ = spring.gfx().blend_func(gl::DST_COLOR, gl::ONE);
    let _ = spring.gfx().use_shader(resources.shader);
    let _ = spring
        .gfx()
        .uniform(resources.eye_position, &[camera.x, camera.y, camera.z], 3);
    let _ = spring.gfx().uniform_matrix(
        resources.view_projection_inverse,
        view_projection_inverse,
        false,
    );

    for light in lights.iter() {
        draw_point_light(
            spring,
            resources,
            light,
            camera,
            width,
            height,
            screen_ratio,
        );
    }

    let _ = spring.gfx().use_shader(0);
    let _ = spring.gfx().blending(false);
    unbind_gbuffers(spring);
}

fn draw_point_light(
    spring: &spring::Spring<'_, spring::Ui>,
    resources: &LightResources,
    light: &PointLight,
    camera: &spring::Float3,
    width: i32,
    height: i32,
    screen_ratio: f32,
) {
    let position = spring::Float3 {
        x: light.position[0],
        y: light.position[1],
        z: light.position[2],
    };
    let Ok(screen_result) = spring.camera().world_to_screen_coords(position) else {
        return;
    };
    let screen = screen_result.screen_pos;
    let sx = (screen.x / width as f32).clamp(0.0, 1.0);
    let sy = screen.y / height as f32;
    let dx = light.position[0] - camera.x;
    let dy = light.position[1] - camera.y;
    let dz = light.position[2] - camera.z;
    let distance = (dx * dx + dy * dy + dz * dz).sqrt().max(0.001);
    let ratio = light.radius / distance * 1.5;

    let _ = spring.gfx().uniform(
        resources.light_position,
        &[
            light.position[0],
            light.position[1],
            light.position[2],
            light.radius,
        ],
        4,
    );
    let _ = spring.gfx().uniform(
        resources.light_color,
        &[light.color[0], light.color[1], light.color[2], 1.0],
        4,
    );
    let _ = spring.gfx().tex_rect(
        ((sx - 0.5) * 2.0 - ratio * screen_ratio).max(-1.0),
        ((sy - 0.5) * 2.0 - ratio).max(-1.0),
        ((sx - 0.5) * 2.0 + ratio * screen_ratio).min(1.0),
        ((sy - 0.5) * 2.0 + ratio).min(1.0),
        (sx - 0.5 * ratio * screen_ratio).max(0.0),
        (sy - 0.5 * ratio).max(0.0),
        (sx + 0.5 * ratio * screen_ratio).min(1.0),
        (sy + 0.5 * ratio).min(1.0),
    );
}

fn draw_bloom(spring: &spring::Spring<'_, spring::Ui>, resources: &Resources) {
    let _ = spring
        .gfx()
        .render_to_texture(&resources.bright_texture, |spring| {
            let _ = spring.gfx().bind_texture(&resources.screen_hdr, 0, true);
            let _ = spring.gfx().use_shader(resources.bright.shader);
            let _ = spring.gfx().uniform(
                resources.bright.illumination_threshold,
                &[BLOOM_ILLUMINATION_THRESHOLD],
                1,
            );
            let _ = spring
                .gfx()
                .tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
            let _ = spring.gfx().use_shader(0);
            let _ = spring.gfx().bind_texture(&resources.screen_hdr, 0, false);
        });

    let half_width = (resources.width / 2).max(1) as f32;
    let half_height = (resources.height / 2).max(1) as f32;
    let _ = spring
        .gfx()
        .render_to_texture(&resources.blur_texture, |spring| {
            let _ = spring
                .gfx()
                .bind_texture(&resources.bright_texture, 0, true);
            let _ = spring.gfx().use_shader(resources.blur_h.shader);
            let _ =
                spring
                    .gfx()
                    .uniform(resources.blur_h.inverse_resolution, &[1.0 / half_width], 1);
            let _ = spring
                .gfx()
                .uniform(resources.blur_h.kernel_radius, &[BLOOM_KERNEL_RADIUS], 1);
            let _ = spring
                .gfx()
                .tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
            let _ = spring.gfx().use_shader(0);
            let _ = spring
                .gfx()
                .bind_texture(&resources.bright_texture, 0, false);
        });

    let _ = spring
        .gfx()
        .render_to_texture(&resources.bright_texture, |spring| {
            let _ = spring.gfx().bind_texture(&resources.blur_texture, 0, true);
            let _ = spring.gfx().use_shader(resources.blur_v.shader);
            let _ =
                spring
                    .gfx()
                    .uniform(resources.blur_v.inverse_resolution, &[1.0 / half_height], 1);
            let _ = spring
                .gfx()
                .uniform(resources.blur_v.kernel_radius, &[BLOOM_KERNEL_RADIUS], 1);
            let _ = spring
                .gfx()
                .tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
            let _ = spring.gfx().use_shader(0);
            let _ = spring.gfx().bind_texture(&resources.blur_texture, 0, false);
        });

    let _ = spring.gfx().use_shader(resources.combine.shader);
    let _ = spring
        .gfx()
        .uniform(resources.combine.max_brightness, &[BLOOM_MAX_BRIGHTNESS], 1);
    let _ = spring.gfx().bind_texture(&resources.screen_hdr, 0, true);
    let _ = spring
        .gfx()
        .bind_texture(&resources.bright_texture, 1, true);
    let _ = spring.gfx().tex_rect(
        0.0,
        0.0,
        resources.width as f32,
        resources.height as f32,
        0.0,
        0.0,
        1.0,
        1.0,
    );
    let _ = spring.gfx().bind_texture(&resources.screen_hdr, 0, false);
    let _ = spring
        .gfx()
        .bind_texture(&resources.bright_texture, 1, false);
    let _ = spring.gfx().use_shader(0);
}
