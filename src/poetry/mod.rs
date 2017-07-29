use glium;
use glium::backend::glutin_backend::GlutinFacade;
use glium::index::PrimitiveType;
use std::net::{TcpListener, TcpStream};
use bdf;
use std::io::{Read, ErrorKind};
use rand;
use rand::{XorShiftRng, Rng};
use std::f32;
use std::borrow::Cow;


mod render;
use self::render::Vertex;

pub struct Poetry {
    listener: TcpListener,
    speed: f32,
    font: bdf::Font,
    incoming: Vec<(TcpStream, Vec<u8>)>,
    poems: Vec<render::Poem>,
    rand: XorShiftRng,
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
    pub fn new(display: &GlutinFacade, ip: &str, port: u16, font: &str, speed: f32) -> Poetry {
        implement_vertex!(Vertex, position, texcoords);

        let vertex_buffer = glium::VertexBuffer::new(display, &[
            Vertex { position: [0.0,  0.0], texcoords: [ 0.0, 0.0 ] },
            Vertex { position: [0.0, -1.0], texcoords: [ 0.0, 1.0 ] },
            Vertex { position: [1.0, -1.0], texcoords: [ 1.0, 1.0 ] },
            Vertex { position: [1.0,  0.0], texcoords: [ 1.0, 0.0 ] },
            ]).unwrap();
        let index_buffer = glium::IndexBuffer::new(display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 2, 3, 0]).unwrap();

        let program = program!(display, 140 => { vertex: VERTEX_SHADER, fragment: FRAGMENT_SHADER }).unwrap();

        let addr = format!("{}:{}", ip, port);
        info!("Poetry listening on {}...", addr);
        let font = bdf::open(font).expect("Cannot load font file");

        let listener = TcpListener::bind(addr).unwrap();
        listener.set_nonblocking(true).expect("Cannot set nonblocking mode.");

        Poetry {
            listener: listener,
            speed: speed,
            font: font,
            incoming: Vec::new(),
            poems: Vec::new(),
            rand: rand::weak_rng(),
            program: program,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
        }
    }

    pub fn step(&mut self, display: &GlutinFacade) {
        while let Ok((stream, addr)) = self.listener.accept() {
            info!("Poetry connection from {}", addr);
            stream.set_nonblocking(true).expect("Cannot set nonblocking mode.");
            self.incoming.push((stream, Vec::new()));
        }

        let mut texts = Vec::new();
        for i in (0..self.incoming.len()).rev() {
            let mut drop_connection = false;
            {
                let &mut (ref mut stream, ref mut buffer) = &mut self.incoming[i];
                let mut readbuffer = [0_u8; 1024];
                loop {
                    match stream.read(&mut readbuffer) {
                        Ok(count) => {
                            if count == 0 {
                                if let Ok(text) = String::from_utf8(buffer.clone()) {
                                    texts.push(text);
                                }
                                info!("Poetry connection to {} closed.", stream.peer_addr().unwrap());

                                drop_connection = true;
                                break;
                            }
                            buffer.append(&mut readbuffer[0..count].to_vec());
                        },
                        Err(err) => {
                            match err.kind() {
                                ErrorKind::Interrupted => {
                                    continue;
                                },
                                ErrorKind::WouldBlock => {
                                    break;
                                },
                                _ => {
                                    error!("Poetry connection to {} got error {}.", stream.peer_addr().unwrap(), err);
                                    drop_connection = true;
                                },
                            }
                        }
                    }
                }
            }

            if drop_connection {
                self.incoming.swap_remove(i);
            }
        }
        // fade old poems
        for i in (0..self.poems.len()).rev() {
            let duration = self.poems[i].created.elapsed();
            let alpha = 1.0 - (duration.as_secs() as f32 + duration.subsec_nanos() as f32 / 1e9) / self.speed;
            self.poems[i].color.alpha = alpha;
            if (alpha * 255.0).round() < f32::EPSILON {
                self.poems.swap_remove(i);
            }
        }

        let ref font = &self.font;
        let ref mut rand = &mut self.rand;
        self.poems.extend(texts.into_iter().map(|text| {
            render::Poem::new(&display, &font, &text, rand)
        }));

        render::Poem::render_all(&display, &self.poems, &self.vertex_buffer, &self.index_buffer, &self.program);
    }
}
