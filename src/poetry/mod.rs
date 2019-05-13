use glium::{implement_vertex, program};
use glium::backend::glutin::Display;
use glium::index::PrimitiveType;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, ErrorKind};
use rand::FromEntropy;
use rand_xoshiro::Xoshiro256Plus;
use std::f32;
use std::borrow::Cow;

mod render;
use self::render::Vertex;

pub struct Poetry {
    speed: f32,
    font: bdf::Font,
    poems: Vec<render::Poem>,
    rand: Xoshiro256Plus,
    program: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u16>,
}

const VERTEX_SHADER: &'static str = "#version 140

in vec2 position;
in vec2 texcoords;

out vec2 vTexCoords;

uniform vec4 bounds;

void main() {
    gl_Position = vec4((position * bounds.zw + bounds.xy) * 2.0 - 1.0, 0.0, 1.0);
    vTexCoords = texcoords;
}
";

const FRAGMENT_SHADER: &'static str = "#version 140

in vec2 vTexCoords;
out vec4 fragColor;

uniform vec4 color;
uniform sampler2D text;

void main() {
    fragColor = texture(text, vTexCoords).rrrr * color;
}
";

impl Poetry {
    pub fn new(display: &Display, font: &str, speed: f32) -> Poetry {
        implement_vertex!(Vertex, position, texcoords);

        let vertex_buffer = glium::VertexBuffer::new(display, &[
            Vertex { position: [0.0,  0.0], texcoords: [ 0.0, 0.0 ] },
            Vertex { position: [0.0, -1.0], texcoords: [ 0.0, 1.0 ] },
            Vertex { position: [1.0, -1.0], texcoords: [ 1.0, 1.0 ] },
            Vertex { position: [1.0,  0.0], texcoords: [ 1.0, 0.0 ] },
            ]).unwrap();
        let index_buffer = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 2, 3, 0]).unwrap();
        let program = program!(display, 140 => { vertex: VERTEX_SHADER, fragment: FRAGMENT_SHADER }).unwrap();
        let font = bdf::open(font).expect("Cannot load font file");

        Poetry {
            speed: speed,
            font: font,
            poems: Vec::new(),
            rand: Xoshiro256Plus::from_entropy(),
            program: program,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
        }
    }

    pub fn show_poem(&mut self, display: &Display, text: &str) {
        self.poems.push(render::Poem::new(&display, &self.font, text, &mut self.rand));
    }

    pub fn step(&mut self, display: &Display) {
        // fade poems
        for i in (0..self.poems.len()).rev() {
            let duration = self.poems[i].created.elapsed();
            let alpha = 1.0 - (duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1e9) / self.speed;
            self.poems[i].color.alpha = alpha;
            if (alpha * 255.0).round() < f32::EPSILON {
                self.poems.swap_remove(i);
            }
        }

        render::Poem::render_all(&display, &self.poems, &self.vertex_buffer, &self.index_buffer, &self.program);
    }
}
