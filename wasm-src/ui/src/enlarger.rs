use alloc::vec::Vec;
use core::cell::{Cell, RefCell};

thread_local! {
    static SWEEP_TICKS: Cell<u32> = const { Cell::new(0) };
    static SEEN_UNITS: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
}

pub fn reset() {
    SWEEP_TICKS.with(|ticks| ticks.set(0));
    SEEN_UNITS.with_borrow_mut(Vec::clear);
}

pub fn arm(unit_id: i32) {
    if scale(unit_id).is_none() {
        return;
    }
    // The UI side has no UnitCreated callback. New waves (including the
    // leader) therefore need to be picked up by the next sweep as well.
    let _ = spring::unsynced_ctrl::set_unit_lua_draw(unit_id, true);
}

pub fn sweep() {
    let should_scan = SWEEP_TICKS.with(|ticks| {
        let tick = ticks.get();
        ticks.set(tick.wrapping_add(1));
        // Unit creation is infrequent in this game; a delayed discovery is
        // preferable to putting a native unit-list query in the hot path.
        tick.is_multiple_of(300)
    });
    if !should_scan {
        return;
    }
    let units = spring::get_all_units().unwrap_or_default();
    if units.is_empty() {
        return;
    }
    SEEN_UNITS.with_borrow_mut(|seen| {
        // Unit IDs can be recycled, so forget units which no longer exist.
        seen.retain(|seen_id| units.contains(seen_id));
        for unit_id in units.iter().copied() {
            if seen.contains(&unit_id) {
                continue;
            }
            arm(unit_id);
            seen.push(unit_id);
        }
    });
}

/// Draw scales come straight from the unit def's custom params, the same place
/// the Lua build reads them from.
fn scale(unit_id: i32) -> Option<(f32, f32)> {
    let def_id: i32 = spring::get_unit_def_id(unit_id).ok()?.into();
    let horizontal = custom_scale(def_id, "hscale");
    let vertical = custom_scale(def_id, "vscale");
    if horizontal.is_none() && vertical.is_none() {
        return None;
    }
    Some((horizontal.unwrap_or(1.0), vertical.unwrap_or(1.0)))
}

fn custom_scale(def_id: i32, key: &str) -> Option<f32> {
    spring::get_unit_def_custom_param(def_id, key)
        .ok()?
        .trim()
        .parse()
        .ok()
        .filter(|value: &f32| *value != 0.0)
}
