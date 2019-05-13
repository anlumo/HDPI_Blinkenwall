use mpv;
use glium::backend::{glutin::Display, Context, Facade};
use glutin::ContextTrait;
use std::os::raw::{c_void,c_char};
use std::ffi::CStr;
use std::rc::Rc;

pub struct Video {
    player: Box<mpv::MpvHandlerWithGl>,
}

unsafe extern "C" fn get_proc_address(arg: *mut c_void, name: *const c_char) -> *mut c_void {
    let backend = &*(arg as *mut Box<&glutin::Context>);
    let name = CStr::from_ptr(name).to_str().unwrap();
    let addr = (***backend).get_proc_address(name) as *mut c_void;
    addr
}

impl Video {
    pub fn new(display: &Display) -> Video {
        let window = display.gl_window();
        let context = window.context();
        let mut ptr = Box::new(context.clone());
        let mut mpv_builder = mpv::MpvHandlerBuilder::new().expect("Error while creating MPV builder");
        mpv_builder.set_option("ytdl", "yes").unwrap();
        // mpv_builder.set_option("ytdl-format", "worst").unwrap();
        mpv_builder.try_hardware_decoding().unwrap();
        Video {
            player: mpv_builder.build_with_gl(Some(get_proc_address), &mut ptr as *mut _ as *mut c_void).expect("Error while initializing MPV with opengl"),
        }
    }

    pub fn play(&mut self, url: &str) {
        info!("Loading URL {}", url);
        self.player.command_async(&["loadfile", &format!("ytdl://{}", url), "replace"], 1).unwrap();
    }

    pub fn stop(&mut self) {
        self.player.command_async(&["stop"], 2).unwrap();
    }

    pub fn step(&mut self, context: &Context) -> Option<mpv::Event> {
        let event = self.player.wait_event(0.0);
        match event {
            Some(_) => event,
            None => {
                let (width, height) = context.get_framebuffer_dimensions();
                if self.player.is_update_available() {
                    self.player.draw(0, width as i32, -(height as i32)).expect("Failed to draw on glutin window");
                    context.swap_buffers().unwrap();
                }
                None
            }
        }
    }
}
