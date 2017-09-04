use glium::backend::glutin_backend::GlutinFacade;
use std::time::Duration;
use std::process::Command;

use shadertoy::ShaderToy;
use video::Video;
use poetry::Poetry;
use config::Config;

pub enum State {
    Off,
    ShaderToy { shader_toy: ShaderToy },
    Video { video: Video },
    Emulator,
    VNC,
    Poetry { poetry: Poetry },
    Tox,
}

pub struct StateMachine {
    display: GlutinFacade,
    state: State,
    config: Config,
}

impl StateMachine {
    pub fn new(display: GlutinFacade, config: Config) -> Self {
        StateMachine {
            state: State::Off,
            display: display,
            config: config,
        }
    }

    fn exit_transition(&mut self) {
        match self.state {
            State::Off => {},
            State::ShaderToy { ref shader_toy } => {},
            State::Video { ref mut video } => {
                video.stop();
            },
            State::Emulator => {},
            State::VNC => {},
            State::Poetry { ref poetry } => {},
            State::Tox => {
                Command::new("/usr/bin/sudo")
                    .arg("-Hu")
                    .arg("zoff")
                    .arg("/home/zoff/ToxBlinkenwall/toxblinkenwall/initscript.sh")
                    .arg("stop")
                    .output()
                    .expect("failed to execute process");
            },
        };
    }
    pub fn interval(&self) -> Option<Duration> {
        match self.state {
            State::Off => None,
            State::ShaderToy { ref shader_toy } => Some(Duration::from_secs(0)),
            State::Video { ref video } => Some(Duration::from_secs(1)),
            State::Emulator => None,
            State::VNC => None,
            State::Poetry { ref poetry } => Some(Duration::from_secs(0)),
            State::Tox => None,
        }
    }

    pub fn to_off(&mut self) {
        if let State::Off = self.state {
        } else {
            self.exit_transition();
            self.state = State::Off;
        }
    }

    pub fn to_shader_toy(&mut self, shader: &str) {
        if let State::ShaderToy { ref shader_toy } = self.state {
            self.state = State::ShaderToy { shader_toy: ShaderToy::new_with_audio(&self.display, shader) };
        } else {
            self.exit_transition();
            self.state = State::ShaderToy { shader_toy: ShaderToy::new_with_audio(&self.display, shader) };
        }
    }

    pub fn to_video(&mut self, url: &str) {
        if let State::Video { ref video } = self.state {
        } else {
            self.exit_transition();
            let mut video = Video::new(&self.display.get_window().unwrap());
            video.play(url);
            self.state = State::Video { video: video };
        }
    }

    pub fn to_tox(&mut self) {
        if let State::Tox = self.state {
        } else {
            self.exit_transition();
            Command::new("/usr/bin/sudo")
                .arg("-Hu")
                .arg("zoff")
                .arg("/home/zoff/ToxBlinkenwall/toxblinkenwall/initscript.sh")
                .arg("start")
                .output()
                .expect("failed to execute process");
        }
    }

    pub fn to_poetry(&mut self, text: &str) {
        if let State::Poetry { ref mut poetry } = self.state {
            if text.len() > 0 {
                poetry.show_poem(&self.display, text);
            }
        } else {
            self.exit_transition();
            let mut poetry = Poetry::new(&self.display, &self.config.poetry.font, self.config.poetry.speed);
            if text.len() > 0 {
                poetry.show_poem(&self.display, text);
            }
            self.state = State::Poetry { poetry: poetry };
        }
    }

    pub fn update(&mut self) {
        match self.state {
            State::Off => {},
            State::ShaderToy { ref mut shader_toy } => {
                shader_toy.step(&self.display);
            },
            State::Video { ref mut video } => {
                match video.step(&self.display.get_window().unwrap()) {
                    None => {},
                    Some(evt) => info!("MPV event: {:?}", evt),
                };
            },
            State::Emulator => {},
            State::VNC => {},
            State::Poetry { ref mut poetry } => {
                poetry.step(&self.display);
            },
            State::Tox => {},
        };
    }
}
