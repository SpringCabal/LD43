use spring_addons::ui::Widget;
use springcabal::message::Encoder;

use crate::hud;
use crate::state::Global;

/// The updates the visual suite takes its screenshots on, with
/// [`springcabal::shots::Screenshots`] counting the same updates as this
/// widget. The suite's goldens are the first three; the first two also
/// feed its leader-render comparison.
/// - 11, 22: the leader-render baseline, then with its fixture spawned (at
///   12; SetUnitLuaDraw needs an update to switch to the legacy drawer);
/// - 26, 28, 30: Heartburn's fireball in flight;
/// - 37, 42: hovering the ork leader, then a big ork;
/// - 44: after the engine selection was cleared;
/// - 55, 65, 70, 75: the Poultice throw;
/// - 90, 120, 240, 330: the camera following the mage to the map edge.
pub const SCREENSHOTS: [u32; 16] = [
    11, 22, 26, 28, 30, 37, 42, 44, 55, 65, 70, 75, 90, 120, 240, 330,
];

#[derive(Default)]
pub struct DbgE2eWidget {
    ticks: core::cell::Cell<u32>,
    poultice_target: core::cell::Cell<i32>,
    poultice_x: core::cell::Cell<f32>,
    poultice_z: core::cell::Cell<f32>,
}

impl DbgE2eWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Widget<Global> for DbgE2eWidget {
    fn name(&self) -> &'static str {
        "DbgE2e"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let tick = self.ticks.get();
        self.ticks.set(tick + 1);
        match tick {
            10 => hud::close_intro(ctx.global()),
            12 => {
                // Spawn a stationary boss/normal-ork pair for the pixel-level
                // render regression. This uses the regular synced Lua message
                // path and never synthesizes F12 or Pause.
                let _ = spring::send_lua_rules_msg("e2e|leader-render");
            }
            20 => {
                report_leader_fixture();
            }
            // Exercise the real synced spell path in the visual run. This
            // uses the same rules message as the key/mouse input path and
            // does not synthesize F12 or Pause.
            25 => send_spell(2, 1000.0),
            // Exercise hover-only target UI and the unit outline with a real
            // engine mouse move. This intentionally does not synthesize F12
            // or Pause.
            35 => hover_fixture("orkboss", "orkboss"),
            39 => hover_fixture("orkbig", "orkbig"),
            // Force a real engine selection, then let the normal input widget
            // clear it on the next update (no test-only clearing).
            43 => {
                let _ = spring::selection::select_unit_array(&[player_unit()], false);
                let count = spring::selection::get_selected_units_count(0).unwrap_or(0);
                let _ = spring::echo(&alloc::format!("TTLD E2E selection seeded={count}"), "");
            }
            44 => {
                let count = spring::selection::get_selected_units_count(0).unwrap_or(u32::MAX);
                let _ = spring::echo(&alloc::format!("TTLD E2E selection cleared={count}"), "");
            }
            45 => report_wheel_capture(),
            50 => {
                report_tracking("before_move");
                // Drive the player toward the north map edge as well as east.
                // This exercises the minimap's edge behavior instead of only
                // testing the centered portion of the viewport.
                move_player(500.0, -7000.0);
            }
            55 => self.remember_poultice_target(),
            60 => send_spell(5, 0.0),
            // Sample shortly after the delayed impulse, while the target is
            // still expected to be queryable and airborne.
            65 | 75 => self.report_poultice(),
            70 => {
                report_tracking("after_move");
                self.report_poultice();
            }
            240 => report_tracking("north_edge"),
            330 => report_tracking("north_edge_late"),
            _ => {}
        }
    }
}

fn player_unit() -> i32 {
    crate::with_global(|global| global.status.access().player_unit)
}

fn report_leader_fixture() {
    let player_id = player_unit();
    let Some(player) = unit_position(player_id) else {
        return;
    };
    let leader_def = spring::get_unit_def_id_by_name("orkboss").ok();
    let orkbig_def = spring::get_unit_def_id_by_name("orkbig").ok();
    let mut leader = None;
    let mut orkbig = None;
    for unit_id in spring::get_all_units().unwrap_or_default() {
        let Ok(def_id) = spring::units_info::get_unit_def_id(unit_id) else {
            continue;
        };
        let Some(position) = unit_position(unit_id) else {
            continue;
        };
        let dx = position.x - player.x;
        let dz = position.z - player.z;
        if dx * dx + dz * dz > 700.0 * 700.0 {
            continue;
        }
        let Ok(screen) = spring::camera::world_to_screen_coords(position) else {
            continue;
        };
        if Some(def_id) == leader_def {
            leader = Some((unit_id, screen.screen_pos.x, screen.screen_pos.y));
        } else if Some(def_id) == orkbig_def {
            orkbig = Some((unit_id, screen.screen_pos.x, screen.screen_pos.y));
        }
    }
    let _ = spring::echo(
        &alloc::format!("TTLD E2E leader render pixels: leader={leader:?} orkbig={orkbig:?}"),
        "",
    );
}

fn unit_position(unit_id: i32) -> Option<spring::types::Float3> {
    (unit_id >= 0)
        .then(|| {
            spring::units_info::get_unit_position(
                unit_id,
                spring::units_info::GetUnitPositionOptions {
                    mid_pos: false,
                    aim_pos: false,
                },
            )
        })?
        .ok()
}

fn send_spell(index: i32, z_offset: f32) {
    let unit_id = player_unit();
    if let Ok(position) = spring::units_info::get_unit_position(
        unit_id,
        spring::units_info::GetUnitPositionOptions {
            mid_pos: false,
            aim_pos: false,
        },
    ) {
        let message = Encoder::new("spell")
            .field(index)
            .field(position.x)
            .field(position.y)
            .field(position.z + z_offset)
            .finish();
        let accepted = spring::send_lua_rules_msg(&message).unwrap_or(false);
        let name = if index == 5 { "poultice" } else { "heartburn" };
        let _ = spring::echo(&alloc::format!("TTLD E2E {name} sent: {accepted}"), "");
    } else {
        let _ = spring::echo(
            &alloc::format!("TTLD E2E spell {index} skipped: no player unit"),
            "",
        );
    }
}

fn move_player(x_offset: f32, z_offset: f32) {
    let unit_id = player_unit();
    let Ok(position) = spring::units_info::get_unit_position(
        unit_id,
        spring::units_info::GetUnitPositionOptions {
            mid_pos: false,
            aim_pos: false,
        },
    ) else {
        return;
    };
    let message = Encoder::new("movement")
        .field(position.x + x_offset)
        .field(position.z + z_offset)
        .finish();
    let _ = spring::send_lua_rules_msg(&message);
}

fn report_tracking(label: &str) {
    let unit_id = player_unit();
    let Ok(unit) = spring::units_info::get_unit_position(
        unit_id,
        spring::units_info::GetUnitPositionOptions {
            mid_pos: true,
            aim_pos: false,
        },
    ) else {
        return;
    };
    let Ok(camera) = spring::camera::get_camera_state(true) else {
        return;
    };
    let dx = camera.pos.x - unit.x;
    let dz = camera.pos.z - unit.z;
    let error = (dx * dx + dz * dz).sqrt();
    let _ = spring::echo(
        &alloc::format!(
            "TTLD E2E tracking {label}: unit=({:.0},{:.0}) camera=({:.0},{:.0}) error={error:.1}",
            unit.x,
            unit.z,
            camera.pos.x,
            camera.pos.z
        ),
        "",
    );
}

fn report_wheel_capture() {
    let Ok(before) = spring::camera::get_camera_state(true) else {
        return;
    };
    let accepted = spring::debug_input::emulate_mouse_wheel(1.0).is_ok();
    let Ok(after) = spring::camera::get_camera_state(true) else {
        return;
    };
    let delta = after.dist - before.dist;
    let captured = delta.abs() < 0.01;
    let _ = spring::echo(
        &alloc::format!(
            "TTLD E2E mouse wheel accepted={accepted} before_dist={:.2} after_dist={:.2} delta={delta:.2} captured={captured}",
            before.dist,
            after.dist,
        ),
        "",
    );
}

impl DbgE2eWidget {
    fn remember_poultice_target(&self) {
        let unit_id = player_unit();
        let Ok(origin) = spring::units_info::get_unit_position(
            unit_id,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: false,
                aim_pos: false,
            },
        ) else {
            return;
        };
        let mut output = [0i32; 512];
        let Ok(spring::BufferFill::Complete(count)) =
            spring::get_units_in_cylinder_into(origin.x, origin.z, 720.0, 1, &mut output)
        else {
            return;
        };
        for target in output.into_iter().take(count) {
            let Ok(team) = spring::units_info::get_unit_team(target) else {
                continue;
            };
            let Ok(def) = spring::units_info::get_unit_def_id(target) else {
                continue;
            };
            // The native query is team-filtered, but retain the check here so
            // the visual assertion cannot accidentally observe a stale or
            // misclassified unit if the query contract changes.
            if team != 1 {
                continue;
            }
            let Ok(position) = spring::units_info::get_unit_position(
                target,
                spring::units_info::GetUnitPositionOptions {
                    mid_pos: false,
                    aim_pos: false,
                },
            ) else {
                continue;
            };
            self.poultice_target.set(target);
            self.poultice_x.set(position.x);
            self.poultice_z.set(position.z);
            let _ = spring::echo(
                &alloc::format!(
                    "TTLD E2E poultice target={target} team={} def={} before=({:.0},{:.0})",
                    team,
                    def,
                    position.x,
                    position.z
                ),
                "",
            );
            return;
        }
    }

    fn report_poultice(&self) {
        let target = self.poultice_target.get();
        if target < 0 {
            let _ = spring::echo("TTLD E2E poultice target missing", "");
            return;
        }
        let position = match spring::units_info::get_unit_position(
            target,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: false,
                aim_pos: false,
            },
        ) {
            Ok(position) => position,
            Err(_) => {
                let _ = spring::echo(
                    &alloc::format!("TTLD E2E poultice target={target} no longer queryable"),
                    "",
                );
                return;
            }
        };
        let dx = position.x - self.poultice_x.get();
        let dz = position.z - self.poultice_z.get();
        let distance = (dx * dx + dz * dz).sqrt();
        let velocity = spring::units_info::get_unit_velocity(target).unwrap_or_default();
        let _ = spring::echo(
            &alloc::format!(
                "TTLD E2E poultice target={target} after=({:.0},{:.0}) moved={distance:.1} velocity=({:.1},{:.1},{:.1})",
                position.x,
                position.z,
                velocity.x,
                velocity.y,
                velocity.z
            ),
            "",
        );
    }
}

fn hover_fixture(label: &str, def_name: &str) {
    let Some(def_id) = spring::get_unit_def_id_by_name(def_name).ok() else {
        let _ = spring::echo(
            &alloc::format!("TTLD E2E hover fixture={label} skipped: no def"),
            "",
        );
        return;
    };
    let Some(player) = Some(player_unit()).filter(|unit_id| *unit_id >= 0) else {
        return;
    };
    let Some(player_position) = unit_position(player) else {
        return;
    };
    let mut target = None;
    for unit_id in spring::get_all_units().unwrap_or_default() {
        if spring::units_info::get_unit_def_id(unit_id).ok() != Some(def_id) {
            continue;
        }
        let Some(position) = unit_position(unit_id) else {
            continue;
        };
        let dx = position.x - player_position.x;
        let dz = position.z - player_position.z;
        if dx * dx + dz * dz <= 700.0 * 700.0 {
            target = Some((unit_id, position));
        }
    }
    let Some((unit_id, position)) = target else {
        let _ = spring::echo(
            &alloc::format!("TTLD E2E hover fixture={label} skipped: no unit"),
            "",
        );
        return;
    };
    let position = spring::types::Float3 {
        x: position.x,
        y: position.y + 40.0,
        z: position.z,
    };
    let Ok(screen) = spring::camera::world_to_screen_coords(position) else {
        return;
    };
    let height = crate::with_global(|global| global.view.get().1);
    if height <= 1 {
        return;
    }
    let x = screen.screen_pos.x.round() as i32;
    let bottom_y = screen.screen_pos.y.round() as i32;
    let mut hover = None;
    'search: for dx in (-100..=100).step_by(10) {
        for dy in (-100..=100).step_by(10) {
            if trace_unit(x + dx, bottom_y + dy) == Some(unit_id) {
                hover = Some((x + dx, bottom_y + dy));
                break 'search;
            }
        }
    }
    let Some((hover_x, hover_bottom_y)) = hover else {
        let _ = spring::echo(
            &alloc::format!("TTLD E2E hover fixture={label} unit={unit_id} skipped: trace miss"),
            "",
        );
        return;
    };
    let hover_top_y = height - 1 - hover_bottom_y;
    let result = spring::debug_input::emulate_mouse_move(hover_x, hover_top_y);
    let hit = trace_hit(hover_x, hover_bottom_y);
    let _ = spring::echo(
        &alloc::format!(
            "TTLD E2E hover fixture={label} unit={unit_id} mouse=({hover_x},{hover_bottom_y}) trace={hit} move_ok={}",
            result.is_ok(),
        ),
        "",
    );
}

fn trace_unit(x: i32, y: i32) -> Option<i32> {
    crate::cursor::unit_at(x as f32, y as f32)
}

fn trace_hit(x: i32, y: i32) -> alloc::string::String {
    spring::camera::trace_screen_ray(
        x as f32,
        y as f32,
        spring::camera::TraceScreenRayOptions {
            only_coords: false,
            use_minimap: false,
            include_sky: false,
            ignore_water: false,
            height_offset: 0.0,
        },
    )
    .ok()
    .map(|trace| alloc::format!("{}:{}", trace.hit_type, trace.hit_id))
    .unwrap_or_else(|| "none".into())
}
