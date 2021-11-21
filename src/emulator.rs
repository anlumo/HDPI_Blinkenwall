use std::{borrow::Cow, path::Path};

use glium::{
    backend::glutin::Display,
    index::PrimitiveType,
    texture::{
        texture2d::Texture2d, ClientFormat, MipmapsOption, RawImage2d, UncompressedFloatFormat,
    },
    uniform,
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler, UniformValue},
    Frame, Rect, Surface, {implement_vertex, program},
};
use mizu_core::{GameBoy, GameboyConfig, JoypadButton};

mod audio;
use audio::AudioPlayer;

pub const GB_WIDTH: u32 = 160;
pub const GB_HEIGHT: u32 = 144;
pub const SCALE: (f32, f32) = (
    GB_WIDTH as f32 / GB_WIDTH.next_power_of_two() as f32,
    GB_HEIGHT as f32 / GB_HEIGHT.next_power_of_two() as f32,
);

const VERTEX_SHADER: &str = "#version 140

in vec2 position;
in vec2 texcoords;

out vec2 vTexCoords;

uniform vec4 transform;

void main() {
    gl_Position = vec4(position.x * transform.x + transform.z, position.y * transform.y + transform.w, 0.0, 1.0);
    vTexCoords = texcoords;
}
";

const FRAGMENT_SHADER: &str = "#version 140

in vec2 vTexCoords;
out vec4 fragColor;

uniform sampler2D tex;

void main() {
    fragColor = vec4(texture(tex, vTexCoords).rgb, 1.0);
}
";

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub texcoords: [f32; 2],
}

pub struct Emulator {
    game_name: String,
    gameboy: GameBoy,
    fps: u32,
    texture: Texture2d,
    program: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
    audio_player: AudioPlayer,
}

impl Emulator {
    pub fn available_roms(roms: impl AsRef<Path>) -> Result<Vec<String>, std::io::Error> {
        let path: &Path = roms.as_ref();
        std::fs::read_dir(path)?
            .map(|file| -> Result<Option<_>, _> {
                let path = file?.path();
                Ok(path
                    .extension()
                    .and_then(|extension| {
                        (extension.to_str() == Some("gb") || extension.to_str() == Some("gbc"))
                            .then(|| path.as_path())
                    })
                    .and_then(|path| {
                        path.file_name()
                            .and_then(|file_name| file_name.to_str().map(|s| s.to_owned()))
                    }))
            })
            .filter_map(std::result::Result::transpose)
            .collect()
    }

    pub fn new(display: &Display, game: &str, config: &crate::config::Emulator) -> Self {
        let mizu_config = GameboyConfig { is_dmg: config.dmg };
        let mut file_path = <String as AsRef<Path>>::as_ref(&config.roms).to_path_buf();
        file_path.push(game);

        implement_vertex!(Vertex, position, texcoords);

        let vertex_buffer = glium::VertexBuffer::new(
            display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                    texcoords: [0.0, SCALE.1],
                },
                Vertex {
                    position: [-1.0, 1.0],
                    texcoords: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    texcoords: [SCALE.0, 0.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                    texcoords: [SCALE.0, SCALE.1],
                },
            ],
        )
        .unwrap();
        let index_buffer = glium::IndexBuffer::new(
            display,
            PrimitiveType::TrianglesList,
            &[0u16, 1, 2, 2, 3, 0],
        )
        .unwrap();
        let program =
            program!(display, 140 => { vertex: VERTEX_SHADER, fragment: FRAGMENT_SHADER }).unwrap();

        let texture = Texture2d::empty_with_mipmaps(
            display,
            MipmapsOption::NoMipmap,
            GB_WIDTH.next_power_of_two(),
            GB_HEIGHT.next_power_of_two(),
        )
        .unwrap();

        let audio_player = AudioPlayer::new(44100);
        audio_player.play();

        Self {
            game_name: game.to_owned(),
            gameboy: GameBoy::new(file_path, None, mizu_config).unwrap(),
            fps: config.fps,
            texture,
            program,
            vertex_buffer,
            index_buffer,
            audio_player,
        }
    }

    pub fn game_name(&self) -> &str {
        self.gameboy.game_title()
    }

    pub fn input(&mut self, key: &str, press: bool) {
        let button = match key {
            "a" => JoypadButton::A,
            "b" => JoypadButton::B,
            "select" => JoypadButton::Select,
            "start" => JoypadButton::Start,
            "up" => JoypadButton::Up,
            "down" => JoypadButton::Down,
            "left" => JoypadButton::Left,
            "right" => JoypadButton::Right,
            _ => return,
        };
        if press {
            self.gameboy.press_joypad(button);
        } else {
            self.gameboy.release_joypad(button);
        }
    }

    pub fn step(&mut self, display: &Display) {
        let size = display.gl_window().window().inner_size();
        // TODO: handle key inputs
        self.gameboy.clock_for_frame();

        let audio_buffer = self.gameboy.audio_buffer();
        self.audio_player.queue(&audio_buffer);

        let raw_image = RawImage2d {
            data: Cow::Borrowed(self.gameboy.screen_buffer()),
            width: GB_WIDTH,
            height: GB_HEIGHT,
            format: ClientFormat::U8U8U8,
        };
        self.texture.write(
            Rect {
                left: 0,
                bottom: 0,
                width: GB_WIDTH,
                height: GB_HEIGHT,
            },
            raw_image,
        );

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        let uniforms = uniform! {
            tex: Sampler::new(&self.texture)
                .minify_filter(MinifySamplerFilter::Nearest)
                .magnify_filter(MagnifySamplerFilter::Nearest),
            transform: [GB_WIDTH as f32 / size.width as f32, GB_HEIGHT as f32 / size.height as f32, 0.0, 0.0]
        };

        target
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish();
    }
}
