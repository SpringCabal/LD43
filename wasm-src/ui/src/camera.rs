//! The close third-person camera that follows the blood mage, as
//! camera_control.lua's play mode sets it.

use spring::Float3;
use springcabal_control::camera::{Aim, Follow, View};

const START_Y: f32 = 436.300_78;
const ROTATION_X: f32 = 2.5;
const START_VIEW: View = View::spring(1000.0, ROTATION_X)
    .pitch(-0.828_376_8, -0.560_171_2)
    .height(START_Y)
    .angle(ROTATION_X);
const START_POSITION: Float3 = Float3::new(5580.0, START_Y, 7100.0);

/// Locked on the blood mage's mid position, as the engine's unit tracker
/// follows it (drawMidPos, not the footprint origin, which keeps the camera
/// on scaled units). The tracker is armed too, as in the Lua build, and the
/// target is still set every update: camera or minimap interaction can
/// switch the tracker off.
pub const FOLLOW: Follow = Follow::target(0.0)
    .start(START_VIEW, START_POSITION)
    .aim(Aim::MidPosition)
    .engine_tracker();
