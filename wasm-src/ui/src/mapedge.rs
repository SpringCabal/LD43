use spring::callback::SyncCallback;

use crate::state;

pub const CALLBACK_LIST: u32 = 2;
pub const CALLBACK_VERTICES: u32 = 3;

pub fn draw() {
    let Some((shader_id, list_id)) = resources() else {
        return;
    };
    let (width, height) = map_size();
    let _ = spring::gfx::fog(true);
    let _ = spring::gfx::fog_coord(1.0);
    let _ = spring::gfx::use_shader(shader_id);
    let _ = spring::gfx::push_matrix(0);
    let _ = spring::gfx::depth_mask(true);
    let _ = spring::gfx::bind_texture(GROUND_TEXTURE, 0, true);
    let uniforms = Uniforms::locate(shader_id);
    uniforms.set(Uniform::Brightness, TEXTURE_BRIGHTNESS);
    uniforms.set(Uniform::Grid, 0.0);
    uniforms.set(Uniform::MirrorX, width);
    uniforms.set(Uniform::MirrorZ, height);
    uniforms.set(Uniform::LengthX, width);
    uniforms.set(Uniform::LengthZ, height);
    uniforms.set(Uniform::Left, 1.0);
    uniforms.set(Uniform::Up, 1.0);
    translate(-width, -height);
    call(list_id);
    uniforms.set(Uniform::Left, 0.0);
    translate(width * 2.0, 0.0);
    call(list_id);
    uniforms.set(Uniform::Up, 0.0);
    translate(0.0, height * 2.0);
    call(list_id);
    uniforms.set(Uniform::Left, 1.0);
    translate(-width * 2.0, 0.0);
    call(list_id);

    uniforms.set(Uniform::MirrorX, 0.0);
    translate(width, 0.0);
    call(list_id);
    uniforms.set(Uniform::Left, 0.0);
    uniforms.set(Uniform::Up, 1.0);
    translate(0.0, -height * 2.0);
    call(list_id);

    uniforms.set(Uniform::Up, 0.0);
    uniforms.set(Uniform::MirrorZ, 0.0);
    uniforms.set(Uniform::MirrorX, width);
    translate(width, height);
    call(list_id);
    uniforms.set(Uniform::Left, 1.0);
    translate(-width * 2.0, 0.0);
    call(list_id);

    let _ = spring::gfx::depth_mask(false);
    let _ = spring::gfx::bind_texture(GROUND_TEXTURE, 0, false);
    let _ = spring::gfx::pop_matrix(0);
    let _ = spring::gfx::use_shader(0);
    let _ = spring::gfx::fog(false);
}

pub fn dispatch_callback(callback_id: u32) {
    match callback_id {
        CALLBACK_LIST => emit_list(),
        CALLBACK_VERTICES => emit_vertices(),
        _ => {}
    }
}

const GRID_SIZE: f32 = 32.0;
const TEXTURE_BRIGHTNESS: f32 = 0.5;
const GROUND_TEXTURE: &str = "$grass";
const GL_TRIANGLE_STRIP: u32 = 5;
const GL_LEQUAL: u32 = 0x0203;

const VERTEX_SOURCE: &str = r#"
		// Application to vertex shader
		uniform float mirrorX;
		uniform float mirrorZ;
		uniform float lengthX;
	  uniform float lengthZ;
		uniform float left;
		uniform float up;
		uniform float brightness;

		varying vec4 vertex;
		varying vec4 color;

		void main()
		{
		gl_TexCoord[0]= gl_TextureMatrix[0]*gl_MultiTexCoord0;
		gl_Vertex.x = abs(mirrorX-gl_Vertex.x);
		gl_Vertex.z = abs(mirrorZ-gl_Vertex.z);

		float alpha = 1.0;

		float ff = 20000.0;
		if((mirrorZ && mirrorX))
	ff=ff/(pow(abs(gl_Vertex.z-up*mirrorZ)/150.0, 2.0)+pow(abs(gl_Vertex.x-left*mirrorX)/150.0, 2.0)+2.0);
		else if(mirrorX)
	ff=ff/(pow(abs(gl_Vertex.x-left*mirrorX)/150.0, 2.0)+2.0);
		else if(mirrorZ)
	ff=ff/(pow(abs(gl_Vertex.z-up*mirrorZ)/150.0, 2.0)+2.0);

		gl_Position  = gl_ModelViewProjectionMatrix*gl_Vertex;

		gl_FrontColor = vec4(brightness * gl_Color.rgb, alpha);

		color = gl_FrontColor;
		vertex = gl_Vertex;
		}
"#;

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
    fn locate(shader_id: u32) -> Self {
        Self {
            mirror_x: location(shader_id, "mirrorX"),
            mirror_z: location(shader_id, "mirrorZ"),
            length_x: location(shader_id, "lengthX"),
            length_z: location(shader_id, "lengthZ"),
            up: location(shader_id, "up"),
            left: location(shader_id, "left"),
            grid: location(shader_id, "grid"),
            brightness: location(shader_id, "brightness"),
        }
    }

    fn set(&self, uniform: Uniform, value: f32) {
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
            let _ = spring::gfx::uniform(location, &[value], 1);
        }
    }
}

fn location(shader_id: u32, name: &str) -> i32 {
    spring::gfx::get_uniform_location(shader_id, name).unwrap_or(-1)
}

fn resources() -> Option<(u32, u32)> {
    let existing = state::with_state(|ui| (ui.edge_shader, ui.edge_list)).unwrap_or((0, 0));
    if existing.0 != 0 && existing.1 != 0 {
        return Some(existing);
    }
    if state::with_state(|ui| ui.edge_setup_failed).unwrap_or(true) {
        return None;
    }
    let shader = spring::gfx::create_shader(
        "",
        VERTEX_SOURCE,
        "",
        "",
        "",
        "",
        "",
        spring::gfx::GfxCreateShaderOptions::default(),
    )
    .ok();
    let shader_id = shader.map(|shader| shader.shader_id).unwrap_or(0);
    if shader_id == 0 {
        let _ = state::with_state(|ui| ui.edge_setup_failed = true);
        return None;
    }
    let Ok(list_id) = spring::gfx::create_list(SyncCallback::new(CALLBACK_LIST, 0)) else {
        let _ = state::with_state(|ui| ui.edge_setup_failed = true);
        return None;
    };
    let _ = state::with_state(|ui| {
        ui.edge_shader = shader_id;
        ui.edge_list = list_id;
    });
    Some((shader_id, list_id))
}

fn emit_list() {
    let _ = spring::gfx::depth_test(spring::gfx::GfxDepthTestOptions {
        enable: true,
        set_func: true,
        func: GL_LEQUAL,
    });
    let _ = spring::gfx::bind_texture(GROUND_TEXTURE, 0, true);
    let _ = spring::gfx::begin_end(GL_TRIANGLE_STRIP, SyncCallback::new(CALLBACK_VERTICES, 0));
    let _ = spring::gfx::color(1.0, 1.0, 1.0, 1.0);
}

fn emit_vertices() {
    let (width, height) = map_size();
    let _ = spring::gfx::color(1.0, 1.0, 1.0, 1.0);
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
            emit_pair(previous_x, current_x, z, x, next_x, width, height);
            if (step > 0.0 && z >= to) || (step < 0.0 && z <= to) {
                break;
            }
            z += step;
        }
        x = next_x;
    }
}

fn emit_pair(
    map_x0: f32,
    map_x1: f32,
    z: f32,
    vertex_x0: f32,
    vertex_x1: f32,
    width: f32,
    height: f32,
) {
    let _ = spring::gfx::tex_coord(map_x0 / width, z / height, 0.0, 0.0, 2);
    let _ = spring::gfx::vertex(vertex_x0, ground(map_x0, z), z, 1.0, 1);
    let _ = spring::gfx::tex_coord(map_x1 / width, z / height, 0.0, 0.0, 2);
    let _ = spring::gfx::vertex(vertex_x1, ground(map_x1, z), z, 1.0, 1);
}

fn ground(x: f32, z: f32) -> f32 {
    spring::get_ground_height(x, z).unwrap_or(0.0)
}

fn map_size() -> (f32, f32) {
    spring::game::get_game_map_info(0)
        .map(|map| (map.map_size_x as f32, map.map_size_z as f32))
        .unwrap_or((0.0, 0.0))
}

fn translate(x: f32, z: f32) {
    let _ = spring::gfx::translate(x, 0.0, z);
}

fn call(list_id: u32) {
    let _ = spring::gfx::call_list(list_id);
}
