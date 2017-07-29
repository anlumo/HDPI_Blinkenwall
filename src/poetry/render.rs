use glium;
use glium::{Surface, Frame};
use glium::index::PrimitiveType;
use glium::texture::texture2d::Texture2d;
use glium::texture::{RawImage2d, ClientFormat, MipmapsOption, UncompressedFloatFormat};
use glium::backend::glutin_backend::GlutinFacade;
use std::time::Instant;
use palette::{Rgb, Rgba, RgbHue, Colora};
use rand;
use bdf;
use std::borrow::Cow;
use rand::Rng;
use std::cmp;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub texcoords: [f32; 2],
}

pub struct Poem {
    pub created: Instant,
    pub color: Rgba,
    texture: Texture2d,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Poem {
    pub fn new(display: &GlutinFacade, font: &bdf::Font, text: &str, rand: &mut Rng) -> Poem {
        let window_size = display.get_window().unwrap().get_inner_size().unwrap();
        let font_size = font.device_width().unwrap();

        let width = cmp::min((window_size.0 / font_size.0) as usize, text.lines().map(|line| line.len()).max().unwrap());
        let height = cmp::min((window_size.1 / font_size.1) as usize, text.lines().count());
        let pixel_w = ((font.bounds().width as usize) * width).next_power_of_two();
        let pixel_h = ((font.bounds().height as usize) * height).next_power_of_two();
        info!("Got string: \"{}\"", text);
        info!("size {}x{}", pixel_w, pixel_h);

        let data = vec![0.0; pixel_w * pixel_h];
        // TODO: render into data
        let rawimage = RawImage2d {
            data: Cow::from(&data[..]),
            width: pixel_w as u32,
            height: pixel_h as u32,
            format: ClientFormat::U8,
        };
        let texture = Texture2d::with_format(display, rawimage, UncompressedFloatFormat::U8, MipmapsOption::NoMipmap).unwrap();

        Poem {
            created: Instant::now(),
            color: Rgba::from(Colora::hsv(RgbHue::from(rand.next_f32() * 360.0), 1.0, 1.0, 1.0)),
            texture: texture,
            x: cmp::max(0, (rand.next_f32() * (window_size.0 as usize - pixel_w) as f32).trunc() as u16),
            y: cmp::max(0, (rand.next_f32() * (window_size.1 as usize - pixel_h) as f32).trunc() as u16),
            height: pixel_h as u16,
            width: pixel_w as u16,
        }
    }

    pub fn render_all(display: &GlutinFacade, poems: &Vec<Poem>, vertex_buffer: &glium::VertexBuffer<Vertex>, index_buffer: &glium::IndexBuffer<u16>, program: &glium::Program) {
        let size = display.get_window().unwrap().get_inner_size().unwrap();
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        for poem in poems {
            poem.render(&mut target, &vertex_buffer, &index_buffer, &program);
        }
    }

    fn render(&self, target: &mut Frame, vertex_buffer: &glium::VertexBuffer<Vertex>, index_buffer: &glium::IndexBuffer<u16>, program: &glium::Program) {
        let c = &self.color;
        let uniforms = uniform! {
            color: [c.red, c.green, c.blue, c.alpha],
            text: &self.texture,
        };
        target.draw(vertex_buffer, index_buffer, program, &uniforms, &Default::default()).unwrap();
    }
}
