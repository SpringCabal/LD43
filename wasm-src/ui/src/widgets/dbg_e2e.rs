use core::cell::Cell;

use spring::ResultExt;
use spring_addons::ui::Widget;
use springcabal::shots::screenshot;
use ttld_shared::Command;

use crate::state::Global;
use crate::widgets::gui_hud::hud;

pub const DONE_MARKER: &str = "TTLD E2E done";

const STEPS: &[(i32, When, Step)] = &[
    (40, When::Running, Step::CloseIntro),
    (73, When::Paused, Step::Shot("leader-render baseline")),
    (78, When::Paused, Step::LeaderFixture),
    (121, When::Running, Step::ReportLeaderFixture),
    (132, When::Paused, Step::Shot("leader-render fixture")),
    (150, When::Paused, Step::Heartburn),
    (167, When::Paused, Step::Shot("heartburn fireball")),
    (189, When::Running, Step::Shot("heartburn landing")),
    (227, When::Running, Step::Hover("orkboss")),
    (233, When::Running, Step::CheckHover("orkboss")),
    (240, When::Running, Step::Shot("hovering the ork leader")),
    (254, When::Running, Step::Hover("orkbig")),
    (260, When::Running, Step::CheckHover("orkbig")),
    (274, When::Running, Step::Shot("hovering a big ork")),
    (281, When::Running, Step::SeedSelection),
    (288, When::Running, Step::CheckSelection),
    (288, When::Running, Step::Shot("selection cleared")),
    (303, When::Running, Step::MouseWheel),
    (339, When::Running, Step::MovePlayer),
    (376, When::Running, Step::Shot("before poultice")),
    (418, When::Running, Step::RememberPoultice),
    (418, When::Running, Step::Poultice),
    (435, When::Running, Step::ReportPoultice),
    (440, When::Running, Step::ReportPoultice),
    (445, When::Running, Step::ReportPoultice),
    (450, When::Running, Step::ReportPoultice),
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

const PAUSED_UPDATES: u32 = 10;
const SETTLE_UPDATES: u32 = 2;

#[derive(Default)]
pub struct DbgE2eWidget {
    next: Cell<usize>,
    pauses_requested: Cell<bool>,
    resume_in: Cell<Option<u32>>,
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

    fn request_pauses(&self, spring: &spring::Spring<'_, spring::Ui>) {
        if self.pauses_requested.replace(true) {
            return;
        }
        for (frame, when, _) in STEPS {
            if *when == When::Paused {
                command(spring, &alloc::format!("pauseatframe {frame}"));
            }
        }
    }

    fn settled_in_pause(&self, spring: &spring::Spring<'_, spring::Ui>) -> bool {
        if !paused(spring) {
            self.paused_updates.set(0);
            return false;
        }
        let updates = self.paused_updates.get() + 1;
        self.paused_updates.set(updates);
        updates > SETTLE_UPDATES
    }

    fn resume_when_due(&self, spring: &spring::Spring<'_, spring::Ui>) -> bool {
        let Some(updates) = self.resume_in.get() else {
            return false;
        };
        if updates > 0 {
            self.resume_in.set(Some(updates - 1));
        } else {
            self.resume_in.set(None);
            self.paused_updates.set(0);
            command(spring, "pause 0");
        }
        true
    }

    fn run(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, step: Step) {
        let spring = ctx.spring();
        match step {
            Step::CloseIntro => hud::close_intro(spring, ctx.global()),
            Step::LeaderFixture => {
                springcabal::message::send_to_rules(
                    spring,
                    &Command::LeaderRenderFixture.message(),
                );
            }
            Step::ReportLeaderFixture => report_leader_fixture(spring),
            Step::Heartburn => send_spell(spring, 2, 1000.0),
            Step::Hover(def_name) => self.hovered.set(hover_fixture(spring, def_name)),
            Step::CheckHover(def_name) => {
                report_hover(spring, ctx.global(), def_name, self.hovered.get())
            }
            Step::SeedSelection => {
                let _ = spring
                    .selection()
                    .select_unit_array(&[player_unit()], false);
                let count = spring.selection().get_selected_units_count().unwrap_or(0);
                echo(spring, &alloc::format!("TTLD E2E selection seeded={count}"));
            }
            Step::CheckSelection => {
                let count = spring
                    .selection()
                    .get_selected_units_count()
                    .unwrap_or(u32::MAX);
                echo(
                    spring,
                    &alloc::format!("TTLD E2E selection cleared={count}"),
                );
            }
            Step::MouseWheel => report_wheel_capture(spring),
            Step::MovePlayer => {
                report_tracking(spring, "before_move");
                move_player(spring, 500.0, -7000.0);
            }
            Step::RememberPoultice => self.remember_poultice_target(spring),
            Step::Poultice => send_spell(spring, 5, 0.0),
            Step::ReportPoultice => self.report_poultice(spring),
            Step::Track(label) => report_tracking(spring, label),
            Step::Shot(label) => {
                let shot = self.shots.get() + 1;
                self.shots.set(shot);
                let frame = spring.game().current_frame().unwrap_or(0);
                screenshot(spring, &alloc::format!("{shot} ({label}) at frame {frame}"));
                report_projectiles(spring);
            }
            Step::Done => echo(spring, DONE_MARKER),
        }
    }
}

impl Widget<Global> for DbgE2eWidget {
    fn name(&self) -> &'static str {
        "DbgE2e"
    }

    fn update(&self, ctx: &spring_addons::AddonContext<'_, Global, spring::Ui>, _dt: f32) {
        let spring = ctx.spring();
        let frame = spring.game().current_frame().unwrap_or(0);
        if frame < 0 {
            return;
        }
        self.request_pauses(spring);
        if self.resume_when_due(spring) {
            return;
        }
        let mut running = None;
        while let Some(&(due, when, step)) = STEPS.get(self.next.get()) {
            if frame < due || running.is_some_and(|running| running != due) {
                return;
            }
            running = Some(due);
            if when == When::Paused {
                if frame == due && !self.settled_in_pause(spring) {
                    return;
                }
                if frame > due {
                    echo(
                        spring,
                        &alloc::format!("TTLD E2E missed the pause at frame {due} (now {frame})"),
                    );
                } else {
                    self.resume_in.set(Some(PAUSED_UPDATES));
                }
            }
            self.run(ctx, step);
            self.next.set(self.next.get() + 1);
        }
    }
}

fn report_projectiles(spring: &spring::Spring<'_, spring::Ui>) {
    let projectiles = spring
        .projectiles()
        .get_all_projectiles(spring::projectiles::GetAllProjectilesOptions {
            exclude_weapon_projectiles: false,
            exclude_piece_projectiles: true,
        })
        .unwrap_or_default();
    for projectile in projectiles {
        if let Ok(position) = spring.projectiles().get_projectile_position(projectile) {
            echo(
                spring,
                &alloc::format!(
                    "TTLD E2E projectile {projectile} at ({:.1},{:.1},{:.1})",
                    position.x,
                    position.y,
                    position.z
                ),
            );
        }
    }
}

fn paused(spring: &spring::Spring<'_, spring::Ui>) -> bool {
    spring
        .display()
        .get_game_speed()
        .is_ok_and(|speed| speed.paused)
}

fn command(spring: &spring::Spring<'_, spring::Ui>, command: &str) {
    spring
        .messages()
        .send_commands(command, "")
        .log_err(spring, command);
}

fn echo(spring: &spring::Spring<'_, spring::Ui>, line: &str) {
    spring
        .messages()
        .echo(line, "")
        .log_err(spring, "e2e report");
}

fn report_hover(
    spring: &spring::Spring<'_, spring::Ui>,
    global: &Global,
    def_name: &str,
    unit: Option<i32>,
) {
    let target = global.hud.access().target.map(|(unit, _)| unit);
    let cursor = global.cursor.get().unit;
    echo(
        spring,
        &alloc::format!(
            "TTLD E2E hover check fixture={def_name} unit={} hud_target={} cursor={}",
            unit.unwrap_or(-1),
            target.unwrap_or(-1),
            cursor.unwrap_or(-1),
        ),
    );
}

fn player_unit() -> i32 {
    crate::with_global(|global| global.status.access().player_unit)
}

fn report_leader_fixture(spring: &spring::Spring<'_, spring::Ui>) {
    let player_id = player_unit();
    let Some(player) = unit_position(spring, player_id) else {
        return;
    };
    let leader_def = spring.unit_defs().get_unit_def_id_by_name("orkboss").ok();
    let orkbig_def = spring.unit_defs().get_unit_def_id_by_name("orkbig").ok();
    let mut leader = None;
    let mut orkbig = None;
    for unit_id in spring.units_query().get_all_units().unwrap_or_default() {
        let Ok(def_id) = spring.units_info().get_unit_def_id(unit_id) else {
            continue;
        };
        let Some(position) = unit_position(spring, unit_id) else {
            continue;
        };
        let dx = position.x - player.x;
        let dz = position.z - player.z;
        if dx * dx + dz * dz > 700.0 * 700.0 {
            continue;
        }
        let Ok(screen) = spring.camera().world_to_screen_coords(position) else {
            continue;
        };
        if !screen.valid {
            continue;
        }
        if Some(def_id) == leader_def {
            leader = Some((unit_id, screen.screen_pos.x, screen.screen_pos.y));
        } else if Some(def_id) == orkbig_def {
            orkbig = Some((unit_id, screen.screen_pos.x, screen.screen_pos.y));
        }
    }
    let _ = spring.messages().echo(
        &alloc::format!("TTLD E2E leader render pixels: leader={leader:?} orkbig={orkbig:?}"),
        "",
    );
}

fn unit_position(spring: &spring::Spring<'_, spring::Ui>, unit_id: i32) -> Option<spring::Float3> {
    (unit_id >= 0)
        .then(|| {
            spring.units_info().get_unit_position(
                unit_id,
                spring::units_info::GetUnitPositionOptions {
                    mid_pos: false,
                    aim_pos: false,
                },
            )
        })?
        .ok()
}

fn send_spell(spring: &spring::Spring<'_, spring::Ui>, index: i32, z_offset: f32) {
    let unit_id = player_unit();
    if let Ok(position) = spring.units_info().get_unit_position(
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
        let accepted = springcabal::message::send_to_rules(spring, &message);
        let name = if index == 5 { "poultice" } else { "heartburn" };
        let _ = spring
            .messages()
            .echo(&alloc::format!("TTLD E2E {name} sent: {accepted}"), "");
    } else {
        let _ = spring.messages().echo(
            &alloc::format!("TTLD E2E spell {index} skipped: no player unit"),
            "",
        );
    }
}

fn move_player(spring: &spring::Spring<'_, spring::Ui>, x_offset: f32, z_offset: f32) {
    let unit_id = player_unit();
    let Ok(position) = spring.units_info().get_unit_position(
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
    springcabal::message::send_to_rules(spring, &message);
}

fn report_tracking(spring: &spring::Spring<'_, spring::Ui>, label: &str) {
    let unit_id = player_unit();
    let Ok(unit) = spring.units_info().get_unit_position(
        unit_id,
        spring::units_info::GetUnitPositionOptions {
            mid_pos: true,
            aim_pos: false,
        },
    ) else {
        return;
    };
    let Ok(camera) = spring.camera().get_camera_state(true) else {
        return;
    };
    let dx = camera.pos.x - unit.x;
    let dz = camera.pos.z - unit.z;
    let error = (dx * dx + dz * dz).sqrt();
    let _ = spring.messages().echo(
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

fn report_wheel_capture(spring: &spring::Spring<'_, spring::Ui>) {
    let Ok(before) = spring.camera().get_camera_state(true) else {
        return;
    };
    let accepted = spring.debug_input().emulate_mouse_wheel(1.0).is_ok();
    let Ok(after) = spring.camera().get_camera_state(true) else {
        return;
    };
    let delta = after.dist - before.dist;
    let captured = delta.abs() < 0.01;
    let _ = spring.messages().echo(
        &alloc::format!(
            "TTLD E2E mouse wheel accepted={accepted} before_dist={:.2} after_dist={:.2} delta={delta:.2} captured={captured}",
            before.dist,
            after.dist,
        ),
        "",
    );
}

impl DbgE2eWidget {
    fn remember_poultice_target(&self, spring: &spring::Spring<'_, spring::Ui>) {
        let unit_id = player_unit();
        let Ok(origin) = spring.units_info().get_unit_position(
            unit_id,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: false,
                aim_pos: false,
            },
        ) else {
            return;
        };
        let Ok(output) = spring
            .units()
            .get_units_in_cylinder(origin.x, origin.z, 720.0, 1)
        else {
            return;
        };
        let mut remembered = None;
        let mut best_health = 0.0;
        for target in output {
            let Ok(team) = spring.units_info().get_unit_team(target) else {
                continue;
            };
            let Ok(def) = spring.units_info().get_unit_def_id(target) else {
                continue;
            };
            if team != 1 {
                continue;
            }
            let Ok(position) = spring.units_info().get_unit_position(
                target,
                spring::units_info::GetUnitPositionOptions {
                    mid_pos: false,
                    aim_pos: false,
                },
            ) else {
                continue;
            };
            let health = spring
                .units_info()
                .get_unit_health(target)
                .map_or(0.0, |h| h.health);
            if health > best_health {
                best_health = health;
                remembered = Some((target, team, def, position));
            }
        }
        // Observe the cast's live target now, rather than an arbitrary enemy
        // selected 42 combat frames earlier. Record early flight before impact.
        if let Some((target, team, def, position)) = remembered {
            self.poultice_target.set(target);
            self.poultice_x.set(position.x);
            self.poultice_z.set(position.z);
            let _ = spring.messages().echo(
                &alloc::format!(
                    "TTLD E2E poultice target={target} team={} def={} before=({:.0},{:.0})",
                    team,
                    def,
                    position.x,
                    position.z
                ),
                "",
            );
        }
    }

    fn report_poultice(&self, spring: &spring::Spring<'_, spring::Ui>) {
        let target = self.poultice_target.get();
        if target < 0 {
            let _ = spring
                .messages()
                .echo("TTLD E2E poultice target missing", "");
            return;
        }
        let position = match spring.units_info().get_unit_position(
            target,
            spring::units_info::GetUnitPositionOptions {
                mid_pos: false,
                aim_pos: false,
            },
        ) {
            Ok(position) => position,
            Err(_) => {
                let _ = spring.messages().echo(
                    &alloc::format!("TTLD E2E poultice target={target} no longer queryable"),
                    "",
                );
                return;
            }
        };
        let dx = position.x - self.poultice_x.get();
        let dz = position.z - self.poultice_z.get();
        let distance = (dx * dx + dz * dz).sqrt();
        let velocity = spring
            .units_info()
            .get_unit_velocity(target)
            .unwrap_or_default();
        let _ = spring.messages().echo(
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

fn hover_fixture(spring: &spring::Spring<'_, spring::Ui>, def_name: &str) -> Option<i32> {
    let label = def_name;
    let Some(def_id) = spring.unit_defs().get_unit_def_id_by_name(def_name).ok() else {
        let _ = spring.messages().echo(
            &alloc::format!("TTLD E2E hover fixture={label} skipped: no def"),
            "",
        );
        return None;
    };
    let player_position = unit_position(spring, player_unit())?;
    let mut target = None;
    for unit_id in spring.units_query().get_all_units().unwrap_or_default() {
        if spring.units_info().get_unit_def_id(unit_id).ok() != Some(def_id) {
            continue;
        }
        let Some(position) = unit_position(spring, unit_id) else {
            continue;
        };
        let dx = position.x - player_position.x;
        let dz = position.z - player_position.z;
        if dx * dx + dz * dz <= 700.0 * 700.0 {
            target = Some((unit_id, position));
        }
    }
    let Some((unit_id, position)) = target else {
        let _ = spring.messages().echo(
            &alloc::format!("TTLD E2E hover fixture={label} skipped: no unit"),
            "",
        );
        return None;
    };
    let position = spring::Float3 {
        x: position.x,
        y: position.y + 40.0,
        z: position.z,
    };
    let Ok(screen) = spring.camera().world_to_screen_coords(position) else {
        return None;
    };
    if !screen.valid {
        return None;
    }
    let height = crate::with_global(|global| global.view.get().1);
    if height <= 1 {
        return None;
    }
    let x = screen.screen_pos.x.round() as i32;
    let bottom_y = screen.screen_pos.y.round() as i32;
    let mut hover = None;
    'search: for dx in (-100..=100).step_by(10) {
        for dy in (-100..=100).step_by(10) {
            if trace_unit(spring, x + dx, bottom_y + dy) == Some(unit_id) {
                hover = Some((x + dx, bottom_y + dy));
                break 'search;
            }
        }
    }
    let Some((hover_x, hover_bottom_y)) = hover else {
        let _ = spring.messages().echo(
            &alloc::format!("TTLD E2E hover fixture={label} unit={unit_id} skipped: trace miss"),
            "",
        );
        return None;
    };
    let hover_top_y = height - 1 - hover_bottom_y;
    let result = spring
        .debug_input()
        .emulate_mouse_move(hover_x, hover_top_y);
    let hit = trace_hit(spring, hover_x, hover_bottom_y);
    let _ = spring.messages().echo(
        &alloc::format!(
            "TTLD E2E hover fixture={label} unit={unit_id} mouse=({hover_x},{hover_bottom_y}) trace={hit} move_ok={}",
            result.is_ok(),
        ),
        "",
    );
    Some(unit_id)
}

fn trace_unit(spring: &spring::Spring<'_, spring::Ui>, x: i32, y: i32) -> Option<i32> {
    crate::cursor::unit_at(spring, x as f32, y as f32)
}

fn trace_hit(spring: &spring::Spring<'_, spring::Ui>, x: i32, y: i32) -> alloc::string::String {
    spring
        .camera()
        .trace_screen_ray(
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
