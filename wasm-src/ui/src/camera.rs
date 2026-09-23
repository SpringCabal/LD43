use core::cell::Cell;

/// Puts the camera in its starting position, once.
pub fn apply(initialized: &Cell<bool>) {
    if initialized.get() {
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
    initialized.set(spring::set_camera_state(camera, 0.0, 1.0, 1.0).is_ok());
}

/// Follows the blood mage; `tracked` is the unit the engine tracker follows.
pub fn track(unit_id: i32, tracked: &Cell<i32>) {
    if unit_id < 0 {
        tracked.set(-1);
        return;
    }

    let Ok(target) = spring::units_info::get_unit_position(
        unit_id,
        spring::units_info::GetUnitPositionOptions {
            // The engine tracker follows drawMidPos, rather than the unit's
            // footprint origin.  Using the same point keeps native and Lua
            // camera targets aligned for scaled units.
            mid_pos: true,
            aim_pos: false,
        },
    ) else {
        return;
    };

    // `track` is stateful and camera/minimap interaction can disable it, so
    // re-arm the native tracker whenever the controlled unit changes. The
    // target is set every frame as well (Spring.SetCameraTarget), which keeps
    // working after the engine has disabled the tracker.
    if tracked.get() != unit_id {
        let _ = spring::selection::select_unit_array(&[unit_id], false);
        let _ = spring::send_commands("trackoff", "");
        let _ = spring::send_commands("track", "");
        let _ = spring::send_commands("trackmode 1", "");
        let _ = spring::selection::select_unit_array(&[], false);
        tracked.set(unit_id);
    }

    let _ = spring::camera::set_camera_target(
        target,
        spring::camera::SetCameraTargetOptions {
            transition_time: Some(0.0),
            dir_x: None,
            dir_y: None,
            dir_z: None,
        },
    );
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
