//! The Core-CUS module: attaches scripts to units and dispatches the engine's
//! and the rules' calls to them.

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use core::cell::RefCell;

use spring::UnitId;
use spring::cus::core_module::{CoreCusCallResult, CoreCusModule};
use spring::cus::wasm::WasmCus;
use spring::cus::{CusHandle, CusInstance, CusRegistry, UnitScriptCall};

#[cfg(feature = "pose-probe")]
use crate::defs::{UnitKind, defs};

use super::TtldUnitScript;
use super::common::EnginePieces;
use super::kinds::{capabilities, script_kind_for_def};

/// Runs a named unit-script function now, like Lua's `env.Function()`.
///
/// The rules are compiled into this module, so they reach the scripts
/// directly; going through `call_unit_script` would re-enter the module. The
/// CUS state is only busy while a unit script runs further up the stack, and no
/// unit script ends up in rules code that calls a script, so a busy state is a
/// bug and is logged.
pub fn call(unit: i32, function_name: &str, arguments: &[f32]) {
    if unit < 0 {
        return;
    }
    if crate::with_cus_module(|module| module.call(unit, function_name, arguments)).is_none() {
        spring::log_warning!(
            LOG_SECTION,
            "{function_name} on unit {unit} dropped: the unit scripts are busy"
        );
    }
}

const LOG_SECTION: &str = "ttld-wasm";

fn instance_id(unit: UnitId) -> u32 {
    0x5454_0000u32.wrapping_add(unit.0 as u32)
}

#[derive(Default)]
pub struct TtldCusModule {
    registry: CusRegistry<TtldUnitScript>,
    handles: BTreeMap<u32, CusHandle>,
}

impl TtldCusModule {
    /// Gives the unit its def's script, if it has one.
    pub fn attach(&mut self, unit_id: i32, def: i32) {
        let Some(kind) = script_kind_for_def(def) else {
            return;
        };
        let unit = UnitId(unit_id);
        let instance = instance_id(unit);
        if self.handles.contains_key(&instance) {
            return;
        }
        let map = match spring::get_unit_piece_map(unit_id) {
            Ok(map) => map,
            Err(error) => {
                spring::log_warning!(
                    LOG_SECTION,
                    "no unit script for unit {unit_id}: piece map failed: {error:?}"
                );
                return;
            }
        };
        let resolver = EnginePieces { map };
        let host = WasmCus::new(unit, instance);
        let engine = Rc::new(RefCell::new(host.engine()));
        let instance_state = CusInstance::construct(unit, &resolver, engine);
        let handle = self.registry.attach(instance_state);
        let _ = self.registry.with(handle, |instance| {
            instance.with_state(|script| script.set_kind(kind));
        });
        self.handles.insert(instance, handle);
        if let Err(error) = WasmCus::attach(unit, instance, capabilities(kind)) {
            spring::log_warning!(
                LOG_SECTION,
                "no unit script for unit {unit_id}: attach failed: {error:?}"
            );
            self.handles.remove(&instance);
            let _ = self.registry.detach(handle);
        }
    }

    /// Runs a named function of the unit's script. Units without a script,
    /// and dead units, ignore it, as a Lua script without the function would.
    fn call(&mut self, unit_id: i32, function_name: &str, arguments: &[f32]) {
        let Some(handle) = self.handles.get(&instance_id(UnitId(unit_id))).copied() else {
            return;
        };
        self.sync_instance_frame(handle);
        let _ = self.registry.with(handle, |instance| {
            let ctx = instance.context();
            instance.with_state(|script| script.named_call(&ctx, function_name, arguments))
        });
    }

    fn sync_instance_frame(&mut self, handle: CusHandle) {
        let frame = spring::game::current_frame() as u64;
        let _ = self.registry.with(handle, |instance| instance.tick(frame));
    }

    #[cfg(feature = "pose-probe")]
    fn run_pose_trigger(&mut self, level: i32) {
        let Some(target_def) = defs().id(UnitKind::BloodMage) else {
            return;
        };
        let Ok(units) = spring::get_all_units() else {
            return;
        };
        let Some(unit) = units.into_iter().find(|unit| {
            spring::get_unit_def_id(*unit)
                .map(|def| def.0 == target_def)
                .unwrap_or(false)
        }) else {
            return;
        };
        let instance = instance_id(UnitId(unit));
        let Some(handle) = self.handles.get(&instance).copied() else {
            return;
        };
        self.sync_instance_frame(handle);
        let arguments = if level == 1 {
            [level as f32, 0.0, 0.0]
        } else {
            [level as f32, 6000.0, 6000.0]
        };
        let mut handled = false;
        let _ = self.registry.with(handle, |instance| {
            let ctx = instance.context();
            instance.with_state(|script| {
                handled = script.named_call(&ctx, "CastAnimation", &arguments);
            });
        });
        let _ = spring::echo(
            &alloc::format!(
                "[UnitScriptProbe] cus trigger unit={unit} level={level} handled={handled}"
            ),
            "",
        );
    }
}

impl CoreCusModule for TtldCusModule {
    fn cus_invoke(
        &mut self,
        instance_id: u32,
        call: u32,
        float_arguments: &[f32],
        integer_arguments: &[i32],
        result: &mut CoreCusCallResult<'_>,
    ) -> bool {
        let Some(handle) = self.handles.get(&instance_id).copied() else {
            return false;
        };
        let Some(call) = UnitScriptCall::from_u32(call) else {
            return false;
        };
        // Tasks begin synchronously, so every callback must use the current
        // frame. Idle instances can otherwise start a firing/death sleep
        // against an old clock and skip straight to the recovery pose.
        self.sync_instance_frame(handle);
        let mut call_result = spring::cus::UnitScriptCallResult::default();
        let handled = self
            .registry
            .with(handle, |instance| {
                instance.invoke(call, float_arguments, integer_arguments, &mut call_result)
            })
            .unwrap_or(false);
        if !handled {
            return false;
        }
        result.int_value = call_result.int_value;
        result.float_value = call_result.float_value;
        result.bool_value = call_result.bool_value;
        result.complete = call_result.complete;
        result.int_count = call_result.int_values.len().min(result.int_values.len());
        result.int_values[..result.int_count]
            .copy_from_slice(&call_result.int_values[..result.int_count]);
        true
    }

    fn cus_call_named(
        &mut self,
        instance_id: u32,
        function_name: &str,
        arguments: &[f32],
        _return_values: &mut [f32],
        found: &mut bool,
    ) -> Option<usize> {
        let Some(handle) = self.handles.get(&instance_id).copied() else {
            return Some(0);
        };
        self.sync_instance_frame(handle);
        let handled = self.registry.with(handle, |instance| {
            let ctx = instance.context();
            instance.with_state(|script| script.named_call(&ctx, function_name, arguments))
        })?;
        if handled {
            *found = true;
            Some(0)
        } else {
            None
        }
    }

    fn cus_tick(&mut self, frame: u32) {
        #[cfg(feature = "pose-probe")]
        if let Some(level) = crate::pose_probe::cast_level_at(frame as i32) {
            self.run_pose_trigger(level);
        }
        self.registry.tick(frame as u64);
    }

    fn cus_detach(&mut self, instance_id: u32) {
        if let Some(handle) = self.handles.remove(&instance_id) {
            let _ = self.registry.detach(handle);
        }
    }
}
