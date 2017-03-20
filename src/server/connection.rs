use super::ws::{self, Sender, Handler, CloseCode, Handshake, Error, Result};
use std::sync::mpsc;
use super::Command;
use serde_json;

#[derive(Serialize, Deserialize)]
struct CommandHeader {
    cmd: String,
    id: String,
}

pub struct ResponseHandler {
    out: Sender,
    id: String,
}

pub struct Connection {
    out: Sender,
    channel: mpsc::Sender<(Command, ResponseHandler)>,
}

impl Connection {
    pub fn new(out: Sender, tx: mpsc::Sender<(Command, ResponseHandler)>) -> Connection {
        Connection { out: out, channel: tx }
    }
}

impl Handler for Connection {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> Result<()> {
        match msg {
            ws::Message::Text(text) => {
                println!("Got message {}", text);
                if let Ok(msg) = serde_json::from_str::<CommandHeader>(&text) {
                    let cmd = msg.cmd;
                    let id = msg.id;
                    println!("Got command {} with id {}", cmd, id);
                    let obj: serde_json::Value = serde_json::from_str(&text).unwrap();
                    let resp = ResponseHandler { out: self.out.clone(), id: id };
                    match cmd.as_ref() {
                        "list" => self.channel.send((Command::List, resp)).unwrap(),
                        "read" => {
                            if let serde_json::Value::String(ref key) = obj["key"] {
                                self.channel.send((Command::Read(key.clone()), resp)).unwrap();
                            }
                        },
                        "write" => {
                            if let serde_json::Value::String(ref key) = obj["key"] {
                                if let serde_json::Value::String(ref content) = obj["content"] {
                                    self.channel.send((Command::Write(key.clone(), content.clone()), resp)).unwrap();
                                }
                            }
                        },
                        "create" => {
                            if let serde_json::Value::String(ref content) = obj["content"] {
                                self.channel.send((Command::Create(content.clone()), resp)).unwrap();
                            }
                        },
                        "activate" => {
                            if let serde_json::Value::String(ref key) = obj["key"] {
                                self.channel.send((Command::Activate(key.clone()), resp)).unwrap();
                            }
                        },
                        _ => resp.send_error(404, "Unknown command").unwrap(),
                    }
                }
            },
            ws::Message::Binary(_) => (),
        };
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away   => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        }
    }

    fn on_error(&mut self, err: Error) {
        println!("The server encountered an error: {:?}", err);
    }
}

impl ResponseHandler {
    pub fn send_list(&self, ids: Vec<String>) -> Result<()> {
        self.out.send(json!({
            "id": self.id,
            "keys": "",
        }).to_string())
    }

    pub fn send_ok(&self) -> Result<()> {
        self.out.send(json!({
            "id": self.id,
            "status": "ok",
        }).to_string())
    }

    pub fn send_error(&self, code: u16, message: &str) -> Result<()> {
        self.out.send(json!({
            "id": self.id,
            "status": "error",
            "code": code,
            "message": message,
        }).to_string())
    }
}
