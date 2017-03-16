#[macro_use]
extern crate glium;
use glium::Surface;
use glium::glutin;
use glium::DisplayBuild;
use glium::index::PrimitiveType;

const DISPLAY_WIDTH: u32 = 192;
const DISPLAY_HEIGHT: u32 = 144;

const VERTEX_SHADER: &'static str = "#version 140

in vec2 position;
in vec2 texcoords;

out vec2 vTexCoords;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vTexCoords = texcoords;
}
";

const FRAGMENT_SHADER: &'static str = "#version 140

in vec2 vTexCoords;
out vec4 f_color;

void main() {
    f_color = vec4(vTexCoords, 0.0, 1.0);
}
";

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    texcoords: [f32; 2],
}

fn main() {
    let display = glutin::WindowBuilder::new()
        .with_depth_buffer(24)
        .with_fullscreen(glutin::get_primary_monitor())
        .build_glium()
        .unwrap();
    display.get_window().unwrap().set_inner_size(DISPLAY_WIDTH, DISPLAY_HEIGHT);

    implement_vertex!(Vertex, position, texcoords);

    let vertex_buffer = glium::VertexBuffer::new(&display, &[
        Vertex { position: [-1.0, -1.0], texcoords: [ 0.0, 0.0 ] },
        Vertex { position: [-1.0,  1.0], texcoords: [ 0.0, 1.0 ] },
        Vertex { position: [ 1.0,  1.0], texcoords: [ 1.0, 1.0 ] },
        Vertex { position: [ 1.0, -1.0], texcoords: [ 1.0, 0.0 ] },
        ]).unwrap();
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &[0u16, 1, 2, 2, 3, 0]).unwrap();

    let program = program!(&display, 140 => { vertex: VERTEX_SHADER, fragment: FRAGMENT_SHADER }).unwrap();

    loop {
        let uniforms = uniform! {};
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    }
}
