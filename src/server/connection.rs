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
    address: String,
}

pub struct Connection {
    out: Sender,
    channel: mpsc::Sender<(Command, ResponseHandler)>,
    address: String,
}

impl Connection {
    pub fn new(out: Sender, tx: mpsc::Sender<(Command, ResponseHandler)>) -> Connection {
        Connection { out: out, channel: tx, address: "<unknown>".to_string() }
    }
}

impl Handler for Connection {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        if let Some(addr) = shake.remote_addr().unwrap() {
            self.address = addr;
        }
        info!("[{}] Connection opened.", self.address);
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> Result<()> {
        match msg {
            ws::Message::Text(text) => {
                info!("[{}] Got message {}", self.address, text);
                if let Ok(msg) = serde_json::from_str::<CommandHeader>(&text) {
                    let cmd = msg.cmd;
                    let id = msg.id;
                    info!("[{}] Got command {} with id {}", self.address, cmd, id);
                    let obj: serde_json::Value = serde_json::from_str(&text).unwrap();
                    let resp = ResponseHandler { out: self.out.clone(), id: id, address: self.address.clone() };
                    match cmd.as_ref() {
                        "list" => self.channel.send((Command::List, resp)).unwrap(),
                        "read" => {
                            if let serde_json::Value::String(ref key) = obj["key"] {
                                self.channel.send((Command::Read(key.clone()), resp)).unwrap();
                            } else {
                                error!("[{}] read message has invalid key, ignored.", self.address);
                            }
                        },
                        "write" => {
                            if let serde_json::Value::String(ref key) = obj["key"] {
                                if let serde_json::Value::String(ref content) = obj["content"] {
                                    self.channel.send((Command::Write(key.clone(), content.clone()), resp)).unwrap();
                                } else {
                                    error!("[{}] Write message has invalid content, ignored.", self.address);
                                }
                            } else {
                                error!("[{}] Write message has invalid key, ignored.", self.address);
                            }
                        },
                        "create" => {
                            if let serde_json::Value::String(ref content) = obj["content"] {
                                self.channel.send((Command::Create(content.clone()), resp)).unwrap();
                            } else {
                                error!("[{}] Create message was invalid, ignored.", self.address);
                            }
                        },
                        "activate" => {
                            if let serde_json::Value::String(ref key) = obj["key"] {
                                self.channel.send((Command::Activate(key.clone()), resp)).unwrap();
                            } else {
                                error!("[{}] Activate message was invalid, ignored.", self.address);
                            }
                        },
                        "play_video" => {
                            if let serde_json::Value::String(ref url) = obj["url"] {
                                self.channel.send((Command::PlayVideo(url.clone()), resp)).unwrap();
                            } else {
                                error!("[{}] PlayVideo message was invalid, ignored.", self.address);
                            }
                        },
                        "stop_video" => {
                            self.channel.send((Command::StopVideo, resp)).unwrap();
                        },
                        _ => resp.send_error(404, "Unknown command").unwrap(),
                    }
                } else {
                    error!("[{}] Received invalid header in websocket message, ignored.", self.address);
                }
            },
            ws::Message::Binary(_) => error!("[{}] Received binary websocket message, ignored.", self.address),
        };
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => info!("[{}] The client is done with the connection.", self.address),
            CloseCode::Away   => info!("[{}] The client is leaving the site.", self.address),
            _ => error!("[{}] The client encountered an error: {}", self.address, reason),
        }
    }

    fn on_error(&mut self, err: Error) {
        error!("[{}] The server encountered an error: {:?}", self.address, err);
    }
}

impl ResponseHandler {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn send_list(&self, ids: Vec<String>) -> Result<()> {
        info!("[{}] Sending list", self.address);
        self.out.send(json!({
            "id": self.id,
            "keys": ids,
        }).to_string())
    }

    pub fn send_text(&self, content: &str) -> Result<()> {
        info!("[{}] Sending text", self.address);
        self.out.send(json!({
            "id": self.id,
            "data": content,
        }).to_string())
    }

    pub fn send_id(&self, name: &str, id: &str) -> Result<()> {
        info!("[{}] Sending id", self.address);
        self.out.send(json!({
            "id": self.id,
            name: id,
        }).to_string())
    }

    pub fn send_ok(&self) -> Result<()> {
        info!("[{}] Sending ok", self.address);
        self.out.send(json!({
            "id": self.id,
            "status": "ok",
        }).to_string())
    }

    pub fn send_error(&self, code: u16, message: &str) -> Result<()> {
        error!("[{}] Sending error {}: {}", self.address, code, message);
        self.out.send(json!({
            "id": self.id,
            "status": "error",
            "code": code,
            "message": message,
        }).to_string())
    }
}
