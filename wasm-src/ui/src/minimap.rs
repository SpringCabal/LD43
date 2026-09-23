use alloc::{format, string::String};
use core::cell::{Cell, RefCell};

use spring::{gfx, gl};

use crate::gl_uniforms;

const MINIMAP_SCALE: f32 = 1.3;
const SOURCE_SIZE: i32 = 600;
const SCREEN_X: f32 = 0.0;
const SCREEN_Y: f32 = 0.0;
const MINIMAP_ALPHA: f32 = 1.0;
// The viewport is as large as the town area, x 3300..8000 and z 3400..8100.
const VIEWPORT_WORLD_X: f32 = 8000.0 - 3300.0;
const VIEWPORT_WORLD_Z: f32 = 8100.0 - 3400.0;

thread_local! {
    static CONFIGURED: Cell<bool> = const { Cell::new(false) };
    static SLAVED: Cell<bool> = const { Cell::new(false) };
    static RESOURCES: RefCell<Option<Resources>> = const { RefCell::new(None) };
}

#[derive(Clone)]
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

#[derive(Clone, Copy)]
struct Viewport {
    origin_x: f32,
    origin_z: f32,
    width: f32,
    height: f32,
    scale: f32,
}

/// Draw the same cropped/faded player-centered viewport as
/// LuaUI/Widgets/ui_minimap_viewport.lua.
///
/// The map itself remains engine-owned: the source pass calls the native
/// minimap renderer in the same FBO/configuration used by the Lua widget. The
/// second pass is the Lua widget's crop, scale and edge-fade shader.
pub fn draw(width: i32, height: i32) {
    if width <= 0 || height <= 0 {
        return;
    }

    configure(width, height);

    if !SLAVED.get() {
        if gfx::slave_mini_map(true).is_err() {
            return;
        }
        SLAVED.set(true);
    }

    // Match ui_minimap_viewport.lua: the native source draw and the screen
    // quad both run inside one GL attribute/matrix scope.  In particular,
    // ActiveFBO deliberately preserves the current matrices.
    push_lua_state();
    if let Some(resources) = resources(width, height) {
        draw_composed(&resources);
    } else {
        // Match the Lua widget's shader-unavailable fallback. Lua omits the
        // optional argument, whose API default is true (the default minimap
        // transform).
        let _ = gfx::draw_mini_map(true);
    }
    pop_lua_state();
}

pub fn shutdown() {
    if SLAVED.get() {
        let _ = gfx::slave_mini_map(false);
        SLAVED.set(false);
    }
    RESOURCES.with(|cell| {
        if let Some(resources) = cell.borrow_mut().take() {
            destroy(resources);
        }
    });
    CONFIGURED.set(false);
}

fn configure(_width: i32, height: i32) {
    if !CONFIGURED.get() {
        // These are the settings used by the original Lua widget. Applying
        // them once avoids enqueueing the same engine commands every frame.
        if let (Ok(geometry), Ok(view)) = (
            spring::display::get_mini_map_geometry(0),
            spring::display::get_view_geometry(0),
        ) {
            // GetMiniMapGeometry exposes the engine's bottom-left GL
            // coordinates; the console command expects top-left config
            // coordinates, just like the Lua widget's config string.
            let top_y = view.view_size_y - geometry.pos_y - geometry.size_y;
            let command = format!(
                "minimap geo {} {} {} {}",
                geometry.pos_x, top_y, geometry.size_x, geometry.size_y
            );
            let _ = spring::send_commands(&command, "");
        }
        let _ = spring::send_commands("minimap simplecolors 1", "");
        let _ = spring::send_commands("minimap unitexp 0.3", "");
        let _ = spring::send_commands("minimap unitsize 1.2", "");
        let _ = spring::send_commands("minimap fullproxy 0", "");
        let _ = gfx::config_mini_map(0, height / 2, SOURCE_SIZE, SOURCE_SIZE);
        CONFIGURED.set(true);
    }
}

fn resources(width: i32, height: i32) -> Option<Resources> {
    RESOURCES.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|resources| resources.width == width && resources.height == height)
        {
            return slot.clone();
        }
        if let Some(old) = slot.take() {
            destroy(old);
        }
        let created = create_resources(width, height);
        *slot = created.clone();
        created
    })
}

fn create_resources(width: i32, height: i32) -> Option<Resources> {
    let source = spring::create_texture(
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

    let attachment = gfx::GfxFBOAttachment {
        attachment: gl::COLOR_ATTACHMENT0,
        texture_name: source.clone(),
        texture_target: gl::TEXTURE_2D,
        mip_level: 0,
        rbo_id: 0,
        use_rbo: false,
    };
    let fbo = gfx::create_fbo(gl::FRAMEBUFFER, &[attachment], &[gl::COLOR_ATTACHMENT0], 0)
        .ok()?
        .fbo_id;

    let shader = gfx::create_shader(gfx::ShaderCreateParams {
        vertex: MINIMAP_VERTEX,
        fragment: MINIMAP_FRAGMENT,
        ..Default::default()
    })
    .ok()?
    .shader_id;

    let resources = Resources {
        source,
        fbo,
        shader,
        alpha: gl_uniforms::location(shader, "alpha")?,
        source_bounds: gl_uniforms::location(shader, "sourceBounds")?,
        screen_rect: gl_uniforms::location(shader, "screenRect")?,
        screen: gl_uniforms::location(shader, "screen")?,
        width,
        height,
    };

    let _ = gfx::use_shader(shader);
    let _ = gl_uniforms::set_int(shader, "tex0", 0);
    let _ = gfx::use_shader(0);
    Some(resources)
}

fn destroy(resources: Resources) {
    let _ = gfx::use_shader(0);
    let _ = gfx::delete_shader(resources.shader);
    let _ = gfx::delete_fbo(resources.fbo);
    let _ = gfx::delete_texture_fbo(&resources.source);
}

fn draw_composed(resources: &Resources) {
    // Lua's ActiveFBO call leaves the current matrices intact, so identities
    // must be false here. Its DrawMiniMap() call also uses the configured
    // geometry (false), not the transformed boolean path.
    if gfx::active_fbo(resources.fbo, gl::FRAMEBUFFER, false, draw_source).is_err() {
        return;
    }

    let Some(map) = spring::game::get_game_map_info(0).ok() else {
        return;
    };
    let viewport = player_viewport(map.map_size_x as f32, map.map_size_z as f32);
    let Some(geometry) = spring::display::get_mini_map_geometry(0).ok() else {
        return;
    };
    let source_width = resources.width as f32;
    let source_height = resources.height as f32;
    let map_width = map.map_size_x as f32;
    let map_height = map.map_size_z as f32;
    let minimap_width = geometry.size_x as f32;
    let minimap_height = geometry.size_y as f32;

    // Native minimap coordinates are world X and a GL Y coordinate with
    // world-Z flipped. Convert the player-centered crop directly into the
    // off-screen texture's coordinates.
    let source_left =
        (geometry.pos_x as f32 + minimap_width * viewport.origin_x / map_width) / source_width;
    let source_bottom = (geometry.pos_y as f32
        + minimap_height * (1.0 - (viewport.origin_z + viewport.height) / map_height))
        / source_height;
    let source_width_crop = minimap_width * viewport.width / map_width / source_width;
    let source_height_crop = minimap_height * viewport.height / map_height / source_height;
    let screen_width = minimap_width * viewport.width / map_width * viewport.scale;
    let screen_height = minimap_height * viewport.height / map_height * viewport.scale;

    let _ = gfx::blending(true);
    let _ = gfx::bind_texture(&resources.source, 0, true);
    let _ = gfx::use_shader(resources.shader);
    let _ = gfx::uniform(resources.alpha, &[MINIMAP_ALPHA * 3.0], 1);
    let _ = gfx::uniform(
        resources.source_bounds,
        &[
            source_left,
            source_bottom,
            source_width_crop,
            source_height_crop,
        ],
        4,
    );
    let _ = gfx::uniform(
        resources.screen_rect,
        &[
            SCREEN_X * resources.width as f32,
            SCREEN_Y * resources.height as f32,
            screen_width,
            screen_height,
        ],
        4,
    );
    let _ = gfx::uniform(resources.screen, &[source_width, source_height], 2);
    // Lua's gl.TexRect(-1, 1, 1, -1) uses the engine's default texture
    // coordinates: t=1 at the top vertex and t=0 at the bottom vertex.
    let _ = gfx::tex_rect(-1.0, 1.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0);
    let _ = gfx::use_shader(0);
    let _ = gfx::bind_texture(&resources.source, 0, false);
    let _ = gfx::blending(false);
}

fn player_viewport(map_size_x: f32, map_size_z: f32) -> Viewport {
    let unit_id = crate::state::with_state(|ui| ui.player_unit).unwrap_or(-1);
    let Ok(position) = spring::units_info::get_unit_position(
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

fn viewport_for_position(position_x: f32, position_z: f32) -> Viewport {
    // Keep the crop centered on the unit even when its origin is outside the
    // map. The source texture is edge-clamped, so the map boundary supplies
    // the small amount of padding needed at the edge without changing the
    // minimap's layout or zoom.
    Viewport {
        origin_x: position_x - VIEWPORT_WORLD_X * 0.5,
        origin_z: position_z - VIEWPORT_WORLD_Z * 0.5,
        width: VIEWPORT_WORLD_X,
        height: VIEWPORT_WORLD_Z,
        scale: MINIMAP_SCALE,
    }
}

fn centered_viewport(map_size_x: f32, map_size_z: f32) -> Viewport {
    Viewport {
        origin_x: (map_size_x - VIEWPORT_WORLD_X) * 0.5,
        origin_z: (map_size_z - VIEWPORT_WORLD_Z) * 0.5,
        width: VIEWPORT_WORLD_X,
        height: VIEWPORT_WORLD_Z,
        scale: MINIMAP_SCALE,
    }
}

#[cfg(test)]
mod tests {
    use super::{MINIMAP_SCALE, VIEWPORT_WORLD_X, VIEWPORT_WORLD_Z, viewport_for_position};

    #[test]
    fn centered_viewport_follows_player_interior() {
        let viewport = viewport_for_position(5580.0, 7100.0);

        assert_eq!(viewport.origin_x, 5580.0 - VIEWPORT_WORLD_X * 0.5);
        assert_eq!(viewport.origin_z, 7100.0 - VIEWPORT_WORLD_Z * 0.5);
        assert_eq!(viewport.width, VIEWPORT_WORLD_X);
        assert_eq!(viewport.height, VIEWPORT_WORLD_Z);
        assert_eq!(viewport.scale, MINIMAP_SCALE);
    }

    #[test]
    fn north_edge_keeps_player_centered() {
        let viewport = viewport_for_position(5580.0, 1000.0);

        assert_eq!(viewport.origin_x, 5580.0 - VIEWPORT_WORLD_X * 0.5);
        assert_eq!(viewport.origin_z, 1000.0 - VIEWPORT_WORLD_Z * 0.5);
        assert_eq!(viewport.width, VIEWPORT_WORLD_X);
        assert_eq!(viewport.height, VIEWPORT_WORLD_Z);
        assert_eq!(viewport.scale, MINIMAP_SCALE);
    }
}

fn draw_source() {
    let _ = gfx::clear(gl::COLOR_BUFFER_BIT, &[0.0, 0.0, 0.0, 0.0], 4);
    // The Lua widget calls gl.DrawMiniMap() without an argument. Its
    // defaultTransform parameter defaults to true.
    let _ = gfx::draw_mini_map(true);
}

fn push_lua_state() {
    let _ = gfx::push_attrib(gl::ALL_ATTRIB_BITS);
    let _ = gfx::matrix_mode(gl::PROJECTION);
    let _ = gfx::push_matrix(0);
    let _ = gfx::matrix_mode(gl::MODELVIEW);
    let _ = gfx::push_matrix(0);
}

fn pop_lua_state() {
    let _ = gfx::matrix_mode(gl::PROJECTION);
    let _ = gfx::pop_matrix(0);
    let _ = gfx::matrix_mode(gl::MODELVIEW);
    let _ = gfx::pop_matrix(0);
    let _ = gfx::pop_attrib(0);
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
    // The Lua widget's full-screen quad is kept: Spring's screen projection
    // is not always the conventional OpenGL pixel space.
    // Work out the crop in the fragment shader, where screen coordinates are
    // unambiguous, and discard fragments outside the displayed rectangle.
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
