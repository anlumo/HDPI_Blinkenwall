use glium::backend::glutin::Display;
use std::time::Duration;
use std::process::Command;

use crate::shadertoy::ShaderToy;
use crate::video::Video;
use crate::poetry::Poetry;
use crate::config::Config;

pub enum State {
    Off,
    ShaderToy { shader_toy: ShaderToy },
    Video { video: Video },
    Emulator,
    VNC,
    Poetry { poetry: Poetry },
    Tox,
    ToxMessage { poetry: Poetry },
}

pub struct StateMachine {
    display: Display,
    state: State,
    config: Config,
}

impl StateMachine {
    pub fn new(display: Display, config: Config) -> Self {
        StateMachine {
            state: State::Off,
            display: display,
            config: config,
        }
    }

    fn exit_transition(&mut self, next: &State) {
        match self.state {
            State::Off => {
                info!("Exit Off state");
            },
            State::ShaderToy { ref shader_toy } => {
                info!("Exit ShaderToy state");
            },
            State::Video { ref mut video } => {
                info!("Exit Video state");
                video.stop();
            },
            State::Emulator => {
                info!("Exit Emulator state");
            },
            State::VNC => {
                info!("Exit VNC state");
            },
            State::Poetry { ref poetry } => {
                info!("Exit Poetry state");
            },
            State::Tox => {
                info!("Exit Tox state");
                match next {
                    &State::ToxMessage { ref poetry } => {},
                    _ => {
                        Command::new("/usr/bin/sudo")
                            .arg("-Hu")
                            .arg("zoff")
                            .arg("/home/zoff/ToxBlinkenwall/toxblinkenwall/initscript.sh")
                            .arg("stop")
                            .output()
                            .expect("failed to execute process");
                    },
                }
                Command::new("/usr/bin/sudo")
                    .arg("/bin/chvt")
                    .arg("1")
                    .output()
                    .expect("failed to execute process");
            },
            State::ToxMessage { ref poetry } => {
                info!("Exit Tox Message state");
            },
        };
    }
    pub fn interval(&self) -> Option<Duration> {
        match self.state {
            State::Off => None,
            State::ShaderToy { ref shader_toy } => Some(Duration::from_secs(0)),
            State::Video { ref video } => Some(Duration::from_secs(0)),
            State::Emulator => None,
            State::VNC => None,
            State::Poetry { ref poetry } | State::ToxMessage { ref poetry } => Some(Duration::from_secs(0)),
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
            let next = State::ShaderToy { shader_toy: ShaderToy::new_with_audio(&self.display, shader) };
            self.exit_transition(&next);
            self.state = next;
            info!("Enter ShaderToy state");
            return;
        }
        self.state = State::ShaderToy { shader_toy: ShaderToy::new_with_audio(&self.display, shader) };
    }

    pub fn to_video(&mut self, url: &str) {
        if let State::Video { ref video } = self.state {
        } else {
            let mut video = Video::new(&self.display);
            video.play(url);
            let next = State::Video { video: video };
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
            Command::new("/usr/bin/sudo")
                .arg("/bin/chvt")
                .arg("2")
                .output()
                .expect("failed to execute process");
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
            if text.len() > 0 {
                poetry.show_poem(&self.display, text);
            }
        } else {
            let mut poetry = Poetry::new(&self.display, &self.config.poetry.font, self.config.poetry.speed);
            if text.len() > 0 {
                poetry.show_poem(&self.display, text);
            }
            let next = State::Poetry { poetry: poetry };
            self.exit_transition(&next);
            self.state = next;
            info!("Enter Poetry state");
        }
    }

    pub fn to_tox_message(&mut self, text: &str) {
        if let State::ToxMessage { ref mut poetry } = self.state {
            if text.len() > 0 {
                poetry.show_poem(&self.display, text);
            }
        } else {
            let mut poetry = Poetry::new(&self.display, &self.config.poetry.font, self.config.poetry.speed);
            if text.len() > 0 {
                poetry.show_poem(&self.display, text);
            }
            let next = State::ToxMessage { poetry: poetry };
            self.exit_transition(&next);
            self.state = next;
            info!("Enter Tox Message state");
        }
    }

    pub fn update(&mut self) {
        match self.state {
            State::Off => {},
            State::ShaderToy { ref mut shader_toy } => {
                shader_toy.step(&self.display);
            },
            State::Video { ref mut video } => {
                match video.step(&self.display) {
                    None => {},
                    Some(evt) => info!("MPV event: {:?}", evt),
                };
            },
            State::Emulator => {},
            State::VNC => {},
            State::Poetry { ref mut poetry } | State::ToxMessage { ref mut poetry } => {
                poetry.step(&self.display);
            },
            State::Tox => {},
        };
    }
}
