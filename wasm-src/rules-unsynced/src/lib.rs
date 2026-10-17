extern crate alloc;

use alloc::collections::BTreeMap;
use core::cell::RefCell;

use spring_addons::event::EventResult;
use spring_addons::unsynced::{UnsyncedAddon, UnsyncedHandler};
use spring_addons::{AddonContext, register_unsynced_addons};

#[derive(Default)]
struct UnitEnlarger {
    scales: RefCell<BTreeMap<i32, Option<(f32, f32)>>>,
}

impl UnitEnlarger {
    fn scale(
        &self,
        spring: &spring::Spring<'_, spring::RulesUnsynced>,
        def_id: i32,
    ) -> Option<(f32, f32)> {
        if let Some(scale) = self.scales.borrow().get(&def_id) {
            return *scale;
        }
        let scale = read_scale(spring, def_id);
        self.scales.borrow_mut().insert(def_id, scale);
        scale
    }

    fn arm(&self, spring: &spring::Spring<'_, spring::RulesUnsynced>, unit_id: i32, def_id: i32) {
        if self.scale(spring, def_id).is_some() {
            let _ = spring.unsynced_ctrl().set_unit_lua_draw(unit_id, true);
        }
    }
}

impl UnsyncedAddon for UnitEnlarger {
    fn name(&self) -> &'static str {
        "UnitEnlarger"
    }

    fn init(&self, _ctx: &AddonContext<'_, (), spring::RulesUnsynced>) {
        for unit_id in _ctx.spring().units().get_all_units().unwrap_or_default() {
            if let Ok(def_id) = _ctx.spring().units_info().get_unit_def_id(unit_id) {
                self.arm(_ctx.spring(), unit_id, def_id);
            }
        }
    }

    fn unit_created(
        &self,
        _ctx: &AddonContext<'_, (), spring::RulesUnsynced>,
        unit_id: i32,
        unit_def_id: i32,
        _unit_team: i32,
        _builder_id: i32,
    ) {
        self.arm(_ctx.spring(), unit_id, unit_def_id);
    }

    fn draw_unit(
        &self,
        _ctx: &AddonContext<'_, (), spring::RulesUnsynced>,
        unit_id: i32,
        _draw_mode: i32,
    ) -> EventResult {
        let scale = _ctx
            .spring()
            .units_info()
            .get_unit_def_id(unit_id)
            .ok()
            .and_then(|def_id| self.scale(_ctx.spring(), def_id));
        if let Some((horizontal, vertical)) = scale {
            let _ = _ctx.spring().gfx().scale(horizontal, vertical, horizontal);
        }
        EventResult::Ignored
    }
}

fn read_scale(
    spring: &spring::Spring<'_, spring::RulesUnsynced>,
    def_id: i32,
) -> Option<(f32, f32)> {
    let horizontal = custom_scale(spring, def_id, "hscale").unwrap_or(1.0);
    let vertical = custom_scale(spring, def_id, "vscale").unwrap_or(1.0);
    (horizontal != 1.0 || vertical != 1.0).then_some((horizontal, vertical))
}

fn custom_scale(
    spring: &spring::Spring<'_, spring::RulesUnsynced>,
    def_id: i32,
    key: &str,
) -> Option<f32> {
    spring
        .unit_defs()
        .get_unit_def_custom_param(def_id, key)
        .ok()?
        .trim()
        .parse()
        .ok()
        .filter(|value: &f32| *value != 0.0)
}

fn setup(handler: &mut UnsyncedHandler<()>, _spring: &spring::Spring<'_, spring::RulesUnsynced>) {
    register_unsynced_addons!(handler, [UnitEnlarger::default()]);
}

spring_addons::export_rules_unsynced_addons! {
    state: (),
    setup: setup,
    scratch: 4096,
}
