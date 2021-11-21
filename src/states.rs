#![allow(clippy::wrong_self_convention)]
use glium::backend::glutin::Display;
use libmpv::events::{Event, PropertyData};
use log::{error, info};
use std::{
    process::Command,
    time::{Duration, Instant},
};

use crate::{
    config::Config,
    emulator::Emulator,
    frontpanel::{Led, LedControl},
    poetry::Poetry,
    shadertoy::ShaderToy,
    video::Video,
};

pub enum State {
    Off,
    ShaderToy {
        shader_toy: ShaderToy,
    },
    Video {
        video: Video,
    },
    Emulator {
        emulator: Emulator,
        last_frame: Instant,
    },
    VNC,
    Poetry {
        poetry: Poetry,
    },
    Tox,
    ToxMessage {
        poetry: Poetry,
    },
}

pub struct StateMachine {
    display: Display,
    state: State,
    config: Config,
    led_control: Option<LedControl>,
}

impl StateMachine {
    pub fn new(display: Display, led_control: Option<LedControl>, config: Config) -> Self {
        StateMachine {
            state: State::Off,
            display,
            config,
            led_control,
        }
    }

    fn exit_transition(&mut self, next: &State) {
        match next {
            State::Off => {
                if let Some(led_control) = &mut self.led_control {
                    led_control.set(Led::Earth, false).unwrap_or_else(|err| {
                        error!("{}", err);
                    });
                    led_control.set(Led::Relay, false).unwrap_or_else(|err| {
                        error!("{}", err);
                    });
                }
                // frontpanel::write_display("Blinkenwall     Off").unwrap_or_else(|err| {
                //     error!("{}", err);
                // });
            }
            State::ShaderToy { shader_toy: _ } => {
                // frontpanel::write_display("Blinkenwall     ShaderToy").unwrap_or_else(|err| {
                //     error!("{}", err);
                // });
            }
            State::Video { video: _ } => {
                // frontpanel::write_display("Blinkenwall     YouTube").unwrap_or_else(|err| {
                //     error!("{}", err);
                // });
            }
            State::Poetry { poetry: _ } => {
                // frontpanel::write_display("Blinkenwall     Poetry").unwrap_or_else(|err| {
                //     error!("{}", err);
                // });
            }
            State::Tox => {
                // frontpanel::write_display("Blinkenwall     Tox").unwrap_or_else(|err| {
                //     error!("{}", err);
                // });
            }
            State::ToxMessage { poetry: _ } => {
                // frontpanel::write_display("Blinkenwall     Tox Messages").unwrap_or_else(|err| {
                //     error!("{}", err);
                // });
            }
            _ => {}
        }
        match self.state {
            State::Off => {
                info!("Exit Off state");
                if let Some(led_control) = &mut self.led_control {
                    led_control.set(Led::Earth, true).unwrap_or_else(|err| {
                        error!("{}", err);
                    });
                    led_control.set(Led::Relay, true).unwrap_or_else(|err| {
                        error!("{}", err);
                    });
                }
            }
            State::ShaderToy { ref shader_toy } => {
                info!("Exit ShaderToy state");
            }
            State::Video { ref mut video } => {
                info!("Exit Video state");
                video.stop();
            }
            State::Emulator { .. } => {
                info!("Exit Emulator state");
            }
            State::VNC => {
                info!("Exit VNC state");
            }
            State::Poetry { ref poetry } => {
                info!("Exit Poetry state");
            }
            State::Tox => {
                info!("Exit Tox state");
                match next {
                    State::ToxMessage { ref poetry } => {}
                    _ => {
                        #[cfg(target_os = "linux")]
                        Command::new("/usr/bin/sudo")
                            .arg("-Hu")
                            .arg("zoff")
                            .arg("/home/zoff/ToxBlinkenwall/toxblinkenwall/initscript.sh")
                            .arg("stop")
                            .output()
                            .expect("failed to execute process");
                    }
                }
                #[cfg(target_os = "linux")]
                Command::new("/usr/bin/sudo")
                    .arg("/bin/chvt")
                    .arg("1")
                    .output()
                    .expect("failed to execute process");
            }
            State::ToxMessage { ref poetry } => {
                info!("Exit Tox Message state");
            }
        };
    }
    pub fn interval(&self) -> Option<Duration> {
        match self.state {
            State::Off => None,
            State::ShaderToy { ref shader_toy } => Some(Duration::from_secs(0)),
            State::Video { ref video } => Some(Duration::from_secs(0)),
            State::Emulator { last_frame, .. } => {
                // let time_per_frame =
                //     Duration::from_micros((1e6 / self.config.emulator.fps as f32) as _);
                // let frame_time = Instant::now() - last_frame;
                // if time_per_frame < frame_time {
                Some(Duration::from_secs(0))
                // } else {
                //     Some(time_per_frame - frame_time)
                // }
            }
            State::VNC => None,
            State::Poetry { ref poetry } | State::ToxMessage { ref poetry } => {
                Some(Duration::from_secs(0))
            }
            State::Tox => None,
        }
    }

    pub fn to_off(&mut self) {
        if let State::Off = self.state {
        } else {
            let next = State::Off;
            self.exit_transition(&next);
            self.state = next;
            info!("Enter Off state");
        }
    }

    pub fn to_shader_toy(&mut self, shader: &str) {
        if let State::ShaderToy { ref shader_toy } = self.state {
        } else {
            let next = State::ShaderToy {
                shader_toy: ShaderToy::new_with_audio(&self.display, shader),
            };
            self.exit_transition(&next);
            self.state = next;
            info!("Enter ShaderToy state");
            return;
        }
        self.state = State::ShaderToy {
            shader_toy: ShaderToy::new_with_audio(&self.display, shader),
        };
    }

    pub fn to_video(&mut self, url: &str) {
        if let State::Video { ref video } = self.state {
        } else {
            let mut video = Video::new(&self.display);
            video.play(url);
            let next = State::Video { video };
            self.exit_transition(&next);
            self.state = next;
            info!("Enter Video state");
        }
    }

    pub fn to_tox(&mut self) {
        if let State::Tox = self.state {
        } else {
            let next = State::Tox;
            self.exit_transition(&next);
            #[cfg(target_os = "linux")]
            Command::new("/usr/bin/sudo")
                .arg("/bin/chvt")
                .arg("2")
                .output()
                .expect("failed to execute process");
            #[cfg(target_os = "linux")]
            Command::new("/usr/bin/sudo")
                .arg("-Hu")
                .arg("zoff")
                .arg("/home/zoff/ToxBlinkenwall/toxblinkenwall/initscript.sh")
                .arg("start")
                .output()
                .expect("failed to execute process");
            self.state = next;
            info!("Enter Tox state");
        }
    }

    pub fn to_poetry(&mut self, text: &str) {
        if let State::Poetry { ref mut poetry } = self.state {
            if !text.is_empty() {
                poetry.show_poem(&self.display, text);
            }
        } else {
            let mut poetry = Poetry::new(
                &self.display,
                &self.config.poetry.font,
                self.config.poetry.speed,
            );
            if !text.is_empty() {
                poetry.show_poem(&self.display, text);
            }
            let next = State::Poetry { poetry };
            self.exit_transition(&next);
            self.state = next;
            info!("Enter Poetry state");
        }
    }

    pub fn to_tox_message(&mut self, text: &str) {
        if let State::ToxMessage { ref mut poetry } = self.state {
            if !text.is_empty() {
                poetry.show_poem(&self.display, text);
            }
        } else {
            let mut poetry = Poetry::new(
                &self.display,
                &self.config.poetry.font,
                self.config.poetry.speed,
            );
            if !text.is_empty() {
                poetry.show_poem(&self.display, text);
            }
            let next = State::ToxMessage { poetry };
            self.exit_transition(&next);
            self.state = next;
            info!("Enter Tox Message state");
        }
    }

    pub fn to_emulator(&mut self, game: String) {
        let emulator = crate::emulator::Emulator::new(&self.display, &game, &self.config.emulator);
        let next = State::Emulator {
            emulator,
            last_frame: Instant::now(),
        };
        self.exit_transition(&next);
        self.state = next;
        info!("Enter Emulator state");
    }

    pub fn emulator_input(&mut self, key: &str, press: bool) {
        if let State::Emulator { emulator, .. } = &mut self.state {
            emulator.input(key, press);
        }
    }

    pub fn update(&mut self) {
        match self.state {
            State::Off => {}
            State::ShaderToy { ref mut shader_toy } => {
                shader_toy.step(&self.display);
            }
            State::Video { ref mut video } => {
                match video.step(&self.display) {
                    None => {}
                    Some(evt) => {
                        if let Event::PropertyChange {
                            name: "idle-active",
                            change: PropertyData::Flag(idle),
                            ..
                        } = evt
                        {
                            if idle {
                                self.to_off();
                            }
                        } else {
                            info!("MPV event: {:?}", evt);
                        }
                    }
                };
            }
            State::Emulator {
                ref mut emulator,
                ref mut last_frame,
            } => {
                *last_frame = Instant::now();
                emulator.step(&self.display);
            }
            State::VNC => {}
            State::Poetry { ref mut poetry } | State::ToxMessage { ref mut poetry } => {
                poetry.step(&self.display);
            }
            State::Tox => {}
        };
    }
}
