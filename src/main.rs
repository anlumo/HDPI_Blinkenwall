#[macro_use] extern crate log;
extern crate env_logger;

#[macro_use]
extern crate glium;
extern crate chrono;
use glium::glutin;
use glium::DisplayBuild;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

mod shadertoy;
use shadertoy::ShaderToy;

mod server;
extern crate git2;
mod database;

extern crate mpv;
mod video;
use video::Video;

use std::sync::mpsc;
use std::process;

mod config;

// https://www.shadertoy.com/view/XssczX
// https://www.shadertoy.com/view/XlfGzH

fn main() {
    env_logger::init().unwrap();
    let config = match config::Config::new("blinkenwall.json") {
        Err(err) => {
            error!("Error in config file: {}", err);
            process::exit(-1);
        }
        Ok(config) => config
    };
    let mut database = database::Database::new(&config.database.repository);
    let display = glutin::WindowBuilder::new()
        .with_depth_buffer(24)
        .with_fullscreen(glutin::get_primary_monitor())
        .with_vsync()
        .build_glium()
        .unwrap();
    let window = display.get_window().unwrap();
    window.set_inner_size(config.display.width, config.display.height);

    let (server_thread, command_receiver) = server::open_server(config.server.port);
    let mut video = Video::new(&window);

    loop {
        match video.step(&window) {
            None => {},
            Some(evt) => info!("MPV event: {:?}", evt),
        }
        match command_receiver.try_recv() {
            Ok(message) => {
                let (cmd, resp) = message;
                match cmd {
                    server::Command::List => resp.send_list(database.list().unwrap()),
                    server::Command::Read(id) =>
                        match database.read(&id) {
                            Ok(content) => resp.send_text(&content),
                            Err(error) => resp.send_error(400, &format!("{}", error))
                        },
                    server::Command::Write(_, _) => resp.send_error(404, "Not implemented"),
                    server::Command::Create(content) =>
                        match database.add(&content, &format!("Add file for {}", resp.address())) {
                            Ok(id) => resp.send_id(&id),
                            Err(error) => resp.send_error(400, &format!("{}", error))
                        },
                    server::Command::Activate(_) => resp.send_ok(),
                    server::Command::PlayVideo(url) => {
                        video.play(&url);
                        resp.send_ok()
                    },
                    server::Command::StopVideo => {
                        video.stop();
                        resp.send_ok()
                    },
                }.unwrap();
            },
            Err(err) => match err {
                mpsc::TryRecvError::Empty => (),
                mpsc::TryRecvError::Disconnected => break,
            }
        }
    }

/*
    let mut shadertoy = ShaderToy::new(&display, FRAGMENT_SHADER);

    loop {
        shadertoy.step(&display);
    }
    */
    let _ = server_thread.join().unwrap();
}
