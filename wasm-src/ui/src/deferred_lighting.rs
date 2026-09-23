use alloc::{format, vec::Vec};
use core::cell::RefCell;

use spring::{gfx, gl};

use crate::state::{self, DeferredPointLight};
use crate::{diagnostics, gl_uniforms};

const VERTEX: &str = r#"
void main(void)
{
    gl_TexCoord[0] = gl_MultiTexCoord0;
    gl_Position = gl_Vertex;
}
"#;

const FRAGMENT: &str = include_str!("shaders/deferred_lighting.frag");
const BLOOM_BRIGHT_FRAGMENT: &str = include_str!("shaders/bloom_bright.frag");
const BLOOM_BLUR_H_FRAGMENT: &str = include_str!("shaders/bloom_blur_h.frag");
const BLOOM_BLUR_V_FRAGMENT: &str = include_str!("shaders/bloom_blur_v.frag");
const BLOOM_COMBINE_FRAGMENT: &str = include_str!("shaders/bloom_combine.frag");
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

#[derive(Clone)]
struct Resources {
    light: LightResources,
    bright: BrightResources,
    blur_h: BlurResources,
    blur_v: BlurResources,
    combine: CombineResources,
    screen_hdr: alloc::string::String,
    bright_texture: alloc::string::String,
    blur_texture: alloc::string::String,
    width: i32,
    height: i32,
}

thread_local! {
    static RESOURCES: RefCell<Option<Resources>> = const { RefCell::new(None) };
    static REPORTED: core::cell::Cell<bool> = const { core::cell::Cell::new(false) };
}

pub fn draw(width: i32, height: i32) {
    if width <= 1 || height <= 1 {
        return;
    }

    let Some(resources) = resources(width, height) else {
        return;
    };
    let lights = lights();

    if !REPORTED.get() {
        let (game_frame, player, staff, mouse, projectiles) = state::with_state(|ui| {
            (
                ui.game_frame,
                ui.player_unit,
                ui.staff_deferred_light.is_some(),
                ui.mouse_deferred_light.is_some(),
                ui.projectile_deferred_lights.len(),
            )
        })
        .unwrap_or((0, -1, false, false, 0));
        if game_frame > 0 {
            diagnostics::info(&format!(
                "deferred light pass active: {} lights at game frame {}, player={}, staff={}, mouse={}, projectiles={}",
                lights.len(),
                game_frame,
                player,
                staff,
                mouse,
                projectiles
            ));
            REPORTED.set(true);
        }
    }

    // Lua's deferred widget first copies the rendered frame to an HDR target,
    // composites the lights there, then tone-maps it back to the screen. The
    // HDR target is important: drawing the point lights directly into the
    // 8-bit backbuffer loses the bright red/yellow pools that distinguish the
    // original game.
    if gfx::copy_to_texture(&resources.screen_hdr, 0, 0, 0, 0, width, height, 0, 0).is_err() {
        return;
    }

    if !lights.is_empty() {
        let Ok(camera) = spring::camera::get_camera_position(0) else {
            return;
        };
        let Some(view_projection_inverse) = view_projection_inverse() else {
            return;
        };
        let screen_ratio = height as f32 / width as f32;
        let _ = gfx::render_to_texture(&resources.screen_hdr, || {
            draw_light_pass(
                &resources.light,
                &lights,
                &camera,
                width,
                height,
                screen_ratio,
                &view_projection_inverse,
            );
        });
    }

    draw_bloom(&resources);
}

pub fn shutdown() {
    RESOURCES.with(|slot| {
        if let Some(resources) = slot.borrow_mut().take() {
            destroy_resources(&resources);
        }
    });
    REPORTED.set(false);
}

fn resources(width: i32, height: i32) -> Option<Resources> {
    RESOURCES.with(|slot| {
        if let Some(resources) = slot.borrow().as_ref()
            && resources.width == width
            && resources.height == height
        {
            return Some(resources.clone());
        }

        if let Some(old) = slot.borrow_mut().take() {
            destroy_resources(&old);
        }

        let clip_control = gfx::has_extension("GL_ARB_clip_control").unwrap_or(false);
        let definitions = if clip_control {
            "#version 120\n#define BEAM_LIGHT 0\n#define CLIP_CONTROL 1\n"
        } else {
            "#version 120\n#define BEAM_LIGHT 0\n#define CLIP_CONTROL 0\n"
        };
        let light_shader = gfx::create_shader(gfx::ShaderCreateParams {
            definitions,
            vertex: VERTEX,
            fragment: FRAGMENT,
            ..Default::default()
        })
        .ok()?
        .shader_id;
        let light = LightResources {
            shader: light_shader,
            light_position: gl_uniforms::location(light_shader, "lightpos")?,
            light_color: gl_uniforms::location(light_shader, "lightcolor")?,
            eye_position: gl_uniforms::location(light_shader, "eyePos")?,
            view_projection_inverse: gl_uniforms::location(light_shader, "viewProjectionInv")?,
        };

        let _ = gfx::use_shader(light_shader);
        let _ = gl_uniforms::set_int(light_shader, "modelnormals", 0);
        let _ = gl_uniforms::set_int(light_shader, "modeldepths", 1);
        let _ = gl_uniforms::set_int(light_shader, "mapnormals", 2);
        let _ = gl_uniforms::set_int(light_shader, "mapdepths", 3);
        let _ = gl_uniforms::set_int(light_shader, "modelExtra", 4);
        let _ = gfx::use_shader(0);

        let screen_hdr = create_texture(width, height, gl::RGBA32F_ARB, true)?;
        let bright_texture = create_texture(width / 2, height / 2, gl::RGBA8, true)?;
        let blur_texture = create_texture(width / 2, height / 2, gl::RGBA8, true)?;

        let bright_shader = effect_shader(BLOOM_BRIGHT_FRAGMENT)?;
        let bright = BrightResources {
            shader: bright_shader,
            illumination_threshold: gl_uniforms::location(bright_shader, "illuminationThreshold")?,
        };
        let blur_h_shader = effect_shader(BLOOM_BLUR_H_FRAGMENT)?;
        let blur_h = BlurResources {
            shader: blur_h_shader,
            inverse_resolution: gl_uniforms::location(blur_h_shader, "inverseRX")?,
            kernel_radius: gl_uniforms::location(blur_h_shader, "fragKernelRadius")?,
        };
        let blur_v_shader = effect_shader(BLOOM_BLUR_V_FRAGMENT)?;
        let blur_v = BlurResources {
            shader: blur_v_shader,
            inverse_resolution: gl_uniforms::location(blur_v_shader, "inverseRY")?,
            kernel_radius: gl_uniforms::location(blur_v_shader, "fragKernelRadius")?,
        };
        let combine_shader = effect_shader(BLOOM_COMBINE_FRAGMENT)?;
        let combine = CombineResources {
            shader: combine_shader,
            max_brightness: gl_uniforms::location(combine_shader, "fragMaxBrightness")?,
        };

        let _ = gfx::use_shader(bright_shader);
        let _ = gl_uniforms::set_int(bright_shader, "texture0", 0);
        let _ = gfx::use_shader(blur_h_shader);
        let _ = gl_uniforms::set_int(blur_h_shader, "texture0", 0);
        let _ = gfx::use_shader(blur_v_shader);
        let _ = gl_uniforms::set_int(blur_v_shader, "texture0", 0);
        let _ = gfx::use_shader(combine_shader);
        let _ = gl_uniforms::set_int(combine_shader, "texture0", 0);
        let _ = gl_uniforms::set_int(combine_shader, "texture1", 1);
        let _ = gfx::use_shader(0);

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
        *slot.borrow_mut() = Some(resources);
        slot.borrow().clone()
    })
}

fn create_texture(
    width: i32,
    height: i32,
    format: u32,
    fbo: bool,
) -> Option<alloc::string::String> {
    (width > 0 && height > 0).then(|| {
        spring::create_texture(
            width,
            height,
            0,
            spring::TextureCreateParams {
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

fn effect_shader(fragment: &str) -> Option<u32> {
    let shader = gfx::create_shader(gfx::ShaderCreateParams {
        definitions: "#version 120\n",
        fragment,
        ..Default::default()
    })
    .ok()?
    .shader_id;
    (shader != 0).then_some(shader)
}

fn destroy_resources(resources: &Resources) {
    for shader in [
        resources.light.shader,
        resources.bright.shader,
        resources.blur_h.shader,
        resources.blur_v.shader,
        resources.combine.shader,
    ] {
        let _ = gfx::delete_shader(shader);
    }
    let _ = gfx::delete_texture_fbo(&resources.screen_hdr);
    let _ = gfx::delete_texture_fbo(&resources.bright_texture);
    let _ = gfx::delete_texture_fbo(&resources.blur_texture);
}

fn lights() -> Vec<DeferredPointLight> {
    state::with_state(|ui| {
        let mut lights = ui.house_deferred_lights.clone();
        if let Some(light) = ui.staff_deferred_light {
            lights.push(light);
        }
        if let Some(light) = ui.mouse_deferred_light {
            lights.push(light);
        }
        lights.extend_from_slice(&ui.projectile_deferred_lights);
        lights
    })
    .unwrap_or_default()
}

fn bind_gbuffers() -> bool {
    let names = [
        "$model_gbuffer_normtex",
        "$model_gbuffer_zvaltex",
        "$map_gbuffer_normtex",
        "$map_gbuffer_zvaltex",
        "$model_gbuffer_spectex",
    ];
    names
        .iter()
        .enumerate()
        .all(|(unit, name)| gfx::bind_texture(name, unit as i32, true).unwrap_or(false))
}

fn unbind_gbuffers() {
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
        let _ = gfx::bind_texture(name, unit as i32, false);
    }
}

fn draw_light_pass(
    resources: &LightResources,
    lights: &[DeferredPointLight],
    camera: &spring::types::Float3,
    width: i32,
    height: i32,
    screen_ratio: f32,
    view_projection_inverse: &[f32; 16],
) {
    if !bind_gbuffers() {
        return;
    }

    let _ = gfx::blending(true);
    let _ = gfx::blend_func(gl::DST_COLOR, gl::ONE);
    let _ = gfx::use_shader(resources.shader);
    let _ = gfx::uniform(resources.eye_position, &[camera.x, camera.y, camera.z], 3);
    let _ = gfx::uniform_matrix(
        resources.view_projection_inverse,
        view_projection_inverse,
        false,
    );

    for light in lights {
        draw_point_light(resources, light, camera, width, height, screen_ratio);
    }

    let _ = gfx::use_shader(0);
    let _ = gfx::blending(false);
    unbind_gbuffers();
}

fn draw_point_light(
    resources: &LightResources,
    light: &DeferredPointLight,
    camera: &spring::types::Float3,
    width: i32,
    height: i32,
    screen_ratio: f32,
) {
    let position = spring::types::Float3 {
        x: light.position[0],
        y: light.position[1],
        z: light.position[2],
    };
    let Ok(screen_result) = spring::camera::world_to_screen_coords(position) else {
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

    let _ = gfx::uniform(
        resources.light_position,
        &[
            light.position[0],
            light.position[1],
            light.position[2],
            light.radius,
        ],
        4,
    );
    let _ = gfx::uniform(
        resources.light_color,
        &[light.color[0], light.color[1], light.color[2], 1.0],
        4,
    );
    let _ = gfx::tex_rect(
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

fn draw_bloom(resources: &Resources) {
    // Bright extraction and the two blur passes use the same half-resolution
    // targets and kernel as gfx_deferred_rendering.lua.
    let _ = gfx::render_to_texture(&resources.bright_texture, || {
        let _ = gfx::bind_texture(&resources.screen_hdr, 0, true);
        let _ = gfx::use_shader(resources.bright.shader);
        let _ = gfx::uniform(
            resources.bright.illumination_threshold,
            &[BLOOM_ILLUMINATION_THRESHOLD],
            1,
        );
        let _ = gfx::tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
        let _ = gfx::use_shader(0);
        let _ = gfx::bind_texture(&resources.screen_hdr, 0, false);
    });

    let half_width = (resources.width / 2).max(1) as f32;
    let half_height = (resources.height / 2).max(1) as f32;
    let _ = gfx::render_to_texture(&resources.blur_texture, || {
        let _ = gfx::bind_texture(&resources.bright_texture, 0, true);
        let _ = gfx::use_shader(resources.blur_h.shader);
        let _ = gfx::uniform(resources.blur_h.inverse_resolution, &[1.0 / half_width], 1);
        let _ = gfx::uniform(resources.blur_h.kernel_radius, &[BLOOM_KERNEL_RADIUS], 1);
        let _ = gfx::tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
        let _ = gfx::use_shader(0);
        let _ = gfx::bind_texture(&resources.bright_texture, 0, false);
    });

    let _ = gfx::render_to_texture(&resources.bright_texture, || {
        let _ = gfx::bind_texture(&resources.blur_texture, 0, true);
        let _ = gfx::use_shader(resources.blur_v.shader);
        let _ = gfx::uniform(resources.blur_v.inverse_resolution, &[1.0 / half_height], 1);
        let _ = gfx::uniform(resources.blur_v.kernel_radius, &[BLOOM_KERNEL_RADIUS], 1);
        let _ = gfx::tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
        let _ = gfx::use_shader(0);
        let _ = gfx::bind_texture(&resources.blur_texture, 0, false);
    });

    let _ = gfx::use_shader(resources.combine.shader);
    let _ = gfx::uniform(resources.combine.max_brightness, &[BLOOM_MAX_BRIGHTNESS], 1);
    let _ = gfx::bind_texture(&resources.screen_hdr, 0, true);
    let _ = gfx::bind_texture(&resources.bright_texture, 1, true);
    // Lua's gl.TexRect(0, 0, vsx, vsy, false, true) flips T coordinates.
    let _ = gfx::tex_rect(
        0.0,
        0.0,
        resources.width as f32,
        resources.height as f32,
        0.0,
        0.0,
        1.0,
        1.0,
    );
    let _ = gfx::bind_texture(&resources.screen_hdr, 0, false);
    let _ = gfx::bind_texture(&resources.bright_texture, 1, false);
    let _ = gfx::use_shader(0);
}

fn view_projection_inverse() -> Option<[f32; 16]> {
    let matrix = gfx::get_matrix_data(0x10000).ok()?;
    Some(matrix)
}
