use crate::state;

use super::bars::{self, BarStack};
use super::geometry;
use super::shader;

pub fn draw(seconds: f32) {
    let Some((shader_id, list_id)) = resources() else {
        return;
    };
    let camera = match spring::camera::get_camera_position(0) {
        Ok(camera) => camera,
        Err(_) => return,
    };
    let smooth = spring::terrain::get_smooth_mesh_height(camera.x, camera.z).unwrap_or(0.0);
    let height = camera.y - smooth;
    if height * height >= MAX_UNIT_DISTANCE {
        return;
    }
    let units = spring::unit_rendering::get_visible_units(-1, 0.0, false).unwrap_or_default();
    if units.is_empty() {
        return;
    }
    let blink = (seconds % 1.0) < 0.5;
    let frame = state::with_state(|ui| ui.game_frame).unwrap_or(0);
    begin(shader_id);
    for unit_id in units {
        draw_unit(unit_id, camera, list_id, blink, frame);
    }
    end();
}

const MAX_UNIT_DISTANCE: f32 = 11_000_000.0;
const MINIMUM_RELOAD_TIME: f32 = 4.0;
const HEIGHT_SCALE: f32 = 1.8;
const TEX_UNIT_COLOR: i32 = 1;
const TEX_UNIT_PROGRESS: i32 = 2;

fn resources() -> Option<(u32, u32)> {
    let existing = state::with_state(|ui| (ui.bar_shader, ui.bar_list)).unwrap_or((0, 0));
    if existing.0 != 0 && existing.1 != 0 {
        return Some(existing);
    }
    if state::with_state(|ui| ui.bar_setup_failed).unwrap_or(true) {
        return None;
    }
    let Some(shader_id) = shader::create() else {
        let _ = state::with_state(|ui| ui.bar_setup_failed = true);
        return None;
    };
    let Ok(list_id) = spring::gfx::create_list(geometry::emit_bar) else {
        let _ = state::with_state(|ui| ui.bar_setup_failed = true);
        return None;
    };
    let _ = state::with_state(|ui| {
        ui.bar_shader = shader_id;
        ui.bar_list = list_id;
    });
    Some((shader_id, list_id))
}

fn begin(shader_id: u32) {
    let _ = spring::gfx::depth_test(spring::gfx::GfxDepthTestOptions::default());
    let _ = spring::gfx::depth_mask(true);
    let _ = spring::gfx::use_shader(shader_id);
    let _ = spring::gfx::fog_coord(0.0);
}

fn end() {
    let _ = spring::gfx::use_shader(0);
    let _ = spring::gfx::depth_mask(false);
    let _ = spring::gfx::color(1.0, 1.0, 1.0, 1.0);
}

fn draw_unit(unit_id: i32, camera: spring::types::Float3, list_id: u32, blink: bool, frame: i32) {
    let Ok(position) = spring::units_info::get_unit_position(
        unit_id,
        spring::units_info::GetUnitPositionOptions {
            mid_pos: false,
            aim_pos: false,
        },
    ) else {
        return;
    };
    let dx = position.x - camera.x;
    let dy = position.y - camera.y;
    let dz = position.z - camera.z;
    if dx * dx + dy * dy + dz * dz > MAX_UNIT_DISTANCE {
        return;
    }
    let stack = collect(unit_id, blink, frame);
    if stack.is_empty() {
        return;
    }
    let radius = spring::units_info::get_unit_radius(unit_id).unwrap_or(0.0);
    let _ = spring::gfx::push_matrix(0);
    let _ = spring::gfx::translate(position.x, position.y + HEIGHT_SCALE * radius, position.z);
    let _ = spring::gfx::billboard(0);
    for (offset, bar) in stack.iter() {
        let _ = spring::gfx::multi_tex_coord(
            TEX_UNIT_COLOR,
            bar.color[0],
            bar.color[1],
            bar.color[2],
            bar.color[3],
            1,
        );
        let _ = spring::gfx::multi_tex_coord(TEX_UNIT_PROGRESS, bar.progress, offset, 0.0, 0.0, 1);
        let _ = spring::gfx::call_list(list_id);
    }
    let _ = spring::gfx::pop_matrix(0);
}

fn collect(unit_id: i32, blink: bool, frame: i32) -> BarStack {
    let mut stack = BarStack::new();
    let Ok(health) = spring::units_info::get_unit_health(unit_id) else {
        return stack;
    };
    let max_health = health.max_health.max(1.0);
    let build = health.build_progress;
    let fraction = health.health / max_health;
    let paralyze = health.paralyze_damage / max_health;
    if build >= 1.0 || build - fraction >= 0.01 {
        stack.push(fraction, bars::health_color(fraction));
    }
    if build < 1.0 {
        stack.push(build, bars::BUILD);
    }
    if paralyze > 0.01 && fraction > 0.01 && paralyze < 1e8 {
        let stunned = spring::units_info::get_unit_is_stunned(unit_id).unwrap_or(false);
        let color = if stunned {
            if blink {
                bars::EMP_BLINK
            } else {
                bars::EMP_PAUSED
            }
        } else {
            bars::EMP
        };
        let progress = if stunned { 1.0 } else { paralyze.min(1.0) };
        stack.push(progress, color);
    }
    if health.capture_progress > 0.0 {
        stack.push(health.capture_progress, bars::CAPTURE);
    }
    push_reload(&mut stack, unit_id, frame);
    stack
}

fn push_reload(stack: &mut BarStack, unit_id: i32, frame: i32) {
    let Ok(state) = spring::units_weapons::get_unit_weapon_state(unit_id, 1, "reloadTime") else {
        return;
    };
    let reload_time = state.reload_time;
    if reload_time < MINIMUM_RELOAD_TIME {
        return;
    }
    let reload_frame = state.reload_frame;
    if reload_frame <= frame as f32 {
        return;
    }
    let progress = 1.0 - ((reload_frame - frame as f32) / 30.0) / reload_time;
    stack.push(progress.max(0.0), bars::RELOAD);
}
