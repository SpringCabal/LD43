use core::cell::Cell;

use spring::ResultExt;
use spring_addons::ui::Widget;
use springcabal::shots::screenshot;
use ttld_shared::Command;

use crate::hud;
use crate::state::Global;

/// Logged after the last step; the visual suite quits on it.
pub const DONE_MARKER: &str = "TTLD E2E done";

/// What the visual run does, by game frame. The suite's goldens are the
/// first three screenshots, which are taken with the game paused on their
/// frame (`pauseatframe`), so they show the same frame every run; the first
/// two also feed its leader-render comparison.
const STEPS: &[(i32, When, Step)] = &[
    (40, When::Running, Step::CloseIntro),
    (73, When::Paused, Step::Shot("leader-render baseline")),
    // Spawns the stationary boss/normal-ork pair for the pixel-level render
    // comparison, through the regular synced message path. The rules
    // messages up to the fireball shot are sent while paused, so the rules
    // take them on the same frame every run.
    (78, When::Paused, Step::LeaderFixture),
    (121, When::Running, Step::ReportLeaderFixture),
    (132, When::Paused, Step::Shot("leader-render fixture")),
    // Heartburn through the same rules message as the key/mouse input.
    (150, When::Paused, Step::Heartburn),
    (167, When::Paused, Step::Shot("heartburn fireball")),
    (189, When::Running, Step::Shot("heartburn landing")),
    // Hover-only target UI and the unit outline, with real engine mouse
    // moves.
    (227, When::Running, Step::Hover("orkboss")),
    (233, When::Running, Step::CheckHover("orkboss")),
    (240, When::Running, Step::Shot("hovering the ork leader")),
    (254, When::Running, Step::Hover("orkbig")),
    (260, When::Running, Step::CheckHover("orkbig")),
    (274, When::Running, Step::Shot("hovering a big ork")),
    // A real engine selection, which the input widget clears on its next
    // update.
    (281, When::Running, Step::SeedSelection),
    (288, When::Running, Step::CheckSelection),
    (288, When::Running, Step::Shot("selection cleared")),
    (303, When::Running, Step::MouseWheel),
    // Drive the player toward the north map edge as well as east, which
    // exercises the minimap's edge behaviour.
    (339, When::Running, Step::MovePlayer),
    (376, When::Running, Step::RememberPoultice),
    (376, When::Running, Step::Shot("before poultice")),
    (418, When::Running, Step::Poultice),
    (458, When::Running, Step::ReportPoultice),
    (458, When::Running, Step::Shot("poultice throw")),
    (498, When::Running, Step::Track("after_move")),
    (498, When::Running, Step::ReportPoultice),
    (498, When::Running, Step::Shot("poultice flight")),
    (538, When::Running, Step::ReportPoultice),
    (538, When::Running, Step::Shot("poultice landing")),
    (661, When::Running, Step::Shot("setting off north")),
    (900, When::Running, Step::Track("walking_north")),
    (900, When::Running, Step::Shot("walking north")),
    (1200, When::Running, Step::Track("late")),
    (1200, When::Running, Step::Shot("late")),
    (1200, When::Running, Step::Done),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum When {
    Running,
    /// With the game paused on the step's frame.
    Paused,
}

#[derive(Clone, Copy)]
enum Step {
    CloseIntro,
    LeaderFixture,
    ReportLeaderFixture,
    Heartburn,
    Hover(&'static str),
    CheckHover(&'static str),
    SeedSelection,
    CheckSelection,
    MouseWheel,
    MovePlayer,
    RememberPoultice,
    Poultice,
    ReportPoultice,
    Track(&'static str),
    Shot(&'static str),
    Done,
}

/// Updates to stay paused after a paused step, so its screenshot is drawn
/// and its rules message handled before the game goes on.
const PAUSED_UPDATES: u32 = 10;
/// Updates to wait in a pause before running a paused step.
const SETTLE_UPDATES: u32 = 2;

#[derive(Default)]
pub struct DbgE2eWidget {
    /// The next step to run.
    next: Cell<usize>,
    /// Whether the pause frames have been requested.
    pauses_requested: Cell<bool>,
    /// Updates left before the game resumes after a paused step.
    resume_in: Cell<Option<u32>>,
    /// Updates seen so far in the current pause.
    paused_updates: Cell<u32>,
    shots: Cell<u32>,
    hovered: Cell<Option<i32>>,
    poultice_target: Cell<i32>,
    poultice_x: Cell<f32>,
    poultice_z: Cell<f32>,
}

impl DbgE2eWidget {
    pub fn new() -> Self {
        Self::default()
    }

    fn request_pauses(&self) {
        if self.pauses_requested.replace(true) {
            return;
        }
        for (frame, when, _) in STEPS {
            if *when == When::Paused {
                command(&alloc::format!("pauseatframe {frame}"));
            }
        }
    }

    /// Whether the game has been paused for a few updates. The first draw
    /// after the pause arrives can still interpolate towards the next frame,
    /// which moves fast things such as the fireball.
    fn settled_in_pause(&self) -> bool {
        if !paused() {
            self.paused_updates.set(0);
            return false;
        }
        let updates = self.paused_updates.get() + 1;
        self.paused_updates.set(updates);
        updates > SETTLE_UPDATES
    }

    fn resume_when_due(&self) -> bool {
        let Some(updates) = self.resume_in.get() else {
            return false;
        };
        if updates > 0 {
            self.resume_in.set(Some(updates - 1));
        } else {
            self.resume_in.set(None);
            self.paused_updates.set(0);
            command("pause 0");
        }
        true
    }

    fn run(&self, ctx: &spring_addons::AddonContext<'_, Global>, step: Step) {
        match step {
            Step::CloseIntro => hud::close_intro(ctx.global()),
            Step::LeaderFixture => {
                spring::send_lua_rules_msg(&Command::LeaderRenderFixture.message())
                    .log_err("leader fixture");
            }
            Step::ReportLeaderFixture => report_leader_fixture(),
            Step::Heartburn => send_spell(2, 1000.0),
            Step::Hover(def_name) => self.hovered.set(hover_fixture(def_name)),
            Step::CheckHover(def_name) => report_hover(ctx.global(), def_name, self.hovered.get()),
            Step::SeedSelection => {
                let _ = spring::selection::select_unit_array(&[player_unit()], false);
                let count = spring::selection::get_selected_units_count(0).unwrap_or(0);
                echo(&alloc::format!("TTLD E2E selection seeded={count}"));
            }
            Step::CheckSelection => {
                let count = spring::selection::get_selected_units_count(0).unwrap_or(u32::MAX);
                echo(&alloc::format!("TTLD E2E selection cleared={count}"));
            }
            Step::MouseWheel => report_wheel_capture(),
            Step::MovePlayer => {
                report_tracking("before_move");
                move_player(500.0, -7000.0);
            }
            Step::RememberPoultice => self.remember_poultice_target(),
            Step::Poultice => send_spell(5, 0.0),
            Step::ReportPoultice => self.report_poultice(),
            Step::Track(label) => report_tracking(label),
            Step::Shot(label) => {
                let shot = self.shots.get() + 1;
                self.shots.set(shot);
                let frame = spring::game::current_frame();
                screenshot(&alloc::format!("{shot} ({label}) at frame {frame}"));
                report_projectiles();
            }
            Step::Done => echo(DONE_MARKER),
        }
    }
}

impl Widget<Global> for DbgE2eWidget {
    fn name(&self) -> &'static str {
        "DbgE2e"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global>, _dt: f32) {
        let frame = spring::game::current_frame();
        if frame < 0 {
            return;
        }
        self.request_pauses();
        if self.resume_when_due() {
            return;
        }
        // Steps of one frame run together; a later frame's steps wait for
        // the next update, so each sees what the previous ones did.
        let mut running = None;
        while let Some(&(due, when, step)) = STEPS.get(self.next.get()) {
            if frame < due || running.is_some_and(|running| running != due) {
                return;
            }
            running = Some(due);
            if when == When::Paused {
                if frame == due && !self.settled_in_pause() {
                    return;
                }
                if frame > due {
                    echo(&alloc::format!(
                        "TTLD E2E missed the pause at frame {due} (now {frame})"
                    ));
                } else {
                    self.resume_in.set(Some(PAUSED_UPDATES));
                }
            }
            self.run(ctx, step);
            self.next.set(self.next.get() + 1);
        }
    }
}

fn report_projectiles() {
    let projectiles =
        spring::projectiles::get_all_projectiles(spring::projectiles::GetAllProjectilesOptions {
            exclude_weapon_projectiles: false,
            exclude_piece_projectiles: true,
        })
        .unwrap_or_default();
    for projectile in projectiles {
        if let Ok(position) = spring::projectiles::get_projectile_position(projectile) {
            echo(&alloc::format!(
                "TTLD E2E projectile {projectile} at ({:.1},{:.1},{:.1})",
                position.x,
                position.y,
                position.z
            ));
        }
    }
}

fn paused() -> bool {
    spring::display::get_game_speed(0).is_ok_and(|speed| speed.paused)
}

fn command(command: &str) {
    spring::send_commands(command, "").log_err(command);
}

fn echo(line: &str) {
    spring::echo(line, "").log_err("e2e report");
}

/// Logs whether the HUD shows the unit the mouse was moved onto as its
/// target, as the hover highlight and target panel do.
fn report_hover(global: &Global, def_name: &str, unit: Option<i32>) {
    let target = global.hud.access().target.map(|(unit, _)| unit);
    let cursor = global.cursor.get().unit;
    echo(&alloc::format!(
        "TTLD E2E hover check fixture={def_name} unit={} hud_target={} cursor={}",
        unit.unwrap_or(-1),
        target.unwrap_or(-1),
        cursor.unwrap_or(-1),
    ));
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
        let message = Command::Spell {
            index,
            x: position.x,
            y: position.y,
            z: position.z + z_offset,
        }
        .message();
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
    let message = Command::Move {
        x: position.x + x_offset,
        z: position.z + z_offset,
    }
    .message();
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

fn hover_fixture(def_name: &str) -> Option<i32> {
    let label = def_name;
    let Some(def_id) = spring::get_unit_def_id_by_name(def_name).ok() else {
        let _ = spring::echo(
            &alloc::format!("TTLD E2E hover fixture={label} skipped: no def"),
            "",
        );
        return None;
    };
    let player_position = unit_position(player_unit())?;
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
        return None;
    };
    let position = spring::types::Float3 {
        x: position.x,
        y: position.y + 40.0,
        z: position.z,
    };
    let Ok(screen) = spring::camera::world_to_screen_coords(position) else {
        return None;
    };
    let height = crate::with_global(|global| global.view.get().1);
    if height <= 1 {
        return None;
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
        return None;
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
    Some(unit_id)
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
