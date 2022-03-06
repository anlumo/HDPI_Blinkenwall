#![allow(clippy::type_complexity)]

use glium::glutin;
use gpio_cdev::{Chip, LineRequestFlags};
use log::{error, info};
use once_cell::sync::OnceCell;
use std::{path::PathBuf, process, sync::mpsc, thread};
use tokio::sync::mpsc::unbounded_channel;

mod config;
mod database;
mod frontpanel;
use frontpanel::LedControl;
mod emulator;
mod mqtt;
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
    resp: Option<&server::connection::ResponseHandler>,
    database: &database::Database,
    state_machine: &mut states::StateMachine,
) {
    match cmd {
        server::Command::ListShaders => {
            if let Some(resp) = resp {
                resp.send_list(database.list().unwrap()).ok();
            }
        }
        server::Command::ReadShader(ref id) => {
            if let Some(resp) = resp {
                match database.read(id) {
                    Ok(shader) => resp.send_shader(&shader).ok(),
                    Err(error) => resp.send_error(400, &format!("{}", error)).ok(),
                };
            }
        }
        server::Command::WriteShader(ref id, ref shader, ref commit) => match database.update(
            id,
            shader,
            commit,
            &format!("Update shader for {:?}", resp.map(|resp| resp.address())),
        ) {
            Ok(commit) => {
                if let Some(resp) = resp {
                    resp.send_commit(id, &commit).ok();
                }
            }
            Err(error) => {
                if let Some(resp) = resp {
                    resp.send_error(400, &format!("{}", error)).ok();
                }
            }
        },
        server::Command::CreateShader(ref shader) => {
            match database.add(
                shader,
                &format!("Add shader for {:?}", resp.map(|resp| resp.address())),
            ) {
                Ok((id, commit)) => {
                    if let Some(resp) = resp {
                        resp.send_commit(&id, &commit).ok();
                    }
                }
                Err(error) => {
                    if let Some(resp) = resp {
                        resp.send_error(400, &format!("{}", error)).ok();
                    }
                }
            }
        }
        server::Command::RemoveShader(ref id) => match database.remove(id) {
            Ok(_) => {
                if let Some(resp) = resp {
                    resp.send_ok().ok();
                }
            }
            Err(error) => {
                if let Some(resp) = resp {
                    resp.send_error(400, &format!("{}", error)).ok();
                }
            }
        },
        server::Command::ActivateShader(ref id) => {
            info!(
                "[{:?}] Activating shader {id}",
                resp.map(|resp| resp.address())
            );
            match database.read(id) {
                Ok(shader) => {
                    state_machine.to_shader_toy(&shader.source, &shader.title);
                    if let Some(resp) = resp {
                        resp.send_ok().ok();
                    }
                }
                Err(error) => {
                    if let Some(resp) = resp {
                        resp.send_error(404, error.message()).ok();
                    }
                }
            }
        }
        server::Command::PlayVideo(ref url) => {
            state_machine.to_video(url);
            if let Some(resp) = resp {
                resp.send_ok().ok();
            }
        }
        server::Command::TurnOff => {
            state_machine.to_off();
            if let Some(resp) = resp {
                resp.send_ok().ok();
            }
        }
        server::Command::ShowPoetry(ref text) => {
            state_machine.to_poetry(text);
            if let Some(resp) = resp {
                resp.send_ok().ok();
            }
        }
        server::Command::StartTox => {
            state_machine.to_tox();
            if let Some(resp) = resp {
                resp.send_ok().ok();
            }
        }
        server::Command::ToxMessage(ref text) => {
            state_machine.to_tox_message(text);
            if let Some(resp) = resp {
                resp.send_ok().ok();
            }
        }
        server::Command::StartEmulator(game) => {
            state_machine.to_emulator(game.clone());
            if let Some(resp) = resp {
                resp.send_ok().ok();
            }
        }
        server::Command::EmulatorInput(key, press) => {
            state_machine.emulator_input(key, *press);
            if let Some(resp) = resp {
                resp.send_ok().ok();
            }
        }
        server::Command::SetVolume(value) => {
            state_machine.set_volume(*value);
            if let Some(resp) = resp {
                resp.send_ok().ok();
            }
            if let Some(sender) = state_machine.get_sender() {
                sender.send(mqtt::State::Volume(*value as _)).ok();
            }
        }
    }
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
    let database = database::Database::new(&config.database.repository);
    let events_loop = glutin::event_loop::EventLoop::new();
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

    let (server_thread, command_receiver, command_sender) =
        server::open_server(&config.server.address, config.server.port);

    let mqtt_thread = config.mqtt.as_ref().map(|mqtt| {
        let config = mqtt.clone();
        let (tx, rx) = unbounded_channel::<mqtt::State>();
        (
            thread::Builder::new()
                .name("MQTT Interface".to_owned())
                .spawn(move || {
                    mqtt::run_mqtt(config, command_sender, rx);
                })
                .unwrap(),
            tx,
        )
    });

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
    ROMS_PATH.set((&config.emulator.roms).into()).ok();
    let mut state_machine = states::StateMachine::new(
        display,
        led_control,
        config,
        mqtt_thread.as_ref().map(|(_, sender)| sender.clone()),
    );

    loop {
        state_machine.update();
        match state_machine.interval() {
            None => match command_receiver.recv() {
                Ok((cmd, resp)) => {
                    handle_message(&cmd, resp.as_ref(), &database, &mut state_machine)
                }
                Err(_err) => break,
            },
            Some(timeout) => match command_receiver.recv_timeout(timeout) {
                Err(err) => match err {
                    mpsc::RecvTimeoutError::Timeout => {}
                    mpsc::RecvTimeoutError::Disconnected => break,
                },
                Ok((cmd, resp)) => {
                    handle_message(&cmd, resp.as_ref(), &database, &mut state_machine)
                }
            },
        };
    }

    let _ = server_thread.join().unwrap();

    if let Some((join_handle, state_sender)) = mqtt_thread {
        state_sender.send(mqtt::State::Shutdown).ok();
        join_handle.join().ok();
    }
}
