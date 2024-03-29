use chrono::{offset::Utc, DateTime, Datelike, Timelike};
use glium::backend::glutin::Display;
use glium::index::PrimitiveType;
use glium::texture::texture2d::Texture2d;
use glium::texture::{ClientFormat, MipmapsOption, RawImage2d, UncompressedFloatFormat};
use glium::{implement_vertex, program, uniform, Surface};
use std::borrow::Cow;
use std::time::Instant;

mod audio;
mod audio_fft;

const VERTEX_SHADER: &str = "#version 140

in vec2 position;
in vec2 texcoords;

out vec2 vTexCoords;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vTexCoords = texcoords;
}
";

const FRAGMENT_SHADER_PREAMBLE: &str = "#version 140

in vec2 vTexCoords;
out vec4 fragColor;

uniform float iGlobalTime;
uniform float iTime;
uniform vec3 iResolution;
uniform vec4 iMouse;
uniform vec4 iDate;
uniform int iFrame;
uniform sampler2D iChannel0;

void mainImage(out vec4, in vec2);

void main() {
    mainImage(fragColor, vTexCoords * iResolution.xy);
}

";

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    texcoords: [f32; 2],
}

struct Audio {
    input: audio::AudioInput,
    texture: Texture2d,
    fft: audio_fft::AudioFFT,
}

pub struct ShaderToy {
    startup_time: Instant,
    frame: i32,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
    program: glium::Program,
    audio: Option<Audio>,
}

impl ShaderToy {
    fn new_internal(display: &Display, shader: &str, audio: Option<Audio>) -> ShaderToy {
        implement_vertex!(Vertex, position, texcoords);

        let vertex_buffer = glium::VertexBuffer::new(
            display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                    texcoords: [0.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                    texcoords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    texcoords: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                    texcoords: [1.0, 0.0],
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

        let fragment_shader = String::from(FRAGMENT_SHADER_PREAMBLE) + shader;
        let program =
            program!(display, 140 => { vertex: VERTEX_SHADER, fragment: &fragment_shader })
                .unwrap();

        ShaderToy {
            startup_time: Instant::now(),
            frame: 0,
            vertex_buffer,
            index_buffer,
            program,
            audio,
        }
    }

    // pub fn new(display: &Display, shader: &str) -> ShaderToy {
    //     Self::new_internal(display, shader, None)
    // }

    pub fn new_with_audio(display: &Display, shader: &str) -> ShaderToy {
        let mut input = audio::AudioInput::new();
        let data = [0.0; 1024];
        let rawimage = RawImage2d {
            data: Cow::from(&data[..]),
            width: 512,
            height: 2,
            format: ClientFormat::F32,
        };
        let texture = Texture2d::with_format(
            display,
            rawimage,
            UncompressedFloatFormat::F32,
            MipmapsOption::NoMipmap,
        )
        .unwrap();

        input.start().ok();
        Self::new_internal(
            display,
            shader,
            Some(Audio {
                input,
                texture,
                fft: audio_fft::AudioFFT::new(1024),
            }),
        )
    }

    pub fn step(&mut self, display: &Display) {
        let elapsed = self.startup_time.elapsed();
        let utc: DateTime<Utc> = Utc::now();
        let size = display.gl_window().window().inner_size();
        let time = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 / 1.0e9;
        let uniforms = uniform! {
            iGlobalTime: time,
            iTime: time,
            iResolution: [size.width as f32, size.height as f32, 1.0],
            iMouse: [0.0_f32, 0.0, 0.0, 0.0],
            iDate: [utc.year() as f32, utc.month0() as f32, utc.day0() as f32, utc.num_seconds_from_midnight() as f32 + utc.nanosecond() as f32 / 1.0e9],
            iFrame: self.frame,
        };
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        if let Some(ref mut audio) = self.audio {
            if let Some(buffer) = audio.input.poll() {
                let mut texels = [0.; 1024];
                // time domain
                for index in 0..512 {
                    texels[index + 512] = buffer[index * 2] * 0.5 + 0.5;
                }
                // frequency domain
                let fft = audio.fft.process(&buffer);
                for (index, sample) in fft.iter().enumerate() {
                    texels[index] = *sample;
                }
                let rawimage = RawImage2d {
                    data: Cow::from(&texels[..]),
                    width: 512,
                    height: 2,
                    format: ClientFormat::F32,
                };
                audio.texture.write(
                    glium::Rect {
                        left: 0,
                        bottom: 0,
                        width: 512,
                        height: 2,
                    },
                    rawimage,
                );
            }
            let uniforms = uniforms.add("iChannel0", &audio.texture);
            target
                .draw(
                    &self.vertex_buffer,
                    &self.index_buffer,
                    &self.program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();
        } else {
            target
                .draw(
                    &self.vertex_buffer,
                    &self.index_buffer,
                    &self.program,
                    &uniforms,
                    &Default::default(),
                )
                .unwrap();
        }

        target.finish().unwrap();
        self.frame += 1;
    }
}
