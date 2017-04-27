extern crate ws;
mod connection;
use self::connection::Connection;
use std::thread;
use std::sync::mpsc::{channel, Receiver};

pub struct ShaderData {
    pub title: String,
    pub description: String,
    pub source: String,
}

pub enum Command {
    ListShaders,
    ReadShader(String),
    WriteShader(String, ShaderData),
    CreateShader(ShaderData),
    RemoveShader(String),
    ActivateShader(String),
    PlayVideo(String),
    StopVideo,
}

pub fn open_server(port: u16) -> (thread::JoinHandle<ws::Result<()>>, Receiver<(Command, connection::ResponseHandler)>) {
    let addr = format!("127.0.0.1:{}", port);
    info!("Listening on {}...", addr);
    let (tx, rx) = channel::<(Command, connection::ResponseHandler)>();
    (thread::Builder::new().name("Websocket Server".to_string()).spawn(move || {
        let result = ws::listen(addr.as_str(), |out| {
            Connection::new(out, tx.clone())
        });
        result
    }).unwrap(), rx)
}
