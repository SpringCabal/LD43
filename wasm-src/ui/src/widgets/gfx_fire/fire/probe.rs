//! The same five held-age capture schedule used by the Gravitas fire probe.
use super::{Fire, emit};
use springcabal_control::camera::{View, place_camera};

pub struct Probe {
    stage: i32,
    shot: i32,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            stage: -1,
            shot: -1,
        }
    }
}
impl Probe {
    pub fn update(&mut self, spring: &spring::Spring<'_, spring::Ui>, fire: &mut Fire) {
        let _ = spring.unsynced_ctrl().warp_mouse(200, 500);
        place_camera(
            spring,
            &View::overhead(650.0, 0.464).pitch(-0.9, -0.45).fov(45.0),
            spring::Float3 {
                x: 4288.0,
                y: 0.0,
                z: 600.0,
            },
        );
        let frame = spring.game().current_frame().unwrap_or(0);
        if frame < 30 {
            return;
        }
        let stage = ((frame - 30) / 30).min(4);
        if self.stage != stage {
            let continuing = stage == 1 && self.stage == 0;
            self.stage = stage;
            let wind = spring.game().get_wind_vector().expect("wind vector query");
            spring::log_notice!(
                spring,
                "TTLDWind",
                "stage={stage} x={} y={} z={}",
                wind.x,
                wind.y,
                wind.z
            );
            if continuing {
                // Keep the uploaded interior marker from age nine to age ten.
                fire.field.advance(1.0);
            } else {
                fire.field = Default::default();
                let mut anchor = emit::effect(0.0, [0.0; 3]);
                anchor.count = 1;
                anchor.life = 100.0;
                anchor.life_spread = 0.0;
                anchor.delay_spread = 0.0;
                anchor.force = [0.0; 3];
                anchor.speed = 0.0;
                anchor.size = 0.0;
                anchor.size_spread = 0.0;
                anchor.size_growth = 0.0;
                anchor.rotation_speed = 0.0;
                anchor.rotation_speed_spread = 0.0;
                anchor.rotation_spread = 0.0;
                anchor.start_radius = 0.0;
                anchor.colormap = &[[0.0; 4]];
                springcabal::particles::emit_effect(
                    &mut fire.field,
                    &anchor,
                    [9000.0, 144.25, 531.0],
                    &mut || 0.5,
                );
                let mut marker = anchor;
                marker.life = 10.0;
                marker.size = 100.0;
                marker.colormap = &[[1.0; 4]];
                springcabal::particles::emit_effect(
                    &mut fire.field,
                    &marker,
                    [4320.0, 170.0, 531.0],
                    &mut || 0.5,
                );
                let mut random = emit::Random::default();
                let target = [9, 10, 45, 60, 90][stage as usize];
                for n in 1..=target {
                    fire.field.advance(1.0);
                    if n % 6 == 0 && (random.sample() * 400.0) as i32 + 1 < 399 {
                        springcabal::particles::emit_effect(
                            &mut fire.field,
                            &emit::effect(25.0, [0.5, 0.0, -0.25]),
                            [4400.0, 144.25, 531.0],
                            &mut || random.sample(),
                        );
                    }
                }
            }
        }
        if frame >= 45 + 30 * stage && self.shot != stage {
            self.shot = stage;
            let _ = spring.messages().send_commands("screenshot png", "");
        }
        if frame >= 190 {
            let _ = spring.messages().send_commands("quitforce", "");
        }
    }
}
