use core::cell::Cell;

use spring::{ResultExt, gl};

use crate::gl_uniforms;

#[derive(Default)]
pub struct MapEdge {
    resources: Cell<Option<Resources>>,
    setup_failed: Cell<bool>,
}

#[derive(Clone, Copy)]
struct Resources {
    shader: u32,
    list: u32,
    uniforms: Uniforms,
    width: f32,
    height: f32,
    bounds: Option<(f32, f32)>,
}

impl Resources {
    fn draw_tile(&self, spring: &spring::Spring<'_, spring::Ui>, x: f32, z: f32) {
        if let Some((center_y, radius)) = self.bounds {
            let center = spring::Float3::new(x + self.width * 0.5, center_y, z + self.height * 0.5);
            if !spring
                .display()
                .is_sphere_in_view(center, radius)
                .log_err_once(spring, "checking mirrored terrain visibility")
                .unwrap_or(true)
            {
                return;
            }
        }
        let _ = spring.gfx().call_list(self.list);
    }
}

impl MapEdge {
    pub fn draw(&self, spring: &spring::Spring<'_, spring::Ui>) {
        if let Some(resources) = self.resources(spring) {
            draw(spring, resources);
        }
    }

    fn resources(&self, spring: &spring::Spring<'_, spring::Ui>) -> Option<Resources> {
        if let Some(resources) = self.resources.get() {
            return Some(resources);
        }
        if self.setup_failed.get() {
            return None;
        }
        let resources = create(spring);
        self.resources.set(resources);
        self.setup_failed.set(resources.is_none());
        resources
    }
}

fn draw(spring: &spring::Spring<'_, spring::Ui>, resources: Resources) {
    let Resources {
        shader: shader_id,
        uniforms,
        width,
        height,
        ..
    } = resources;
    let _ = spring.gfx().fog(true);
    let _ = spring.gfx().fog_coord(1.0);
    let _ = spring.gfx().use_shader(shader_id);
    let _ = spring.gfx().push_matrix();
    let _ = spring.gfx().depth_mask(true);
    let _ = spring.gfx().bind_texture(GROUND_TEXTURE, 0, true);
    uniforms.set(spring, Uniform::Brightness, TEXTURE_BRIGHTNESS);
    uniforms.set(spring, Uniform::Grid, 0.0);
    uniforms.set(spring, Uniform::MirrorX, width);
    uniforms.set(spring, Uniform::MirrorZ, height);
    uniforms.set(spring, Uniform::LengthX, width);
    uniforms.set(spring, Uniform::LengthZ, height);
    uniforms.set(spring, Uniform::Left, 1.0);
    uniforms.set(spring, Uniform::Up, 1.0);
    translate(spring, -width, -height);
    resources.draw_tile(spring, -width, -height);
    uniforms.set(spring, Uniform::Left, 0.0);
    translate(spring, width * 2.0, 0.0);
    resources.draw_tile(spring, width, -height);
    uniforms.set(spring, Uniform::Up, 0.0);
    translate(spring, 0.0, height * 2.0);
    resources.draw_tile(spring, width, height);
    uniforms.set(spring, Uniform::Left, 1.0);
    translate(spring, -width * 2.0, 0.0);
    resources.draw_tile(spring, -width, height);

    uniforms.set(spring, Uniform::MirrorX, 0.0);
    translate(spring, width, 0.0);
    resources.draw_tile(spring, 0.0, height);
    uniforms.set(spring, Uniform::Left, 0.0);
    uniforms.set(spring, Uniform::Up, 1.0);
    translate(spring, 0.0, -height * 2.0);
    resources.draw_tile(spring, 0.0, -height);

    uniforms.set(spring, Uniform::Up, 0.0);
    uniforms.set(spring, Uniform::MirrorZ, 0.0);
    uniforms.set(spring, Uniform::MirrorX, width);
    translate(spring, width, height);
    resources.draw_tile(spring, width, 0.0);
    uniforms.set(spring, Uniform::Left, 1.0);
    translate(spring, -width * 2.0, 0.0);
    resources.draw_tile(spring, -width, 0.0);

    let _ = spring.gfx().depth_mask(false);
    let _ = spring.gfx().bind_texture(GROUND_TEXTURE, 0, false);
    let _ = spring.gfx().pop_matrix();
    let _ = spring.gfx().use_shader(0);
    let _ = spring.gfx().fog(false);
}

const GRID_SIZE: f32 = 32.0;
const TEXTURE_BRIGHTNESS: f32 = 0.5;
const GROUND_TEXTURE: &str = "$grass";

const VERTEX_SOURCE: &str = include_str!("../../shaders/mapedge.vert");

#[derive(Clone, Copy)]
enum Uniform {
    MirrorX,
    MirrorZ,
    LengthX,
    LengthZ,
    Up,
    Left,
    Grid,
    Brightness,
}

#[derive(Clone, Copy)]
struct Uniforms {
    mirror_x: i32,
    mirror_z: i32,
    length_x: i32,
    length_z: i32,
    up: i32,
    left: i32,
    grid: i32,
    brightness: i32,
}

impl Uniforms {
    fn locate(spring: &spring::Spring<'_, spring::Ui>, shader_id: u32) -> Self {
        Self {
            mirror_x: location(spring, shader_id, "mirrorX"),
            mirror_z: location(spring, shader_id, "mirrorZ"),
            length_x: location(spring, shader_id, "lengthX"),
            length_z: location(spring, shader_id, "lengthZ"),
            up: location(spring, shader_id, "up"),
            left: location(spring, shader_id, "left"),
            grid: location(spring, shader_id, "grid"),
            brightness: location(spring, shader_id, "brightness"),
        }
    }

    fn set(&self, spring: &spring::Spring<'_, spring::Ui>, uniform: Uniform, value: f32) {
        let location = match uniform {
            Uniform::MirrorX => self.mirror_x,
            Uniform::MirrorZ => self.mirror_z,
            Uniform::LengthX => self.length_x,
            Uniform::LengthZ => self.length_z,
            Uniform::Up => self.up,
            Uniform::Left => self.left,
            Uniform::Grid => self.grid,
            Uniform::Brightness => self.brightness,
        };
        if location >= 0 {
            let _ = spring.gfx().uniform(location, &[value], 1);
        }
    }
}

fn location(spring: &spring::Spring<'_, spring::Ui>, shader_id: u32, name: &str) -> i32 {
    gl_uniforms::location(spring, shader_id, name).unwrap_or(-1)
}

fn create(spring: &spring::Spring<'_, spring::Ui>) -> Option<Resources> {
    let shader = spring
        .gfx()
        .create_shader("", VERTEX_SOURCE, "", "", "", "", "", Default::default())
        .and_then(|shader| {
            spring.gfx().use_shader(shader.shader_id)?;
            spring.gfx().use_shader(0)?;
            Ok(shader)
        })
        .ok()
        .map(|shader| shader.shader_id)
        .filter(|shader| *shader != 0)?;
    let list = spring.gfx().create_list(|spring| emit_list(spring)).ok()?;
    let (width, height) = map_size(spring);
    // The display list freezes the terrain heights at creation. Its full tile
    // sphere stays conservative for the geometry, including mirrored corners.
    let bounds = spring
        .terrain()
        .get_ground_extremes()
        .log_err_once(spring, "reading mirrored terrain bounds")
        .map(|bounds| {
            let half_y = (bounds.curr_max_height - bounds.curr_min_height) * 0.5;
            (
                bounds.curr_min_height + half_y,
                ((width * 0.5).powi(2) + (height * 0.5).powi(2) + half_y.powi(2)).sqrt() + 1.0,
            )
        });
    Some(Resources {
        shader,
        list,
        uniforms: Uniforms::locate(spring, shader),
        width,
        height,
        bounds,
    })
}

fn emit_list(spring: &spring::Spring<'_, spring::Ui>) {
    let _ = spring.gfx().depth_test(spring::gfx::GfxDepthTestOptions {
        enable: true,
        set_func: true,
        func: gl::LEQUAL,
    });
    let _ = spring.gfx().bind_texture(GROUND_TEXTURE, 0, true);
    let _ = spring
        .gfx()
        .begin_end(gl::TRIANGLE_STRIP, |spring| emit_vertices(spring));
    let _ = spring.gfx().color(1.0, 1.0, 1.0, 1.0);
}

fn emit_vertices(spring: &spring::Spring<'_, spring::Ui>) {
    let (width, height) = map_size(spring);
    let _ = spring.gfx().color(1.0, 1.0, 1.0, 1.0);
    let last_row = (height / GRID_SIZE).floor() * GRID_SIZE;
    let mut current_x = 0.0;
    let mut reverse = false;
    let mut x = 0.0;
    while x <= width - GRID_SIZE {
        let previous_x = current_x;
        current_x += GRID_SIZE;
        let next_x = x + GRID_SIZE;
        let (from, to, step) = if reverse {
            (last_row, 0.0, -GRID_SIZE)
        } else {
            (0.0, last_row, GRID_SIZE)
        };
        reverse = !reverse;
        let mut z = from;
        loop {
            emit_pair(spring, previous_x, current_x, z, x, next_x, width, height);
            if (step > 0.0 && z >= to) || (step < 0.0 && z <= to) {
                break;
            }
            z += step;
        }
        x = next_x;
    }
}

fn emit_pair(
    spring: &spring::Spring<'_, spring::Ui>,
    map_x0: f32,
    map_x1: f32,
    z: f32,
    vertex_x0: f32,
    vertex_x1: f32,
    width: f32,
    height: f32,
) {
    let _ = spring
        .gfx()
        .tex_coord(map_x0 / width, z / height, 0.0, 0.0, 2);
    let _ = spring
        .gfx()
        .vertex(vertex_x0, ground(spring, map_x0, z), z, 1.0, 1);
    let _ = spring
        .gfx()
        .tex_coord(map_x1 / width, z / height, 0.0, 0.0, 2);
    let _ = spring
        .gfx()
        .vertex(vertex_x1, ground(spring, map_x1, z), z, 1.0, 1);
}

fn ground(spring: &spring::Spring<'_, spring::Ui>, x: f32, z: f32) -> f32 {
    spring.terrain().get_ground_height(x, z).unwrap_or(0.0)
}

fn map_size(spring: &spring::Spring<'_, spring::Ui>) -> (f32, f32) {
    spring
        .game()
        .get_game_map_info()
        .map(|map| (map.map_size_x as f32, map.map_size_z as f32))
        .unwrap_or((0.0, 0.0))
}

fn translate(spring: &spring::Spring<'_, spring::Ui>, x: f32, z: f32) {
    let _ = spring.gfx().translate(x, 0.0, z);
}
