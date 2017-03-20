extern crate ws;
mod connection;
use self::connection::Connection;
use std::thread;
use std::sync::mpsc::{channel, Receiver};

pub enum Command {
    List,
    Read(String),
    Write(String, String),
    Create(String),
    Activate(String),
}

pub fn open_server(port: u16) -> (thread::JoinHandle<ws::Result<()>>, Receiver<(Command, connection::ResponseHandler)>) {
    let addr = format!("127.0.0.1:{}", port);
    println!("Listening on {}...", addr);
    let (tx, rx) = channel::<(Command, connection::ResponseHandler)>();
    (thread::Builder::new().name("Websocket Server".to_string()).spawn(move || {
        let result = ws::listen(addr.as_str(), |out| {
            println!("Client connected.");
            Connection::new(out, tx.clone())
        });
        result
    }).unwrap(), rx)
}
