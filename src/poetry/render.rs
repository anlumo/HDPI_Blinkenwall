use glium::{
    backend::glutin::Display,
    texture::{
        texture2d::Texture2d, ClientFormat, MipmapsOption, RawImage2d, UncompressedFloatFormat,
    },
    uniform,
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter, Sampler},
    Frame, Surface,
};
use log::info;
use palette::{Alpha, Hsv, IntoColor, RgbHue, Srgba};
use rand::Rng;
use std::{borrow::Cow, cmp, time::Instant, u8};
use unicode_normalization::UnicodeNormalization;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub texcoords: [f32; 2],
}

pub struct Poem {
    pub created: Instant,
    pub color: Srgba,
    texture: Texture2d,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl Poem {
    pub fn new<R: Rng>(display: &Display, font: &bdf::Font, text: &str, rand: &mut R) -> Poem {
        let window_size = display.gl_window().window().inner_size();
        let font_size = font.bounds();
        let char_size = (
            font_size.width as usize + 1_usize,
            font_size.height as usize + 1_usize,
        );

        let width = cmp::min(
            window_size.width as usize / char_size.0,
            text.lines().map(|line| line.len()).max().unwrap(),
        );
        let height = cmp::min(
            window_size.height as usize / char_size.1,
            text.lines().count(),
        );
        let real_w = cmp::min(window_size.width as usize, char_size.0 * width);
        let real_h = cmp::min(window_size.height as usize, char_size.1 * height);
        let pixel_w = real_w.next_power_of_two();
        let pixel_h = real_h.next_power_of_two();
        info!("Got string: \"{}\", size {}x{}", text, pixel_w, pixel_h);

        let mut data = vec![u8::MIN; pixel_w * pixel_h];

        let glyphs = font.glyphs();
        let mut y = char_size.1 - 1_usize;
        for line in text.lines() {
            let mut pos = 0;
            for ch in line.nfc() {
                if (pos + 1) * char_size.0 > pixel_w {
                    break;
                }
                if let Some(glyph) = glyphs.get(&ch) {
                    let bounds = glyph.bounds();
                    for ((gx, gy), value) in glyph.pixels() {
                        if value {
                            data[(bounds.x
                                + (pos * char_size.0) as i32
                                + gx as i32
                                + (y as i32 - bounds.height as i32 - bounds.y + gy as i32)
                                    * pixel_w as i32) as usize] = u8::MAX;
                        }
                    }
                    pos += 1;
                }
            }

            y += char_size.1;
            if y >= pixel_h {
                break;
            }
        }

        let rawimage = RawImage2d {
            data: Cow::from(&data[..]),
            width: pixel_w as u32,
            height: pixel_h as u32,
            format: ClientFormat::U8,
        };
        let texture = Texture2d::with_format(
            display,
            rawimage,
            UncompressedFloatFormat::U8,
            MipmapsOption::NoMipmap,
        )
        .unwrap();

        Poem {
            created: Instant::now(),
            color: Alpha {
                color: Hsv::new(
                    RgbHue::from_degrees(rand.gen_range(0..36000) as f32 / 100.0),
                    1.0,
                    1.0,
                )
                .into_color(),
                alpha: 1.0,
            },
            texture,
            x: real_w as u16 + rand.gen_range(0..(window_size.width as usize - real_w)) as u16,
            y: real_h as u16 + rand.gen_range(0..(window_size.height as usize - real_h)) as u16,
            height: pixel_h as u16,
            width: pixel_w as u16,
        }
    }

    pub fn render_all(
        display: &Display,
        poems: &[Poem],
        vertex_buffer: &glium::VertexBuffer<Vertex>,
        index_buffer: &glium::IndexBuffer<u16>,
        program: &glium::Program,
    ) {
        let size = display.gl_window().window().inner_size();
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        for poem in poems {
            poem.render(
                &mut target,
                &(size.width as u32, size.height as u32),
                vertex_buffer,
                index_buffer,
                program,
            );
        }
        target.finish().unwrap();
    }

    fn render(
        &self,
        target: &mut Frame,
        size: &(u32, u32),
        vertex_buffer: &glium::VertexBuffer<Vertex>,
        index_buffer: &glium::IndexBuffer<u16>,
        program: &glium::Program,
    ) {
        let c = &self.color;
        let uniforms = uniform! {
            color: [c.red, c.green, c.blue, c.alpha],
            text: Sampler::new(&self.texture).minify_filter(MinifySamplerFilter::Nearest).magnify_filter(MagnifySamplerFilter::Nearest),
            bounds: [self.x as f32 / size.0 as f32, self.y as f32 / size.1 as f32, self.width as f32 / size.0 as f32, self.height as f32 / size.1 as f32],
        };
        target
            .draw(
                vertex_buffer,
                index_buffer,
                program,
                &uniforms,
                &glium::DrawParameters {
                    blend: glium::draw_parameters::Blend::alpha_blending(),
                    ..Default::default()
                },
            )
            .unwrap();
    }
}
