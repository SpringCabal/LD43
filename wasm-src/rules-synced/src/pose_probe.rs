//! Deterministic Lua/CUS animation conformance probe.
//!
//! This is test-only code. It invokes the same named unit-script functions
//! that the Lua reference fixture invokes, then samples the engine's piece
//! matrices. The probe deliberately observes the engine boundary rather than
//! sharing animation implementation code with the game.

use alloc::format;
use alloc::string::String;
use core::cell::Cell;

use spring::UnitId;
use spring_addons::rules::Gadget;

use crate::state::Global;

const TARGET_DEF: &str = "bloodmage";
/// The frames that start CastAnimation levels 1, 2 and 3.
const TRIGGER_FRAMES: &[i32] = &[30, 90, 145];
const LAST_FRAME: i32 = 195;
const SAMPLE_FRAMES: &[i32] = &[
    20, 29, 31, 33, 35, 39, 42, 45, 50, 55, 60, 70, 80, 91, 93, 95, 98, 100, 103, 106, 110, 115,
    120, 130, 146, 148, 150, 155, 160, 163, 168, 173, 180, 190, 195,
];
const POSE_PIECES: &[&str] = &["Torso", "Hand_Right", "Hand_Left"];

#[derive(Default)]
pub struct PoseProbeGadget {
    target: Cell<i32>,
    finished: Cell<bool>,
    skeleton_emitted: Cell<bool>,
}

impl PoseProbeGadget {
    pub fn new() -> Self {
        Self {
            target: Cell::new(-1),
            finished: Cell::new(false),
            skeleton_emitted: Cell::new(false),
        }
    }

    fn find_target(&self) -> Option<i32> {
        let target_def = spring::get_unit_def_id_by_name(TARGET_DEF).ok()?;
        let units = spring::get_all_units().ok()?;
        units.into_iter().find(|unit| {
            spring::get_unit_def_id(*unit)
                .map(|def| def.0 == target_def)
                .unwrap_or(false)
        })
    }

    fn emit_skeleton(&self, unit: i32) {
        if self.skeleton_emitted.replace(true) {
            return;
        }
        let Ok(map) = spring::get_unit_piece_map(unit) else {
            let _ = spring::echo("[UnitScriptProbe] ERROR no-piece-map", "");
            return;
        };
        for entry in map {
            let _ = spring::echo(
                &format!(
                    "[UnitScriptProbe] map piece={} num={}",
                    entry.name, entry.piece_num
                ),
                "",
            );
            if let Ok(info) =
                spring::owned::units_pieces::get_unit_piece_info(unit, entry.piece_num)
                && info.exists
            {
                let offset = info.info.offset;
                let _ = spring::echo(
                    &format!(
                        "[UnitScriptProbe] info piece={} offset={:.9},{:.9},{:.9}",
                        entry.name, offset.x, offset.y, offset.z
                    ),
                    "",
                );
            }
        }
    }

    fn emit_matrix(&self, frame: i32, unit: i32, name: &str, piece: i32) {
        let Ok(matrix) = spring::owned::units_pieces::get_unit_piece_matrix(unit, piece) else {
            let _ = spring::echo(
                &format!("[UnitScriptProbe] ERROR frame={frame} matrix-piece={name}"),
                "",
            );
            return;
        };
        if matrix.m.len() != 16 {
            let _ = spring::echo(
                &format!("[UnitScriptProbe] ERROR frame={frame} matrix-piece={name}"),
                "",
            );
            return;
        }
        let mut values = String::new();
        for (index, value) in matrix.m.iter().enumerate() {
            if index != 0 {
                values.push(',');
            }
            values.push_str(&format!("{value:.6}"));
        }
        let _ = spring::echo(
            &format!("[UnitScriptProbe] frame={frame} piece={name} m={values}"),
            "",
        );
    }

    fn sample(&self, frame: i32, unit: i32) {
        let Ok(map) = spring::get_unit_piece_map(unit) else {
            let _ = spring::echo(
                &format!("[UnitScriptProbe] ERROR frame={frame} no-piece-map"),
                "",
            );
            return;
        };
        let mut count = 0;
        for name in POSE_PIECES {
            let Some(entry) = map.iter().find(|entry| entry.name == *name) else {
                let _ = spring::echo(
                    &format!("[UnitScriptProbe] ERROR frame={frame} missing-piece={name}"),
                    "",
                );
                continue;
            };
            self.emit_matrix(frame, unit, name, entry.piece_num);
            count += 1;
        }
        let _ = spring::echo(
            &format!("[UnitScriptProbe] SAMPLE_DONE frame={frame} pieces={count}"),
            "",
        );
    }

    /// Starts CastAnimation on the target, as the Lua probe does from its
    /// GameFrame: level 1 at the first trigger frame, 2 and 3 at the others.
    fn trigger(&self, frame: i32, unit: i32, level: i32) {
        let (tx, tz) = if level == 1 {
            (0.0, 0.0)
        } else {
            (6000.0, 6000.0)
        };
        let call = crate::unit_scripts::ScriptCall::CastAnimation { level, tx, tz };
        let handled = crate::with_cus_module(|host| {
            host.with_script(UnitId(unit), |ctx, script| script.handle(ctx, call))
                .is_some()
        })
        .unwrap_or(false);
        let _ = spring::echo(
            &format!("[UnitScriptProbe] cus trigger unit={unit} level={level} handled={handled}"),
            "",
        );
        let _ = spring::echo(
            &format!("[UnitScriptProbe] trigger frame={frame} cast=true"),
            "",
        );
    }
}

impl Gadget<Global> for PoseProbeGadget {
    fn name(&self) -> &'static str {
        "TTLD Pose Probe"
    }

    fn game_frame(&self, _ctx: &spring_addons::AddonContext<'_, Global>, frame: i32) {
        if self.finished.get() {
            return;
        }
        let unit = if self.target.get() >= 0 {
            Some(self.target.get())
        } else {
            let target = self.find_target();
            if let Some(target) = target {
                self.target.set(target);
            }
            target
        };
        let Some(unit) = unit else {
            return;
        };

        self.emit_skeleton(unit);
        if let Some(index) = TRIGGER_FRAMES.iter().position(|trigger| *trigger == frame) {
            self.trigger(frame, unit, index as i32 + 1);
        }
        if SAMPLE_FRAMES.contains(&frame) {
            self.sample(frame, unit);
        }
        if frame >= LAST_FRAME {
            let _ = spring::echo(
                &format!("[UnitScriptProbe] DONE unit={unit} last_frame={LAST_FRAME}"),
                "",
            );
            self.finished.set(true);
        }
    }
}
