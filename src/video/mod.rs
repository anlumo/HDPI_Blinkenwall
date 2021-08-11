use glium::backend::{glutin::Display, Context, Facade};
use glutin::PossiblyCurrent;
use libmpv::{
    events::{Event, EventContext},
    render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType},
    Mpv,
};
// use libmpv_sys::{
//     mpv_opengl_init_params, mpv_render_context_create, mpv_render_context_free,
//     mpv_render_context_set_update_callback, mpv_render_context_update,
// };
use log::{error, info};
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::rc::Rc;

pub struct Video {
    player: Mpv,
    render_context: RenderContext,
}

fn get_proc_address(display: &Display, name: &str) -> *mut c_void {
    display.gl_window().context().get_proc_address(name) as *mut c_void
}

impl Video {
    pub fn new(display: &Display) -> Video {
        let window = display.gl_window();
        let mut player = Mpv::with_initializer(|config| config.set_option("ytdl", "yes"))
            .expect("Error while creating MPV");
        let render_context = RenderContext::new(
            unsafe { player.ctx.as_mut() },
            &[
                RenderParam::ApiType(RenderParamApiType::OpenGl),
                RenderParam::InitParams(OpenGLInitParams {
                    get_proc_address,
                    ctx: display.clone(),
                }),
            ],
        )
        .expect("Failed creating render context");
        // mpv.set_property("ytdl-format", "worst").unwrap();
        Video {
            player,
            render_context,
        }
    }

    pub fn play(&mut self, url: &str) {
        info!("Loading URL {}", url);
        self.player
            .command("loadfile", &[&format!("ytdl://{}", url), "replace"])
            .unwrap();
    }

    pub fn stop(&mut self) {
        self.player.command("stop", &[]).unwrap();
    }

    pub fn step(&mut self, context: &Context) -> Option<Event> {
        let event = self.player.event_context_mut().wait_event(0.0);
        match event {
            Some(Ok(event)) => Some(event),
            Some(Err(err)) => {
                error!("MPV Error: {}", err);
                None
            }
            None => {
                let (width, height) = context.get_framebuffer_dimensions();
                // if self.player.is_update_available() {
                //     self.player
                //         .draw(0, width as i32, -(height as i32))
                //         .expect("Failed to draw on glutin window");
                //     context.swap_buffers().unwrap();
                // }
                None
            }
        }
    }
}
