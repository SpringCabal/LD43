#![no_std]

extern crate alloc;

pub mod minimap;

use alloc::string::String;

use springcabal::message::{Encoder, fields, key_values};

pub const STATUS_PERIOD: i32 = 6;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameEnd {
    #[default]
    Start,
    VictoryPossible,
    Victory,
}

impl GameEnd {
    pub fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::VictoryPossible => "victoryPossible",
            Self::Victory => "victory",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "start" => Some(Self::Start),
            "victoryPossible" => Some(Self::VictoryPossible),
            "victory" => Some(Self::Victory),
            _ => None,
        }
    }
}

const STATUS: &str = "TTLD_STATUS";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Status {
    pub round: u32,
    pub player_unit: i32,
    pub health: i32,
    pub max_health: i32,
    pub casting: bool,
    pub cast_start: i32,
    pub cast_animation_frames: i32,
    pub villagers_killed: i32,
    pub allies_drained: i32,
    pub orks_killed: i32,
    pub game_end: GameEnd,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            round: 0,
            player_unit: -1,
            health: 0,
            max_health: 1000,
            casting: false,
            cast_start: 0,
            cast_animation_frames: 0,
            villagers_killed: 0,
            allies_drained: 0,
            orks_killed: 0,
            game_end: GameEnd::Start,
        }
    }
}

impl Status {
    pub fn player_alive(&self) -> bool {
        self.player_unit >= 0
    }

    pub fn message(&self) -> String {
        Encoder::new(STATUS)
            .kv("round", self.round)
            .kv("playerUnit", self.player_unit)
            .kv("health", self.health)
            .kv("maxHealth", self.max_health)
            .kv_flag("casting", self.casting)
            .kv("castStart", self.cast_start)
            .kv("castAnimTime", self.cast_animation_frames)
            .kv("villagersKilled", self.villagers_killed)
            .kv("alliesDrained", self.allies_drained)
            .kv("orksKilled", self.orks_killed)
            .kv("gameEnd", self.game_end.name())
            .finish()
    }

    pub fn parse(message: &str) -> Option<Self> {
        let mut status = Self::default();
        for (key, value) in key_values(message, STATUS)? {
            match key {
                "round" => parse_into(value, &mut status.round),
                "playerUnit" => parse_into(value, &mut status.player_unit),
                "health" => parse_into(value, &mut status.health),
                "maxHealth" => parse_into(value, &mut status.max_health),
                "casting" => status.casting = value != "0",
                "castStart" => parse_into(value, &mut status.cast_start),
                "castAnimTime" => parse_into(value, &mut status.cast_animation_frames),
                "villagersKilled" => parse_into(value, &mut status.villagers_killed),
                "alliesDrained" => parse_into(value, &mut status.allies_drained),
                "orksKilled" => parse_into(value, &mut status.orks_killed),
                "gameEnd" => {
                    if let Some(game_end) = GameEnd::parse(value) {
                        status.game_end = game_end;
                    }
                }
                _ => {}
            }
        }
        Some(status)
    }
}

fn parse_into<T: core::str::FromStr>(value: &str, target: &mut T) {
    if let Ok(value) = value.parse() {
        *target = value;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    Move { x: f32, z: f32 },
    Attack { x: f32, z: f32, move_to: bool },
    Stop,
    Spell { index: i32, x: f32, y: f32, z: f32 },
    Restart,
    LeaderRenderFixture,
}

impl Command {
    pub fn message(&self) -> String {
        match *self {
            Self::Move { x, z } => Encoder::new("movement").field(x).field(z).finish(),
            Self::Attack { x, z, move_to } => {
                Encoder::new(if move_to { "attmove" } else { "attack" })
                    .field(x)
                    .field(z)
                    .finish()
            }
            Self::Stop => String::from("stop"),
            Self::Spell { index, x, y, z } => Encoder::new("spell")
                .field(index)
                .field(x)
                .field(y)
                .field(z)
                .finish(),
            Self::Restart => String::from("restart"),
            Self::LeaderRenderFixture => String::from("e2e|leader-render"),
        }
    }

    pub fn parse(message: &str) -> Option<Self> {
        match message {
            "stop" => return Some(Self::Stop),
            "restart" => return Some(Self::Restart),
            "e2e|leader-render" => return Some(Self::LeaderRenderFixture),
            _ => {}
        }
        if let Some(mut fields) = fields(message, "movement") {
            let (x, z) = fields.xz()?;
            return fields.is_done().then_some(Self::Move { x, z });
        }
        for (prefix, move_to) in [("attack", false), ("attmove", true)] {
            if let Some(mut fields) = fields(message, prefix) {
                let (x, z) = fields.xz()?;
                return fields.is_done().then_some(Self::Attack { x, z, move_to });
            }
        }
        let mut fields = fields(message, "spell")?;
        let index = fields.i32()?;
        let (x, y, z) = fields.xyz()?;
        fields.is_done().then_some(Self::Spell { index, x, y, z })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_round_trips() {
        let status = Status {
            round: 2,
            player_unit: 17,
            health: 1500,
            max_health: 3118,
            casting: true,
            cast_start: 151,
            cast_animation_frames: 20,
            villagers_killed: 4,
            allies_drained: 3,
            orks_killed: 9,
            game_end: GameEnd::VictoryPossible,
        };
        assert_eq!(Status::parse(&status.message()), Some(status));
        assert_eq!(
            Status::parse(&Status::default().message()),
            Some(Status::default())
        );
    }

    #[test]
    fn status_keeps_defaults_for_bad_fields() {
        let status = Status::parse("TTLD_STATUS|health=x|gameEnd=lost|round=3").unwrap();
        assert_eq!(status.health, 0);
        assert_eq!(status.game_end, GameEnd::Start);
        assert_eq!(status.round, 3);
        assert!(!status.player_alive());
    }

    #[test]
    fn other_messages_are_not_statuses() {
        assert_eq!(Status::parse("stop"), None);
        assert_eq!(Status::parse("TTLD_STATUSX|round=1"), None);
    }

    #[test]
    fn game_end_names_round_trip() {
        for game_end in [GameEnd::Start, GameEnd::VictoryPossible, GameEnd::Victory] {
            assert_eq!(GameEnd::parse(game_end.name()), Some(game_end));
        }
    }

    #[test]
    fn commands_round_trip() {
        for command in [
            Command::Move { x: 1.5, z: -2.0 },
            Command::Attack {
                x: 3.0,
                z: 4.0,
                move_to: false,
            },
            Command::Attack {
                x: 3.0,
                z: 4.0,
                move_to: true,
            },
            Command::Stop,
            Command::Spell {
                index: 4,
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            Command::Restart,
            Command::LeaderRenderFixture,
        ] {
            assert_eq!(Command::parse(&command.message()), Some(command));
        }
    }

    #[test]
    fn malformed_commands_are_rejected() {
        assert_eq!(Command::parse("movement|1"), None);
        assert_eq!(Command::parse("spell|x|1|2|3"), None);
        assert_eq!(Command::parse("attack|nan|1"), None);
        assert_eq!(Command::parse("hello"), None);
        for message in [
            "movement|1|2|3",
            "attack|1|2|3",
            "attmove|1|2|3",
            "spell|1|2|3|4|5",
        ] {
            assert_eq!(Command::parse(message), None);
        }
    }

    #[test]
    fn the_ui_messages_are_the_ones_lua_used() {
        assert_eq!(Command::Stop.message(), "stop");
        assert!(
            Command::Move { x: 1.0, z: 2.0 }
                .message()
                .starts_with("movement|")
        );
        assert!(
            Command::Spell {
                index: 2,
                x: 0.0,
                y: 0.0,
                z: 0.0
            }
            .message()
            .starts_with("spell|2|")
        );
    }
}
