use core::cell::Cell;

use spring::{Float3, ResultExt};
use spring_addons::UnitDestroyedEvent;
use spring_addons::ui::Widget;
use springcabal_control::camera::{Aim, CameraFollow, Follow, View};

use crate::state::Global;

pub struct GuiCameraWidget {
    camera: CameraFollow,
    round: Cell<u32>,
    player_def: Cell<Option<i32>>,
    controlled: Cell<Option<i32>>,
}

impl Default for GuiCameraWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiCameraWidget {
    pub fn new() -> Self {
        Self {
            camera: CameraFollow::new(FOLLOW),
            round: Cell::new(0),
            player_def: Cell::new(None),
            controlled: Cell::new(None),
        }
    }
}

impl Widget<Global> for GuiCameraWidget {
    fn name(&self) -> &'static str {
        "GuiCamera"
    }

    fn init(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>) {
        let spring = ctx.spring();
        self.player_def.set(
            spring
                .unit_defs()
                .get_unit_def_id_by_name("bloodmage")
                .log_err_once(spring, "resolving the camera's player definition"),
        );
        for unit in spring.units_query().get_all_units().unwrap_or_default() {
            if spring.units_info().get_unit_def_id(unit).ok() == self.player_def.get() {
                self.controlled.set(Some(unit));
            }
        }
    }

    fn unit_created(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        unit: i32,
        def: i32,
        _team: i32,
        _builder: i32,
    ) {
        if self.player_def.get() == Some(def) {
            self.controlled.set(Some(unit));
        }
    }

    fn unit_destroyed(
        &self,
        _ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>,
        event: &UnitDestroyedEvent,
    ) {
        if self.controlled.get() == Some(event.unit_id) {
            self.controlled.set(None);
        }
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, dt: f32) {
        let spring = ctx.spring();
        // Unit events arrive immediately, like the original Lua camera.
        // The HUD status packet is sent only every six simulation frames.
        let round = ctx.global().status.access().round;
        if self.round.replace(round) != round && round > 1 {
            self.camera.reset();
        }
        self.camera.update(spring, self.controlled.get(), dt);
    }
}

const START_Y: f32 = 436.300_78;
const ROTATION_X: f32 = 2.5;
const START_VIEW: View = View::spring(1000.0, ROTATION_X)
    .pitch(-0.828_376_8, -0.560_171_2)
    .height(START_Y)
    .angle(ROTATION_X);
const START_POSITION: Float3 = Float3::new(5580.0, START_Y, 7100.0);

const FOLLOW: Follow = Follow::target(0.0)
    .start(START_VIEW, START_POSITION)
    .aim(Aim::Position)
    .engine_tracker();
