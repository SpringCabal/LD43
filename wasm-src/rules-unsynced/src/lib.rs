//! Draws units at the size their def asks for, like the unsynced half of the
//! Lua build's unit_enlarger.lua gadget.

extern crate alloc;

use alloc::collections::BTreeMap;
use core::cell::RefCell;

use spring_addons::event::EventResult;
use spring_addons::unsynced::{UnsyncedAddon, UnsyncedHandler};
use spring_addons::{AddonContext, register_unsynced_addons};

/// Scales units whose def has `hscale`/`vscale` custom params.
#[derive(Default)]
struct UnitEnlarger {
    /// Horizontal and vertical draw scale per unit def; `None` draws unscaled.
    scales: RefCell<BTreeMap<i32, Option<(f32, f32)>>>,
}

impl UnitEnlarger {
    fn scale(&self, def_id: i32) -> Option<(f32, f32)> {
        if let Some(scale) = self.scales.borrow().get(&def_id) {
            return *scale;
        }
        let scale = read_scale(def_id);
        self.scales.borrow_mut().insert(def_id, scale);
        scale
    }

    /// Scaled units need the Lua draw path, which calls DrawUnit.
    fn arm(&self, unit_id: i32, def_id: i32) {
        if self.scale(def_id).is_some() {
            let _ = spring::unsynced_ctrl::set_unit_lua_draw(unit_id, true);
        }
    }
}

impl UnsyncedAddon for UnitEnlarger {
    fn name(&self) -> &'static str {
        "UnitEnlarger"
    }

    fn init(&self, _ctx: &AddonContext<'_, ()>) {
        for unit_id in spring::get_all_units().unwrap_or_default() {
            if let Ok(def_id) = spring::get_unit_def_id(unit_id) {
                self.arm(unit_id, def_id.0);
            }
        }
    }

    fn unit_created(
        &self,
        _ctx: &AddonContext<'_, ()>,
        unit_id: i32,
        unit_def_id: i32,
        _unit_team: i32,
        _builder_id: i32,
    ) {
        self.arm(unit_id, unit_def_id);
    }

    fn draw_unit(&self, _ctx: &AddonContext<'_, ()>, unit_id: i32, _draw_mode: i32) -> EventResult {
        let scale = spring::get_unit_def_id(unit_id)
            .ok()
            .and_then(|def_id| self.scale(def_id.0));
        if let Some((horizontal, vertical)) = scale {
            let _ = spring::gfx::scale(horizontal, vertical, horizontal);
        }
        // Scaling only sets up the transform; the engine still draws the unit.
        EventResult::Ignored
    }
}

/// The def's draw scale, unless it is 1 both ways.
fn read_scale(def_id: i32) -> Option<(f32, f32)> {
    let horizontal = custom_scale(def_id, "hscale").unwrap_or(1.0);
    let vertical = custom_scale(def_id, "vscale").unwrap_or(1.0);
    (horizontal != 1.0 || vertical != 1.0).then_some((horizontal, vertical))
}

fn custom_scale(def_id: i32, key: &str) -> Option<f32> {
    spring::get_unit_def_custom_param(def_id, key)
        .ok()?
        .trim()
        .parse()
        .ok()
        .filter(|value: &f32| *value != 0.0)
}

fn setup(handler: &mut UnsyncedHandler<()>) {
    register_unsynced_addons!(handler, [UnitEnlarger::default()]);
}

spring_addons::export_rules_unsynced_addons! {
    state: (),
    setup: setup,
    scratch: 4096,
}
