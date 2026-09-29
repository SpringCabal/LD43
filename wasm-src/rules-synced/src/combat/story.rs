use spring::commands;

use crate::random;
use crate::state::GameEnd;
use crate::units::{self, ENEMY_TEAM, OrkKind};

/// The wave schedule.
#[derive(Default)]
pub struct Story {
    step: usize,
    next_frame: i32,
}

pub struct Wave {
    pub name: &'static str,
    pub units: &'static [(OrkKind, i32)],
    pub seconds: i32,
    pub able_to_win: bool,
    pub infinite_loop: bool,
}

impl Story {
    /// Picks the wave that is due, if any, and advances the schedule. Once the
    /// town is saved, nothing is due.
    pub fn take_due_wave(&mut self, frame: i32, game_end: GameEnd) -> Option<&'static Wave> {
        if game_end == GameEnd::Victory || self.step >= WAVES.len() || frame < self.next_frame {
            return None;
        }
        let wave = &WAVES[self.step];
        if !wave.infinite_loop {
            self.step += 1;
        }
        self.next_frame = frame + wave.seconds * GAME_FRAMES_PER_SECOND;
        Some(wave)
    }

    /// A story at The Loop, with its wave due.
    #[cfg(feature = "headless-integration")]
    pub fn at_the_loop() -> Self {
        Self {
            step: WAVES.len() - 1,
            next_frame: 0,
        }
    }

    /// The wave that brings the boss.
    #[cfg(feature = "headless-integration")]
    pub fn final_wave() -> &'static Wave {
        WAVES
            .iter()
            .find(|wave| wave.able_to_win)
            .expect("the story has a final wave")
    }
}

const GAME_FRAMES_PER_SECOND: i32 = 33;
/// Waves spawn on a ring around this point, which game_storyteller.lua puts
/// 20 elmos north of the town centre the orks attack.
const RING_CENTER_X: f32 = units::TOWN_CENTRE_X;
const RING_CENTER_Z: f32 = units::TOWN_CENTRE_Z - 20.0;
const RING_RADIUS: f32 = 3000.0;
const RING_RADIUS_JITTER: f32 = 200.0;
const ANGLE_SPAN: f32 = 320.0;
/// Arcs of the spawn ring, in degrees, that game_storyteller.lua keeps orks out
/// of, and how far it rotates a spawn that lands in one.
const SKIPPED_ARCS: [(f32, f32, f32); 2] = [(65.0, 85.0, 270.0), (180.0, 200.0, 210.0)];

const WAVES: [Wave; 13] = [
    Wave {
        name: "First Night",
        units: &[(OrkKind::Small, 40), (OrkKind::Big, 1)],
        seconds: 8,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "First Night b",
        units: &[(OrkKind::Small, 30), (OrkKind::Big, 2)],
        seconds: 13,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "Second Night",
        units: &[(OrkKind::Small, 70), (OrkKind::Big, 3)],
        seconds: 18,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "Getting Bigger",
        units: &[(OrkKind::Small, 70), (OrkKind::Big, 8)],
        seconds: 8,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "Quick Followup",
        units: &[(OrkKind::Small, 70), (OrkKind::Big, 6)],
        seconds: 20,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "Big Ork Wave",
        units: &[(OrkKind::Small, 32), (OrkKind::Big, 28)],
        seconds: 24,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "The Swarm 1",
        units: &[(OrkKind::Small, 60), (OrkKind::Big, 1)],
        seconds: 2,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "The Swarm 2",
        units: &[(OrkKind::Small, 65), (OrkKind::Big, 2)],
        seconds: 2,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "The Swarm 3",
        units: &[(OrkKind::Small, 70), (OrkKind::Big, 2)],
        seconds: 25,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "More Nights",
        units: &[(OrkKind::Small, 45), (OrkKind::Big, 15)],
        seconds: 5,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "More Nights 2",
        units: &[(OrkKind::Small, 50), (OrkKind::Big, 10)],
        seconds: 22,
        able_to_win: false,
        infinite_loop: false,
    },
    Wave {
        name: "Final Boss",
        units: &[(OrkKind::Small, 80), (OrkKind::Big, 8), (OrkKind::Boss, 1)],
        seconds: 15,
        able_to_win: true,
        infinite_loop: false,
    },
    Wave {
        name: "The Loop",
        units: &[(OrkKind::Small, 30), (OrkKind::Big, 3)],
        seconds: 12,
        able_to_win: false,
        infinite_loop: true,
    },
];

/// Spawns the wave once the current callin has returned; see [`spawn`].
pub fn spawn_wave(wave: &Wave) {
    spring::log_info!("ttld-wasm", "wave: {}", wave.name);
    for (kind, count) in wave.units {
        for _ in 0..*count {
            spawn(*kind);
        }
    }
}

/// Spawns one more boss on the ring, once the current callin has returned.
pub fn spawn_boss() {
    spawn(OrkKind::Boss);
}

/// One command per ork: its ring position is drawn when it is created, so
/// the draws alternate with those of the ork AI's UnitCreated as in Lua.
fn spawn(kind: OrkKind) {
    commands::run(move || {
        let (x, z) = ring_position();
        units::create(kind.unit(), x, z, ENEMY_TEAM, 0);
    });
}

fn ring_position() -> (f32, f32) {
    let radius = RING_RADIUS + random::unit_float() * RING_RADIUS_JITTER;
    let mut angle = random::unit_float() * ANGLE_SPAN;
    for (from, to, rotation) in SKIPPED_ARCS {
        if angle > from && angle < to {
            angle += rotation;
        }
    }
    let angle = angle * core::f32::consts::PI / 180.0;
    (
        RING_CENTER_X + angle.cos() * radius,
        RING_CENTER_Z - angle.sin() * radius,
    )
}
