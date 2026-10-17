use alloc::string::String;
use core::cell::{Cell, RefCell};

use spring::gl;

use crate::gl_uniforms;
use crate::state::Global;

#[derive(Default)]
pub struct Highlight {
    resources: RefCell<Option<Resources>>,
    setup_failed: Cell<bool>,
}

impl Highlight {
    pub fn draw_pre_unit(&self, spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
        let Some(unit) = global.cursor.get().unit else {
            return;
        };
        let (width, height) = global.view.get();
        if width <= 1 || height <= 1 || !self.ensure_resources(spring, width, height) {
            draw_outline(spring, Some(unit), false);
            return;
        }
        let resources = self.resources.borrow();
        let Some(resources) = resources.as_ref() else {
            return;
        };
        let _ = spring.gfx().push_attrib(gl::ALL_ATTRIB_BITS);
        let _ = spring
            .gfx()
            .copy_to_texture(&resources.depth, 0, 0, 0, 0, width, height, 0, 0);
        let _ = spring.gfx().bind_texture(&resources.depth, 0, true);
        let _ = spring.gfx().use_shader(resources.depth_shader);
        let _ = spring
            .gfx()
            .render_to_texture(&resources.offscreen, |spring| {
                draw_unit_offscreen(spring, unit)
            });
        let _ = spring.gfx().bind_texture(&resources.offscreen, 0, true);
        let _ = spring.gfx().render_to_texture(&resources.blur, |spring| {
            blur(spring, resources, Blur::Vertical)
        });
        let _ = spring.gfx().bind_texture(&resources.blur, 0, true);
        let _ = spring
            .gfx()
            .render_to_texture(&resources.offscreen, |spring| {
                blur(spring, resources, Blur::Horizontal)
            });
        enter_2d(spring);
        let _ = spring.gfx().bind_texture(&resources.offscreen, 0, true);
        full_screen_quad(spring, width, height);
        let _ = spring.gfx().bind_texture(&resources.offscreen, 0, false);
        leave_2d(spring);
        let _ = spring.gfx().pop_attrib();
    }

    pub fn draw_refraction(&self, spring: &spring::Spring<'_, spring::Ui>, global: &Global) {
        draw_outline(spring, global.cursor.get().unit, true);
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
        if self.setup_failed.get() {
            return false;
        }
        if let Some(old) = slot.take() {
            destroy(spring, &old);
        }
        *slot = create(spring, width, height);
        if let Some(resources) = slot.as_ref() {
            apply_resolution(spring, resources, width, height);
        }
        self.setup_failed.set(slot.is_none());
        slot.is_some()
    }
}

const THICKNESS: f32 = 1.0;
const LINE_WIDTH: f32 = 4.0;
const POLYGON_OFFSET_FACTOR: f32 = 8.0;
const POLYGON_OFFSET_UNITS: f32 = 4.0;

const DEPTH_FRAGMENT: &str = include_str!("../../shaders/highlight_depth.frag");

const BLUR_HORIZONTAL_FRAGMENT: &str = include_str!("../../shaders/highlight_blur_h.frag");

const BLUR_VERTICAL_FRAGMENT: &str = include_str!("../../shaders/highlight_blur_v.frag");

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

fn destroy(spring: &spring::Spring<'_, spring::Ui>, resources: &Resources) {
    for shader in [
        resources.depth_shader,
        resources.blur_horizontal,
        resources.blur_vertical,
    ] {
        let _ = spring.gfx().delete_shader(shader);
    }
    let _ = spring.gfx().delete_texture(&resources.depth);
    let _ = spring.gfx().delete_texture_fbo(&resources.offscreen);
    let _ = spring.gfx().delete_texture_fbo(&resources.blur);
}

fn create(spring: &spring::Spring<'_, spring::Ui>, width: i32, height: i32) -> Option<Resources> {
    let depth = spring
        .gfx()
        .create_texture(
            width,
            height,
            0,
            spring::gfx::GfxTextureParams {
                format: gl::DEPTH_COMPONENT24,
                min_filter: gl::NEAREST,
                mag_filter: gl::NEAREST,
                ..Default::default()
            },
        )
        .ok()?;
    let offscreen = spring
        .gfx()
        .create_texture(
            width,
            height,
            0,
            spring::gfx::GfxTextureParams {
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
    let blur = spring
        .gfx()
        .create_texture(
            width,
            height,
            0,
            spring::gfx::GfxTextureParams {
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
        depth_shader: fragment_shader(spring, DEPTH_FRAGMENT)?,
        blur_horizontal: fragment_shader(spring, BLUR_HORIZONTAL_FRAGMENT)?,
        blur_vertical: fragment_shader(spring, BLUR_VERTICAL_FRAGMENT)?,
        width,
        height,
    })
}

fn fragment_shader(spring: &spring::Spring<'_, spring::Ui>, source: &str) -> Option<u32> {
    let shader = spring
        .gfx()
        .create_shader("", "", "", "", "", source, "", Default::default())
        .and_then(|shader| {
            spring.gfx().use_shader(shader.shader_id)?;
            spring.gfx().use_shader(0)?;
            Ok(shader)
        })
        .ok()?;
    (shader.shader_id != 0).then_some(shader.shader_id)
}

fn apply_resolution(
    spring: &spring::Spring<'_, spring::Ui>,
    resources: &Resources,
    width: i32,
    height: i32,
) {
    set_uniform_float2(
        spring,
        resources.depth_shader,
        "screenXY",
        width as f32,
        height as f32,
    );
    set_uniform_int(spring, resources.blur_horizontal, "screenX", width);
    set_uniform_int(spring, resources.blur_vertical, "screenY", height);
    set_uniform_int(spring, resources.depth_shader, "tex0", 0);
    set_uniform_int(spring, resources.blur_horizontal, "tex0", 0);
    set_uniform_int(spring, resources.blur_vertical, "tex0", 0);
    set_uniform_int(spring, resources.depth_shader, "useEqualityTest", 1);
}

fn set_uniform_float2(
    spring: &spring::Spring<'_, spring::Ui>,
    shader_id: u32,
    name: &str,
    first: f32,
    second: f32,
) {
    let _ = spring.gfx().use_shader(shader_id);
    if let Some(location) = gl_uniforms::location(spring, shader_id, name) {
        let _ = spring.gfx().uniform(location, &[first, second], 2);
    }
    let _ = spring.gfx().use_shader(0);
}

fn set_uniform_int(
    spring: &spring::Spring<'_, spring::Ui>,
    shader_id: u32,
    name: &str,
    value: i32,
) {
    let _ = spring.gfx().use_shader(shader_id);
    gl_uniforms::set_int(spring, shader_id, name, value);
    let _ = spring.gfx().use_shader(0);
}

fn draw_unit_offscreen(spring: &spring::Spring<'_, spring::Ui>, unit_id: i32) {
    let _ = spring
        .gfx()
        .clear(gl::COLOR_BUFFER_BIT, &[0.0, 0.0, 0.0, 0.0], 4);
    let _ = spring.gfx().push_matrix();
    let _ = spring.gfx().reset_matrices();
    let _ = spring.gfx().color(0.0, 0.0, 0.0, THICKNESS);
    let _ = spring.gfx().unit(
        unit_id,
        spring::gfx::GfxUnitDrawOptions {
            apply_transform: true,
            do_raw_draw: true,
            no_lua_call: false,
            full_model: true,
            ..Default::default()
        },
    );
    let _ = spring.gfx().color(1.0, 1.0, 1.0, 1.0);
    let _ = spring.gfx().pop_matrix();
}

#[derive(Clone, Copy)]
enum Blur {
    Horizontal,
    Vertical,
}

fn blur(spring: &spring::Spring<'_, spring::Ui>, resources: &Resources, direction: Blur) {
    let shader = match direction {
        Blur::Horizontal => resources.blur_horizontal,
        Blur::Vertical => resources.blur_vertical,
    };
    let _ = spring
        .gfx()
        .clear(gl::COLOR_BUFFER_BIT, &[0.0, 0.0, 0.0, 0.0], 4);
    let _ = spring.gfx().use_shader(shader);
    full_screen_quad(spring, resources.width, resources.height);
}

fn full_screen_quad(spring: &spring::Spring<'_, spring::Ui>, width: i32, height: i32) {
    let half_x = 0.5 / width as f32;
    let half_y = 0.5 / height as f32;
    let _ = spring.gfx().tex_rect(
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

fn enter_2d(spring: &spring::Spring<'_, spring::Ui>) {
    let _ = spring.gfx().use_shader(0);
    let _ = spring.gfx().matrix_mode(gl::PROJECTION);
    let _ = spring.gfx().push_matrix();
    let _ = spring.gfx().load_identity();
    let _ = spring.gfx().matrix_mode(gl::MODELVIEW);
    let _ = spring.gfx().push_matrix();
    let _ = spring.gfx().load_identity();
    let _ = spring
        .gfx()
        .blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    let _ = spring.gfx().blending(true);
}

fn leave_2d(spring: &spring::Spring<'_, spring::Ui>) {
    let _ = spring.gfx().matrix_mode(gl::PROJECTION);
    let _ = spring.gfx().pop_matrix();
    let _ = spring.gfx().matrix_mode(gl::MODELVIEW);
    let _ = spring.gfx().pop_matrix();
    let _ = spring.gfx().use_shader(0);
    let _ = spring.gfx().blending(false);
}

fn draw_outline(spring: &spring::Spring<'_, spring::Ui>, unit: Option<i32>, underwater: bool) {
    let Some(unit_id) = unit else {
        return;
    };

    let _ = spring.gfx().push_attrib(gl::ALL_ATTRIB_BITS);
    let _ = spring.gfx().depth_test(spring::gfx::GfxDepthTestOptions {
        enable: true,
        set_func: true,
        func: gl::LESS,
    });
    let _ = spring.gfx().line_width(LINE_WIDTH * THICKNESS);
    if underwater {
        let _ = spring
            .gfx()
            .polygon_offset(POLYGON_OFFSET_FACTOR, POLYGON_OFFSET_UNITS);
    }
    let _ = spring.gfx().polygon_mode(gl::FRONT_AND_BACK, gl::LINE);
    let _ = spring.gfx().culling(true);
    let _ = spring
        .gfx()
        .cull_face(spring::gfx::GfxCullFace::GfxCullFaceFront);
    let _ = spring.gfx().depth_mask(false);
    let _ = spring.gfx().color(0.0, 0.0, 0.0, 1.0);
    let _ = spring.gfx().push_matrix();
    let _ = spring.gfx().reset_matrices();
    let _ = spring.gfx().unit(
        unit_id,
        spring::gfx::GfxUnitDrawOptions {
            apply_transform: true,
            do_raw_draw: true,
            no_lua_call: false,
            full_model: true,
            ..Default::default()
        },
    );
    let _ = spring.gfx().pop_matrix();
    let _ = spring.gfx().pop_attrib();
}
