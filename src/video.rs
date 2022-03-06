use glium::backend::{glutin::Display, Context};
use libmpv::{
    events::Event,
    render::{OpenGLInitParams, RenderContext, RenderParam, RenderParamApiType},
    Format, Mpv,
};
// use libmpv_sys::{
//     mpv_opengl_init_params, mpv_render_context_create, mpv_render_context_free,
//     mpv_render_context_set_update_callback, mpv_render_context_update,
// };
use log::{error, info};
use std::os::raw::c_void;

pub struct Video {
    render_context: RenderContext,
    player: Mpv,
}

fn get_proc_address(display: &Display, name: &str) -> *mut c_void {
    display.gl_window().context().get_proc_address(name) as *mut c_void
}

impl Video {
    pub fn new(display: &Display) -> Video {
        let mut player = Mpv::with_initializer(|config| {
            config
                .set_option("ytdl", "yes")
                .and_then(|_| config.set_option("idle", "yes"))
        })
        .expect("Error while creating MPV");
        let render_context = RenderContext::new(
            unsafe { player.ctx.as_mut() },
            vec![
                RenderParam::ApiType(RenderParamApiType::OpenGl),
                RenderParam::InitParams(OpenGLInitParams {
                    get_proc_address,
                    ctx: display.clone(),
                }),
            ],
        )
        .expect("Failed creating render context");
        // mpv.set_property("ytdl-format", "worst").unwrap();
        player
            .event_context()
            .observe_property("idle-active", Format::Flag, 0)
            .unwrap();
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
                self.render_context
                    .render::<Display>(0, width as _, height as _, true)
                    .expect("Failed to draw on glutin window");
                context.swap_buffers().unwrap();
                None
            }
        }
    }

    pub fn get_volume(&self) -> i64 {
        self.player.get_property("ao-volume").unwrap_or(0)
    }

    pub fn set_volume(&self, value: i64) {
        self.player.set_property("ao-volume", value).ok();
    }
}
