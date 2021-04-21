#![allow(dead_code)]
#![allow(unused)]

use glium::glutin;
use log::{error, info};
use std::process;
use std::process::Command;
use std::sync::mpsc;

mod config;
mod database;
mod frontpanel;
mod poetry;
mod server;
mod shadertoy;
mod states;
mod video;

fn handle_message(
    cmd: &server::Command,
    resp: &server::connection::ResponseHandler,
    database: &database::Database,
    state_machine: &mut states::StateMachine,
) {
    match cmd {
        &server::Command::ListShaders => resp.send_list(database.list().unwrap()),
        &server::Command::ReadShader(ref id) => match database.read(&id) {
            Ok(shader) => resp.send_shader(&shader),
            Err(error) => resp.send_error(400, &format!("{}", error)),
        },
        &server::Command::WriteShader(ref id, ref shader, ref commit) => match database.update(
            &id,
            &shader,
            &commit,
            &format!("Update shader for {}", resp.address()),
        ) {
            Ok(commit) => resp.send_commit(&id, &commit),
            Err(error) => resp.send_error(400, &format!("{}", error)),
        },
        &server::Command::CreateShader(ref shader) => {
            match database.add(&shader, &format!("Add shader for {}", resp.address())) {
                Ok((id, commit)) => resp.send_commit(&id, &commit),
                Err(error) => resp.send_error(400, &format!("{}", error)),
            }
        }
        &server::Command::RemoveShader(ref id) => match database.remove(&id) {
            Ok(_) => resp.send_ok(),
            Err(error) => resp.send_error(400, &format!("{}", error)),
        },
        &server::Command::ActivateShader(ref id) => {
            info!("[{}] Activating shader {}", resp.address(), id);
            match database.read(&id) {
                Ok(shader) => {
                    state_machine.to_shader_toy(&shader.source);
                    resp.send_ok()
                }
                Err(error) => resp.send_error(404, &error.message()),
            }
        }
        &server::Command::PlayVideo(ref url) => {
            state_machine.to_video(&url);
            resp.send_ok()
        }
        &server::Command::TurnOff => {
            state_machine.to_off();
            resp.send_ok()
        }
        &server::Command::ShowPoetry(ref text) => {
            state_machine.to_poetry(&text);
            resp.send_ok()
        }
        &server::Command::StartTox => {
            state_machine.to_tox();
            resp.send_ok()
        }
        &server::Command::ToxMessage(ref text) => {
            state_machine.to_tox_message(&text);
            resp.send_ok()
        }
    };
}

fn main() {
    // just in case
    #[cfg(target_os = "linux")]
    Command::new("/usr/bin/sudo")
        .arg("/bin/chvt")
        .arg("1")
        .output()
        .expect("failed to execute process");

    let config = match config::Config::new("blinkenwall.json") {
        Err(err) => {
            env_logger::init();
            error!("Error in config file: {}", err);
            process::exit(-1);
        }
        Ok(config) => config,
    };
    match log4rs::init_file(config.logconfig.clone(), Default::default()) {
        Err(e) => {
            env_logger::init();
            error!("Error: {}", e);
            process::exit(-1);
        }
        _ => {}
    };
    let mut database = database::Database::new(&config.database.repository);
    let mut events_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new()
        .with_fullscreen(Some(glutin::window::Fullscreen::Borderless(
            events_loop.primary_monitor(),
        )))
        .with_inner_size(glutin::dpi::LogicalSize::new(
            config.display.width as f64,
            config.display.height as f64,
        ));
    let context = glutin::ContextBuilder::new()
        .with_depth_buffer(24)
        .with_vsync(true)
        .with_hardware_acceleration(Some(true));
    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let (server_thread, command_receiver) =
        server::open_server(&config.server.address, config.server.port);

    let mut state_machine = states::StateMachine::new(display, config);

    loop {
        state_machine.update();
        match state_machine.interval() {
            None => match command_receiver.recv() {
                Ok((cmd, resp)) => handle_message(&cmd, &resp, &database, &mut state_machine),
                Err(err) => break,
            },
            Some(timeout) => match command_receiver.recv_timeout(timeout) {
                Err(err) => match err {
                    mpsc::RecvTimeoutError::Timeout => {}
                    mpsc::RecvTimeoutError::Disconnected => break,
                },
                Ok((cmd, resp)) => handle_message(&cmd, &resp, &database, &mut state_machine),
            },
        };
    }

    let _ = server_thread.join().unwrap();
}
