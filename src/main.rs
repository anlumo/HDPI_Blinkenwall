#![allow(dead_code)]
#![allow(unused)]

use glium::glutin;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use log::{error, info};
use once_cell::sync::OnceCell;
use std::{path::PathBuf, process, process::Command, sync::mpsc};

mod config;
mod database;
mod frontpanel;
use frontpanel::LedControl;
mod emulator;
mod poetry;
mod server;
mod shadertoy;
mod states;
mod video;

const RCK: u32 = 13;
const CLR: u32 = 19;
const WRENCH: u32 = 12;

static ROMS_PATH: OnceCell<PathBuf> = OnceCell::new();

fn handle_message(
    cmd: &server::Command,
    resp: &server::connection::ResponseHandler,
    database: &database::Database,
    state_machine: &mut states::StateMachine,
) {
    match cmd {
        server::Command::ListShaders => resp.send_list(database.list().unwrap()),
        server::Command::ReadShader(ref id) => match database.read(id) {
            Ok(shader) => resp.send_shader(&shader),
            Err(error) => resp.send_error(400, &format!("{}", error)),
        },
        server::Command::WriteShader(ref id, ref shader, ref commit) => match database.update(
            id,
            shader,
            commit,
            &format!("Update shader for {}", resp.address()),
        ) {
            Ok(commit) => resp.send_commit(id, &commit),
            Err(error) => resp.send_error(400, &format!("{}", error)),
        },
        server::Command::CreateShader(ref shader) => {
            match database.add(shader, &format!("Add shader for {}", resp.address())) {
                Ok((id, commit)) => resp.send_commit(&id, &commit),
                Err(error) => resp.send_error(400, &format!("{}", error)),
            }
        }
        server::Command::RemoveShader(ref id) => match database.remove(id) {
            Ok(_) => resp.send_ok(),
            Err(error) => resp.send_error(400, &format!("{}", error)),
        },
        server::Command::ActivateShader(ref id) => {
            info!("[{}] Activating shader {}", resp.address(), id);
            match database.read(id) {
                Ok(shader) => {
                    state_machine.to_shader_toy(&shader.source);
                    resp.send_ok()
                }
                Err(error) => resp.send_error(404, error.message()),
            }
        }
        server::Command::PlayVideo(ref url) => {
            state_machine.to_video(url);
            resp.send_ok()
        }
        server::Command::TurnOff => {
            state_machine.to_off();
            resp.send_ok()
        }
        server::Command::ShowPoetry(ref text) => {
            state_machine.to_poetry(text);
            resp.send_ok()
        }
        server::Command::StartTox => {
            state_machine.to_tox();
            resp.send_ok()
        }
        server::Command::ToxMessage(ref text) => {
            state_machine.to_tox_message(text);
            resp.send_ok()
        }
        server::Command::StartEmulator(game) => {
            state_machine.to_emulator(game.clone());
            resp.send_ok()
        }
        server::Command::EmulatorInput(key, press) => {
            state_machine.emulator_input(key, *press);
            resp.send_ok()
        }
    };
}

fn main() {
    let config = match config::Config::new("blinkenwall.json") {
        Err(err) => {
            env_logger::init();
            error!("Error in config file: {}", err);
            process::exit(-1);
        }
        Ok(config) => config,
    };
    if let Err(e) = log4rs::init_file(config.logconfig.clone(), Default::default()) {
        env_logger::init();
        error!("Error: {}", e);
        process::exit(-1);
    }
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

    let (wrench, rck) = match Chip::new("/dev/gpiochip4") {
        Ok(mut chip) => {
            log::debug!("Chip = {:?}", chip);
            // the clear line has to be held high, otherwise the chip doesn't do anything
            chip.get_line(CLR)
                .and_then(|line| line.request(LineRequestFlags::OUTPUT, 1, "frontpanel"))
                .ok();
            (
                chip.get_line(WRENCH)
                    .and_then(|line| line.request(LineRequestFlags::OUTPUT, 0, "blinkenwall"))
                    .ok(),
                chip.get_line(RCK)
                    .and_then(|line| line.request(LineRequestFlags::OUTPUT, 0, "blinkenwall"))
                    .ok(),
            )
        }
        Err(err) => {
            log::error!("Failed getting GPIOs: {:?}", err);
            (None, None)
        }
    };
    log::debug!("wrench = {:?}, rck = {:?}", wrench, rck);

    let led_control = if let (Some(wrench), Some(rck)) = (wrench, rck) {
        LedControl::new(rck, wrench).ok()
    } else {
        None
    };
    ROMS_PATH.set((&config.emulator.roms).into());
    let mut state_machine = states::StateMachine::new(display, led_control, config);

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
