use glium;
use glium::Surface;
use glium::index::PrimitiveType;
use glium::backend::glutin_backend::WinRef;
use mpv;
use std::os::raw::{c_void,c_char};
use std::ffi::CStr;
use log;

pub struct Video<'a> {
    player: Box<mpv::MpvHandlerWithGl>,
    url: &'a str,
}

unsafe extern "C" fn get_proc_address(arg: *mut c_void,
                                      name: *const c_char) -> *mut c_void {
    let win = &*(arg as *mut Box<&WinRef>);
    let name = CStr::from_ptr(name).to_str().unwrap();
    let addr = win.get_proc_address(name) as *mut c_void;
    addr
}

impl<'a> Video<'a> {
    pub fn new(win: &WinRef, url: &'a str) -> Video<'a> {
        let mut ptr = Box::new(win);
        let mut mpv_builder = mpv::MpvHandlerBuilder::new().expect("Error while creating MPV builder");
        mpv_builder.set_option("ytdl", "yes");
        mpv_builder.try_hardware_decoding().unwrap();
        Video {
            player: mpv_builder.build_with_gl(Some(get_proc_address), &mut ptr as *mut _ as *mut c_void).expect("Error while initializing MPV with opengl"),
            url: url,
        }
    }

    pub fn play(&mut self) {
        info!("Loading URL {}", self.url);
        self.player.command_async(&["loadfile", &format!("ytdl://{}", self.url)], 1).unwrap();
    }

    pub fn stop(&mut self) {
        self.player.command_async(&["stop"], 2).unwrap();
    }

    pub fn step(&mut self, win: &WinRef) -> Option<mpv::Event<'a>> {
        let event = self.player.wait_event(0.0);
        match event {
            Some(_) => return event,
            None => {
                match win.get_inner_size() {
                    Some((width, height)) => {
                        if self.player.is_update_available() {
                            self.player.draw(0, width as i32, -(height as i32)).expect("Failed to draw on glutin window");
                            win.swap_buffers();
                        }
                    },
                    None => {}
                }
                return None;
            }
        }
    }
}
