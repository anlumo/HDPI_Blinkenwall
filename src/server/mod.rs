pub mod connection;
use self::connection::Connection;
use log::info;
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};

pub struct ShaderData {
    pub title: String,
    pub description: String,
    pub source: String,
    pub commit: String,
}

pub enum Command {
    ListShaders,
    ReadShader(String),
    WriteShader(String, ShaderData, String),
    CreateShader(ShaderData),
    RemoveShader(String),
    ActivateShader(String),
    PlayVideo(String),
    TurnOff,
    ShowPoetry(String),
    StartTox,
    ToxMessage(String),
    StartEmulator(String),
    EmulatorInput(String, bool),
}

pub fn open_server(
    ip: &str,
    port: u16,
) -> (
    thread::JoinHandle<ws::Result<()>>,
    Receiver<(Command, Option<connection::ResponseHandler>)>,
    Sender<(Command, Option<connection::ResponseHandler>)>,
) {
    let addr = format!("{}:{}", ip, port);
    info!("Listening on {}...", addr);
    let (tx, rx) = channel();
    let tx_2 = tx.clone();
    (
        thread::Builder::new()
            .name("Websocket Server".to_string())
            .spawn(move || {
                let result = ws::listen(addr.as_str(), |out| Connection::new(out, tx.clone()));
                result
            })
            .unwrap(),
        rx,
        tx_2,
    )
}
