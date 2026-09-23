use alloc::string::String;
use core::cell::{Cell, RefCell};

use spring::gl;

use crate::gl_uniforms;
use crate::state::Global;

/// Outlines the unit under the cursor, like the Lua build's gfx_highlight.lua.
#[derive(Default)]
pub struct Highlight {
    resources: RefCell<Option<Resources>>,
    setup_failed: Cell<bool>,
}

impl Highlight {
    pub fn draw_pre_unit(&self, global: &Global) {
        let unit = global.cursor.get().unit;
        let (width, height) = global.view.get();
        if width <= 1 || height <= 1 || !self.ensure_resources(width, height) {
            draw_outline(unit, false);
            return;
        }
        let resources = self.resources.borrow();
        let Some(resources) = resources.as_ref() else {
            return;
        };
        let _ = spring::gfx::push_attrib(gl::ALL_ATTRIB_BITS);
        let _ = spring::gfx::copy_to_texture(&resources.depth, 0, 0, 0, 0, width, height, 0, 0);
        let _ = spring::gfx::bind_texture(&resources.depth, 0, true);
        apply_resolution(resources, width, height);
        let _ = spring::gfx::use_shader(resources.depth_shader);
        let _ = spring::gfx::render_to_texture(&resources.offscreen, || draw_unit_offscreen(unit));
        let _ = spring::gfx::bind_texture(&resources.offscreen, 0, true);
        let _ = spring::gfx::render_to_texture(&resources.blur, || blur(resources, Blur::Vertical));
        let _ = spring::gfx::bind_texture(&resources.blur, 0, true);
        let _ = spring::gfx::render_to_texture(&resources.offscreen, || {
            blur(resources, Blur::Horizontal)
        });
        enter_2d();
        let _ = spring::gfx::bind_texture(&resources.offscreen, 0, true);
        full_screen_quad(width, height);
        let _ = spring::gfx::bind_texture(&resources.offscreen, 0, false);
        leave_2d();
        let _ = spring::gfx::pop_attrib(0);
    }

    pub fn draw_refraction(&self, global: &Global) {
        draw_outline(global.cursor.get().unit, true);
    }

    /// Creates the textures and shaders for this view size, replacing any
    /// made for another size; false if they cannot be made.
    fn ensure_resources(&self, width: i32, height: i32) -> bool {
        let mut slot = self.resources.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|resources| resources.width == width && resources.height == height)
        {
            return true;
        }
        if self.setup_failed.get() {
            return false;
        }
        if let Some(old) = slot.take() {
            destroy(&old);
        }
        *slot = create(width, height);
        self.setup_failed.set(slot.is_none());
        slot.is_some()
    }
}

const THICKNESS: f32 = 1.0;
const LINE_WIDTH: f32 = 4.0;
const POLYGON_OFFSET_FACTOR: f32 = 8.0;
const POLYGON_OFFSET_UNITS: f32 = 4.0;

const DEPTH_FRAGMENT: &str = include_str!("shaders/highlight_depth.frag");

const BLUR_HORIZONTAL_FRAGMENT: &str = include_str!("shaders/highlight_blur_h.frag");

const BLUR_VERTICAL_FRAGMENT: &str = include_str!("shaders/highlight_blur_v.frag");

struct Resources {
    depth: String,
    offscreen: String,
    blur: String,
    depth_shader: u32,
    blur_horizontal: u32,
    blur_vertical: u32,
    width: i32,
    height: i32,
}

fn destroy(resources: &Resources) {
    for shader in [
        resources.depth_shader,
        resources.blur_horizontal,
        resources.blur_vertical,
    ] {
        let _ = spring::gfx::delete_shader(shader);
    }
    let _ = spring::gfx::delete_texture(&resources.depth);
    let _ = spring::gfx::delete_texture_fbo(&resources.offscreen);
    let _ = spring::gfx::delete_texture_fbo(&resources.blur);
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
    if let Some(location) = gl_uniforms::location(shader_id, name) {
        let _ = spring::gfx::uniform(location, &[first, second], 2);
    }
    let _ = spring::gfx::use_shader(0);
}

fn set_uniform_int(shader_id: u32, name: &str, value: i32) {
    let _ = spring::gfx::use_shader(shader_id);
    gl_uniforms::set_int(shader_id, name, value);
    let _ = spring::gfx::use_shader(0);
}

fn draw_unit_offscreen(unit: Option<i32>) {
    let _ = spring::gfx::clear(gl::COLOR_BUFFER_BIT, &[0.0, 0.0, 0.0, 0.0], 4);
    let _ = spring::gfx::push_matrix(0);
    let _ = spring::gfx::reset_matrices(0);
    let _ = spring::gfx::color(0.0, 0.0, 0.0, THICKNESS);
    if let Some(unit_id) = unit {
        let _ = spring::gfx::unit(
            unit_id,
            spring::gfx::GfxUnitDrawOptions {
                apply_transform: true,
                // gfx.Unit's `doRawDraw` argument is the second argument of
                // Lua's gl.Unit.  The original highlight widget calls
                // gl.Unit(unitID, true), so use the raw engine model path.
                do_raw_draw: true,
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

fn blur(resources: &Resources, direction: Blur) {
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
        1.0,
        1.0,
        0.0,
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
    // Deferred lighting leaves the engine's multiplicative light blend
    // function installed.  Lua's gl.Blending("alpha") resets it before the
    // outline quad is composited.
    let _ = spring::gfx::blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
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

fn draw_outline(unit: Option<i32>, underwater: bool) {
    let Some(unit_id) = unit else {
        return;
    };

    // This is the same state as the original Lua fallback: draw the model's
    // back-face wireframe in front of the regular model.  Set every relevant
    // state explicitly because this callback runs after other world widgets.
    let _ = spring::gfx::push_attrib(gl::ALL_ATTRIB_BITS);
    let _ = spring::gfx::depth_test(spring::gfx::GfxDepthTestOptions {
        enable: true,
        set_func: true,
        func: gl::LESS,
    });
    let _ = spring::gfx::line_width(LINE_WIDTH * THICKNESS);
    if underwater {
        let _ = spring::gfx::polygon_offset(POLYGON_OFFSET_FACTOR, POLYGON_OFFSET_UNITS);
    }
    let _ = spring::gfx::polygon_mode(gl::FRONT_AND_BACK, gl::LINE);
    let _ = spring::gfx::culling(true);
    let _ = spring::gfx::cull_face(spring::gfx::GfxCullFace::GfxCullFaceFront);
    let _ = spring::gfx::depth_mask(false);
    let _ = spring::gfx::color(0.0, 0.0, 0.0, 1.0);
    let _ = spring::gfx::push_matrix(0);
    let _ = spring::gfx::reset_matrices(0);
    let _ = spring::gfx::unit(
        unit_id,
        spring::gfx::GfxUnitDrawOptions {
            apply_transform: true,
            // Match gl.Unit(unitID, true) in gfx_highlight.lua.
            do_raw_draw: true,
            no_lua_call: false,
            full_model: true,
            // Lua's gl.Unit omits its LOD argument, which auto-selects the
            // object's Lua-material LOD; Default mirrors that.
            ..Default::default()
        },
    );
    let _ = spring::gfx::pop_matrix(0);
    let _ = spring::gfx::pop_attrib(0);
}
