use alloc::string::String;
use spring::gl;

use crate::state;

pub fn draw_pre_unit() {
    let Some(resources) = resources() else {
        draw_outline(false);
        return;
    };
    let (width, height) = view_size();
    if width <= 1 || height <= 1 {
        return;
    }
    let _ = spring::gfx::copy_to_texture(&resources.depth, 0, 0, 0, 0, width, height, 0, 0);
    let _ = spring::gfx::bind_texture(&resources.depth, 0, true);
    apply_resolution(&resources, width, height);
    let _ = spring::gfx::use_shader(resources.depth_shader);
    let _ = spring::gfx::render_to_texture(&resources.offscreen, draw_units_offscreen);
    let _ = spring::gfx::bind_texture(&resources.offscreen, 0, true);
    let _ = spring::gfx::render_to_texture(&resources.blur, || blur(Blur::Vertical));
    let _ = spring::gfx::bind_texture(&resources.blur, 0, true);
    let _ = spring::gfx::render_to_texture(&resources.offscreen, || blur(Blur::Horizontal));
    enter_2d();
    let _ = spring::gfx::bind_texture(&resources.offscreen, 0, true);
    full_screen_quad(width, height);
    leave_2d();
}

pub fn draw_refraction() {
    draw_outline(true);
}

const THICKNESS: f32 = 1.0;
const LINE_WIDTH: f32 = 4.0;
const POLYGON_OFFSET_FACTOR: f32 = 8.0;
const POLYGON_OFFSET_UNITS: f32 = 4.0;
const HIT_UNIT: i32 = 1;

const DEPTH_FRAGMENT: &str = include_str!("shaders/highlight_depth.frag");

const BLUR_HORIZONTAL_FRAGMENT: &str = include_str!("shaders/highlight_blur_h.frag");

const BLUR_VERTICAL_FRAGMENT: &str = include_str!("shaders/highlight_blur_v.frag");

#[derive(Clone)]
pub struct Resources {
    pub depth: String,
    pub offscreen: String,
    pub blur: String,
    pub depth_shader: u32,
    pub blur_horizontal: u32,
    pub blur_vertical: u32,
    pub width: i32,
    pub height: i32,
}

fn resources() -> Option<Resources> {
    let (width, height) = view_size();
    if width <= 1 || height <= 1 {
        return None;
    }
    let existing = state::with_state(|ui| ui.outline.clone()).flatten();
    if let Some(existing) = existing
        && existing.width == width
        && existing.height == height
    {
        return Some(existing);
    }
    if state::with_state(|ui| ui.outline_setup_failed).unwrap_or(true) {
        return None;
    }
    let created = create(width, height);
    if created.is_none() {
        let _ = state::with_state(|ui| ui.outline_setup_failed = true);
    }
    let _ = state::with_state(|ui| ui.outline = created.clone());
    created
}

fn create(width: i32, height: i32) -> Option<Resources> {
    let depth = spring::create_texture(
        width,
        height,
        0,
        spring::TextureCreateParams {
            format: gl::DEPTH_COMPONENT24,
            min_filter: gl::NEAREST,
            mag_filter: gl::NEAREST,
            ..Default::default()
        },
    )
    .ok()?;
    let offscreen = spring::create_texture(
        width,
        height,
        0,
        spring::TextureCreateParams {
            min_filter: gl::LINEAR,
            mag_filter: gl::LINEAR,
            wrap_s: gl::CLAMP,
            wrap_t: gl::CLAMP,
            fbo: true,
            fbo_depth: true,
            ..Default::default()
        },
    )
    .ok()?;
    let blur = spring::create_texture(
        width,
        height,
        0,
        spring::TextureCreateParams {
            min_filter: gl::LINEAR,
            mag_filter: gl::LINEAR,
            wrap_s: gl::CLAMP,
            wrap_t: gl::CLAMP,
            fbo: true,
            ..Default::default()
        },
    )
    .ok()?;
    Some(Resources {
        depth,
        offscreen,
        blur,
        depth_shader: fragment_shader(DEPTH_FRAGMENT)?,
        blur_horizontal: fragment_shader(BLUR_HORIZONTAL_FRAGMENT)?,
        blur_vertical: fragment_shader(BLUR_VERTICAL_FRAGMENT)?,
        width,
        height,
    })
}

fn fragment_shader(source: &str) -> Option<u32> {
    let shader = spring::gfx::create_shader(spring::gfx::ShaderCreateParams {
        fragment: source,
        ..Default::default()
    })
    .ok()?;
    (shader.shader_id != 0).then_some(shader.shader_id)
}

fn apply_resolution(resources: &Resources, width: i32, height: i32) {
    set_uniform_float2(
        resources.depth_shader,
        "screenXY",
        width as f32,
        height as f32,
    );
    set_uniform_int(resources.blur_horizontal, "screenX", width);
    set_uniform_int(resources.blur_vertical, "screenY", height);
    set_uniform_int(resources.depth_shader, "tex0", 0);
    set_uniform_int(resources.blur_horizontal, "tex0", 0);
    set_uniform_int(resources.blur_vertical, "tex0", 0);
    set_uniform_int(resources.depth_shader, "useEqualityTest", 1);
}

fn set_uniform_float2(shader_id: u32, name: &str, first: f32, second: f32) {
    let _ = spring::gfx::use_shader(shader_id);
    if let Ok(location) = spring::gfx::get_uniform_location(shader_id, name) {
        let _ = spring::gfx::uniform(location, &[first, second], 2);
    }
    let _ = spring::gfx::use_shader(0);
}

fn set_uniform_int(shader_id: u32, name: &str, value: i32) {
    let _ = spring::gfx::use_shader(shader_id);
    if let Ok(location) = spring::gfx::get_uniform_location(shader_id, name) {
        let _ = spring::gfx::uniform_int(location, &[value], 1);
    }
    let _ = spring::gfx::use_shader(0);
}

fn draw_units_offscreen() {
    let _ = spring::gfx::clear(gl::COLOR_BUFFER_BIT, &[0.0, 0.0, 0.0, 0.0], 4);
    let _ = spring::gfx::push_matrix(0);
    let _ = spring::gfx::reset_matrices(0);
    let _ = spring::gfx::color(0.0, 0.0, 0.0, THICKNESS);
    if let Some(unit_id) = unit_under_cursor() {
        let _ = spring::gfx::unit(
            unit_id,
            spring::gfx::GfxUnitDrawOptions {
                apply_transform: true,
                do_raw_draw: false,
                no_lua_call: false,
                full_model: true,
                // Lua's gl.Unit omits its LOD argument, which auto-selects the
                // object's Lua-material LOD; Default mirrors that.
                ..Default::default()
            },
        );
    }
    let _ = spring::gfx::color(1.0, 1.0, 1.0, 1.0);
    let _ = spring::gfx::pop_matrix(0);
}

#[derive(Clone, Copy)]
enum Blur {
    Horizontal,
    Vertical,
}

fn blur(direction: Blur) {
    let Some(resources) = state::with_state(|ui| ui.outline.clone()).flatten() else {
        return;
    };
    let shader = match direction {
        Blur::Horizontal => resources.blur_horizontal,
        Blur::Vertical => resources.blur_vertical,
    };
    let _ = spring::gfx::clear(gl::COLOR_BUFFER_BIT, &[0.0, 0.0, 0.0, 0.0], 4);
    let _ = spring::gfx::use_shader(shader);
    full_screen_quad(resources.width, resources.height);
}

fn full_screen_quad(width: i32, height: i32) {
    let half_x = 0.5 / width as f32;
    let half_y = 0.5 / height as f32;
    let _ = spring::gfx::tex_rect(
        -1.0 - half_x,
        1.0 + half_y,
        1.0 + half_x,
        -1.0 - half_y,
        0.0,
        0.0,
        1.0,
        1.0,
    );
}

fn enter_2d() {
    let _ = spring::gfx::use_shader(0);
    let _ = spring::gfx::matrix_mode(gl::PROJECTION);
    let _ = spring::gfx::push_matrix(0);
    let _ = spring::gfx::load_identity(0);
    let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
    let _ = spring::gfx::push_matrix(0);
    let _ = spring::gfx::load_identity(0);
    let _ = spring::gfx::blending(true);
}

fn leave_2d() {
    let _ = spring::gfx::matrix_mode(gl::PROJECTION);
    let _ = spring::gfx::pop_matrix(0);
    let _ = spring::gfx::matrix_mode(gl::MODELVIEW);
    let _ = spring::gfx::pop_matrix(0);
    let _ = spring::gfx::use_shader(0);
    let _ = spring::gfx::blending(false);
}

fn draw_outline(underwater: bool) {
    let Some(unit_id) = unit_under_cursor() else {
        return;
    };
    let _ = spring::gfx::depth_test(spring::gfx::GfxDepthTestOptions::default());
    let _ = spring::gfx::line_width(LINE_WIDTH * THICKNESS);
    if underwater {
        let _ = spring::gfx::polygon_offset(POLYGON_OFFSET_FACTOR, POLYGON_OFFSET_UNITS);
    }
    let _ = spring::gfx::polygon_mode(gl::FRONT_AND_BACK, gl::LINE);
    let _ = spring::gfx::culling(true);
    let _ = spring::gfx::depth_mask(false);
    let _ = spring::gfx::color(0.0, 0.0, 0.0, 1.0);
    let _ = spring::gfx::push_matrix(0);
    let _ = spring::gfx::reset_matrices(0);
    let _ = spring::gfx::unit(
        unit_id,
        spring::gfx::GfxUnitDrawOptions {
            apply_transform: true,
            do_raw_draw: false,
            no_lua_call: false,
            full_model: true,
            // Lua's gl.Unit omits its LOD argument, which auto-selects the
            // object's Lua-material LOD; Default mirrors that.
            ..Default::default()
        },
    );
    let _ = spring::gfx::pop_matrix(0);
    let _ = spring::gfx::line_width(1.0);
    let _ = spring::gfx::color(1.0, 1.0, 1.0, 1.0);
    let _ = spring::gfx::culling(false);
    let _ = spring::gfx::polygon_mode(gl::FRONT_AND_BACK, gl::FILL);
    if underwater {
        let _ = spring::gfx::polygon_offset(0.0, 0.0);
    }
}

fn view_size() -> (i32, i32) {
    state::with_state(|ui| (ui.view_width, ui.view_height)).unwrap_or((0, 0))
}

fn unit_under_cursor() -> Option<i32> {
    let mouse = spring::input::get_mouse_state(0).ok()?;
    if mouse.offscreen {
        return None;
    }
    let trace = spring::camera::trace_screen_ray(
        mouse.x,
        mouse.y,
        spring::camera::TraceScreenRayOptions {
            only_coords: false,
            use_minimap: false,
            include_sky: false,
            ignore_water: false,
            height_offset: 0.0,
        },
    )
    .ok()?;
    (trace.hit_type == HIT_UNIT).then_some(trace.hit_id)
}
