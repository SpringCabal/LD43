use alloc::{format, string::String};
use core::cell::{Cell, RefCell};

use spring::{gfx, gl};

use ttld_shared::minimap::{Viewport, centered_viewport, viewport_for_position};

use crate::gl_uniforms;

const SOURCE_SIZE: i32 = 600;
const SCREEN_X: f32 = 0.0;
const SCREEN_Y: f32 = 0.0;
const MINIMAP_ALPHA: f32 = 1.0;

#[derive(Default)]
pub struct Minimap {
    configured: Cell<bool>,
    slaved: Cell<bool>,
    resources: RefCell<Option<Resources>>,
}

struct Resources {
    source: String,
    fbo: u32,
    shader: u32,
    alpha: i32,
    source_bounds: i32,
    screen_rect: i32,
    screen: i32,
    width: i32,
    height: i32,
}

impl Minimap {
    pub fn draw(
        &self,
        spring: &spring::Spring<'_, spring::Ui>,
        player_unit: i32,
        width: i32,
        height: i32,
    ) {
        if width <= 0 || height <= 0 {
            return;
        }

        if !self.configured.replace(true) {
            configure(spring, height);
        }

        if !self.slaved.get() {
            if spring.gfx().slave_mini_map(true).is_err() {
                return;
            }
            self.slaved.set(true);
        }

        push_lua_state(spring);
        self.ensure_resources(spring, width, height);
        if let Some(resources) = self.resources.borrow().as_ref() {
            draw_composed(spring, resources, player_unit);
        } else {
            let _ = spring.gfx().draw_mini_map(true);
        }
        pop_lua_state(spring);
    }

    pub fn shutdown(&self, spring: &spring::Spring<'_, spring::Ui>) {
        if self.slaved.replace(false) {
            let _ = spring.gfx().slave_mini_map(false);
        }
        if let Some(resources) = self.resources.borrow_mut().take() {
            destroy(spring, resources);
        }
        self.configured.set(false);
    }

    fn ensure_resources(&self, spring: &spring::Spring<'_, spring::Ui>, width: i32, height: i32) {
        let mut slot = self.resources.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|resources| resources.width == width && resources.height == height)
        {
            return;
        }
        if let Some(old) = slot.take() {
            destroy(spring, old);
        }
        *slot = create_resources(spring, width, height);
    }
}

fn configure(spring: &spring::Spring<'_, spring::Ui>, height: i32) {
    if let (Ok(geometry), Ok(view)) = (
        spring.display().get_mini_map_geometry(),
        spring.display().get_view_geometry(),
    ) {
        let top_y = view.view_size_y - geometry.pos_y - geometry.size_y;
        let command = format!(
            "minimap geo {} {} {} {}",
            geometry.pos_x, top_y, geometry.size_x, geometry.size_y
        );
        let _ = spring.messages().send_commands(&command, "");
    }
    let _ = spring.messages().send_commands("minimap minimize 0", "");
    let _ = spring
        .messages()
        .send_commands("minimap simplecolors 1", "");
    let _ = spring.messages().send_commands("minimap unitexp 0.3", "");
    let _ = spring.messages().send_commands("minimap unitsize 1.2", "");
    let _ = spring.messages().send_commands("minimap fullproxy 0", "");
    let _ = spring
        .gfx()
        .config_mini_map(0, height / 2, SOURCE_SIZE, SOURCE_SIZE);
}

fn create_resources(
    spring: &spring::Spring<'_, spring::Ui>,
    width: i32,
    height: i32,
) -> Option<Resources> {
    let source = spring
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

    let attachment = gfx::GfxFBOAttachment {
        attachment: gl::COLOR_ATTACHMENT0,
        texture_name: source.clone(),
        texture_target: gl::TEXTURE_2D,
        mip_level: 0,
        rbo_id: 0,
        use_rbo: false,
    };
    let fbo = spring
        .gfx()
        .create_fbo(gl::FRAMEBUFFER, &[attachment], &[gl::COLOR_ATTACHMENT0], 0)
        .ok()?
        .fbo_id;

    let shader = spring
        .gfx()
        .create_shader(
            "",
            MINIMAP_VERTEX,
            "",
            "",
            "",
            MINIMAP_FRAGMENT,
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

    let resources = Resources {
        source,
        fbo,
        shader,
        alpha: gl_uniforms::location(spring, shader, "alpha")?,
        source_bounds: gl_uniforms::location(spring, shader, "sourceBounds")?,
        screen_rect: gl_uniforms::location(spring, shader, "screenRect")?,
        screen: gl_uniforms::location(spring, shader, "screen")?,
        width,
        height,
    };

    let _ = spring.gfx().use_shader(shader);
    let _ = gl_uniforms::set_int(spring, shader, "tex0", 0);
    let _ = spring.gfx().use_shader(0);
    Some(resources)
}

fn destroy(spring: &spring::Spring<'_, spring::Ui>, resources: Resources) {
    let _ = spring.gfx().use_shader(0);
    let _ = spring.gfx().delete_shader(resources.shader);
    let _ = spring.gfx().delete_fbo(resources.fbo);
    let _ = spring.gfx().delete_texture_fbo(&resources.source);
}

fn draw_composed(spring: &spring::Spring<'_, spring::Ui>, resources: &Resources, player_unit: i32) {
    if spring
        .gfx()
        .active_fbo(resources.fbo, gl::FRAMEBUFFER, false, |spring| {
            draw_source(spring)
        })
        .is_err()
    {
        return;
    }

    let Some(map) = spring.game().get_game_map_info().ok() else {
        return;
    };
    let viewport = player_viewport(
        spring,
        player_unit,
        map.map_size_x as f32,
        map.map_size_z as f32,
    );
    let Some(geometry) = spring.display().get_mini_map_geometry().ok() else {
        return;
    };
    let source_width = resources.width as f32;
    let source_height = resources.height as f32;
    let map_width = map.map_size_x as f32;
    let map_height = map.map_size_z as f32;
    let minimap_width = geometry.size_x as f32;
    let minimap_height = geometry.size_y as f32;

    let source_left =
        (geometry.pos_x as f32 + minimap_width * viewport.origin_x / map_width) / source_width;
    let source_bottom = (geometry.pos_y as f32
        + minimap_height * (1.0 - (viewport.origin_z + viewport.height) / map_height))
        / source_height;
    let source_width_crop = minimap_width * viewport.width / map_width / source_width;
    let source_height_crop = minimap_height * viewport.height / map_height / source_height;
    let screen_width = minimap_width * viewport.width / map_width * viewport.scale;
    let screen_height = minimap_height * viewport.height / map_height * viewport.scale;

    let _ = spring.gfx().blending(true);
    let _ = spring.gfx().bind_texture(&resources.source, 0, true);
    let _ = spring.gfx().use_shader(resources.shader);
    let _ = spring
        .gfx()
        .uniform(resources.alpha, &[MINIMAP_ALPHA * 3.0], 1);
    let _ = spring.gfx().uniform(
        resources.source_bounds,
        &[
            source_left,
            source_bottom,
            source_width_crop,
            source_height_crop,
        ],
        4,
    );
    let _ = spring.gfx().uniform(
        resources.screen_rect,
        &[
            SCREEN_X * resources.width as f32,
            SCREEN_Y * resources.height as f32,
            screen_width,
            screen_height,
        ],
        4,
    );
    let _ = spring
        .gfx()
        .uniform(resources.screen, &[source_width, source_height], 2);
    let _ = spring
        .gfx()
        .tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
    let _ = spring.gfx().use_shader(0);
    let _ = spring.gfx().bind_texture(&resources.source, 0, false);
    let _ = spring.gfx().blending(false);
}

fn player_viewport(
    spring: &spring::Spring<'_, spring::Ui>,
    unit_id: i32,
    map_size_x: f32,
    map_size_z: f32,
) -> Viewport {
    let Ok(position) = spring.units_info().get_unit_position(
        unit_id,
        spring::units_info::GetUnitPositionOptions {
            mid_pos: false,
            aim_pos: false,
        },
    ) else {
        return centered_viewport(map_size_x, map_size_z);
    };

    viewport_for_position(position.x, position.z)
}

fn draw_source(spring: &spring::Spring<'_, spring::Ui>) {
    let _ = spring
        .gfx()
        .clear(gl::COLOR_BUFFER_BIT, &[0.0, 0.0, 0.0, 0.0], 4);
    let _ = spring.gfx().draw_mini_map(true);
}

fn push_lua_state(spring: &spring::Spring<'_, spring::Ui>) {
    let _ = spring.gfx().push_attrib(gl::ALL_ATTRIB_BITS);
    let _ = spring.gfx().matrix_mode(gl::PROJECTION);
    let _ = spring.gfx().push_matrix();
    let _ = spring.gfx().matrix_mode(gl::MODELVIEW);
    let _ = spring.gfx().push_matrix();
}

fn pop_lua_state(spring: &spring::Spring<'_, spring::Ui>) {
    let _ = spring.gfx().matrix_mode(gl::PROJECTION);
    let _ = spring.gfx().pop_matrix();
    let _ = spring.gfx().matrix_mode(gl::MODELVIEW);
    let _ = spring.gfx().pop_matrix();
    let _ = spring.gfx().pop_attrib();
}

const MINIMAP_VERTEX: &str = r#"
varying vec2 mapCoord;

void main() {
    mapCoord = gl_Vertex.xy * 0.5 + 0.5;
    gl_Position = vec4(gl_Vertex.xyz, 1.0);
}
"#;

const MINIMAP_FRAGMENT: &str = r#"
uniform sampler2D tex0;
uniform float alpha;
uniform vec4 sourceBounds;
uniform vec2 screen;
uniform vec4 screenRect;
varying vec2 mapCoord;

const float edgeFadePixels = 16.0;

void main(void) {
    vec2 screenPosition = vec2(
        mapCoord.x * screen.x,
        (1.0 - mapCoord.y) * screen.y);
    vec2 cropCoord = (screenPosition - screenRect.xy) / screenRect.zw;
    if (cropCoord.x < 0.0 || cropCoord.x > 1.0 ||
        cropCoord.y < 0.0 || cropCoord.y > 1.0) {
        discard;
    }

    vec2 texCoord = vec2(
        sourceBounds.x + cropCoord.x * sourceBounds.z,
        sourceBounds.y + (1.0 - cropCoord.y) * sourceBounds.w);
    vec4 color = texture2D(tex0, texCoord.st);
    float edgeFadeScaledPixels = edgeFadePixels / 1080.0 * screen.y * 20.0;
    vec2 edgeFade = clamp(edgeFadeScaledPixels / screenRect.zw, 0.0, 1.0);
    vec2 edgeAlpha = min(cropCoord, 1.0 - cropCoord) / max(edgeFade, vec2(0.0001));
    float final_alpha = min(min(edgeAlpha.x, edgeAlpha.y), 1.0) * alpha;
    gl_FragColor = vec4(color.rgb, final_alpha);
}
"#;
