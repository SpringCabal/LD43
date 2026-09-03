use crate::state;

pub fn apply() {
    if state::with_state(|ui| ui.camera_initialized).unwrap_or(true) {
        return;
    }
    let camera = spring::CameraState {
        name: "spring",
        position: spring::types::Float3 {
            x: START_X,
            y: START_Y,
            z: START_Z,
        },
        direction: spring::types::Float3 {
            x: 0.0,
            y: DIRECTION_Y,
            z: DIRECTION_Z,
        },
        up: spring::types::Float3 {
            x: 0.0,
            y: -DIRECTION_Z,
            z: DIRECTION_Y,
        },
        right: spring::types::Float3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        fov: FIELD_OF_VIEW,
        rx: ROTATION_X,
        ry: 0.0,
        rz: 0.0,
        dist: DISTANCE,
        height: START_Y,
        angle: ROTATION_X,
        mode: CAMERA_MODE,
    };
    if spring::set_camera_state(camera, 0.0, 1.0, 1.0).is_ok() {
        let _ = state::with_state(|ui| ui.camera_initialized = true);
    }
}

pub fn track() {
    let unit_id = state::with_state(|ui| {
        let unit_id = ui.player_unit;
        if unit_id < 0 || ui.tracked_unit == unit_id {
            return None;
        }
        ui.tracked_unit = unit_id;
        Some(unit_id)
    })
    .flatten();
    let Some(unit_id) = unit_id else {
        return;
    };
    let _ = spring::selection::select_unit_array(&[unit_id], false);
    let _ = spring::send_commands("trackoff", "");
    let _ = spring::send_commands("track", "");
    let _ = spring::send_commands("trackmode 1", "");
    let _ = spring::selection::select_unit_array(&[], false);
}

const START_X: f32 = 5580.0;
const START_Y: f32 = 436.300_78;
const START_Z: f32 = 7100.0;
const DIRECTION_Y: f32 = -0.828_376_8;
const DIRECTION_Z: f32 = -0.560_171_2;
const FIELD_OF_VIEW: f32 = 45.0;
const DISTANCE: f32 = 1000.0;
const ROTATION_X: f32 = 2.5;
const CAMERA_MODE: i32 = 2;
