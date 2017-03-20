use glium;
use glium::Surface;
use glium::index::PrimitiveType;
use glium::backend::glutin_backend::GlutinFacade;
use std::time::Instant;
use chrono::prelude::UTC;
use chrono::datetime::DateTime;
use chrono::{Datelike, Timelike};

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
uniform vec3 iResolution;
uniform vec4 iMouse;
uniform vec4 iDate;
uniform int iFrame;

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

pub struct ShaderToy {
    startup_time: Instant,
    frame: i32,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
    program: glium::Program,
}

impl ShaderToy {
    pub fn new(display: &GlutinFacade, shader: &str) -> ShaderToy {
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
        }
    }

    pub fn step(&mut self, display: &GlutinFacade) {
        let elapsed = self.startup_time.elapsed();
        let utc: DateTime<UTC> = UTC::now();
        let size = display.get_window().unwrap().get_inner_size().unwrap();
        let uniforms = uniform! {
            iGlobalTime: elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 / 1.0e9,
            iResolution: [size.0 as f32, size.1 as f32, 1.0],
            iMouse: [0.0_f32, 0.0, 0.0, 0.0],
            iDate: [utc.year() as f32, utc.month0() as f32, utc.day0() as f32, utc.num_seconds_from_midnight() as f32 + utc.nanosecond() as f32 / 1.0e9],
            iFrame: self.frame,
        };
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
        self.frame += 1;
    }
}