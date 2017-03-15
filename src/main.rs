#[macro_use]
extern crate glium;
use glium::Surface;
use glium::glutin;
use glium::DisplayBuild;

const DISPLAY_WIDTH: u32 = 192;
const DISPLAY_HEIGHT: u32 = 144;

fn main() {
    let display = glutin::WindowBuilder::new()
        .with_depth_buffer(24)
        .with_fullscreen(glutin::get_primary_monitor())
        .build_glium()
        .unwrap();
    display.get_window().unwrap().set_inner_size(DISPLAY_WIDTH, DISPLAY_HEIGHT);

    loop {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        target.finish().unwrap();
    }
}
