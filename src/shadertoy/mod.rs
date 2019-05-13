use glium::{Surface, uniform, program, implement_vertex};
use glium::index::PrimitiveType;
use glium::backend::glutin::Display;
use glium::texture::texture2d::Texture2d;
use glium::texture::{RawImage2d, ClientFormat, MipmapsOption, UncompressedFloatFormat};
use std::time::Instant;
use chrono::{DateTime, Datelike, Timelike, offset::Utc};
use std::borrow::Cow;

mod audio;
mod audio_fft;

const VERTEX_SHADER: &'static str = "#version 140

in vec2 position;
in vec2 texcoords;

out vec2 vTexCoords;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vTexCoords = texcoords;
}
";

const FRAGMENT_SHADER_PREAMBLE: &'static str = "#version 140

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

        let vertex_buffer = glium::VertexBuffer::new(display, &[
            Vertex { position: [-1.0, -1.0], texcoords: [ 0.0, 0.0 ] },
            Vertex { position: [-1.0,  1.0], texcoords: [ 0.0, 1.0 ] },
            Vertex { position: [ 1.0,  1.0], texcoords: [ 1.0, 1.0 ] },
            Vertex { position: [ 1.0, -1.0], texcoords: [ 1.0, 0.0 ] },
            ]).unwrap();
        let index_buffer = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 2, 3, 0]).unwrap();

        let fragment_shader = String::from(FRAGMENT_SHADER_PREAMBLE) + shader;
        let program = program!(display, 140 => { vertex: VERTEX_SHADER, fragment: &fragment_shader }).unwrap();

        ShaderToy {
            startup_time: Instant::now(),
            frame: 0,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            program: program,
            audio: audio,
        }
    }

    pub fn new(display: &Display, shader: &str) -> ShaderToy {
        Self::new_internal(display, shader, None)
    }

    pub fn new_with_audio(display: &Display, shader: &str) -> ShaderToy {
        let mut audio = audio::AudioInput::new();
        let data = [0.0; 1024];
        let rawimage = RawImage2d {
            data: Cow::from(&data[..]),
            width: 512,
            height: 2,
            format: ClientFormat::F32,
        };
        let texture = Texture2d::with_format(display, rawimage, UncompressedFloatFormat::F32, MipmapsOption::NoMipmap).unwrap();

        audio.start();
        Self::new_internal(display, shader, Some(Audio {
            input: audio,
            texture: texture,
            fft: audio_fft::AudioFFT::new(1024),
        }))
    }

    pub fn step(&mut self, display: &Display) {
        let elapsed = self.startup_time.elapsed();
        let utc: DateTime<Utc> = Utc::now();
        let size = display.gl_window().window().get_inner_size().unwrap();
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
                    texels[index + 512] = buffer[index*2] * 0.5 + 0.5;
                }
                // frequency domain
                let fft = audio.fft.process(&buffer.to_vec());
                for (index, sample) in fft.iter().enumerate() {
                    texels[index] = *sample;
                }
                let rawimage = RawImage2d {
                    data: Cow::from(&texels[..]),
                    width: 512,
                    height: 2,
                    format: ClientFormat::F32,
                };
                audio.texture.write(glium::Rect {left: 0, bottom: 0, width: 512, height: 2}, rawimage);
            }
            let uniforms = uniforms.add("iChannel0", &audio.texture);
            target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
        } else {
            target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
        }

        target.finish().unwrap();
        self.frame += 1;
    }
}
