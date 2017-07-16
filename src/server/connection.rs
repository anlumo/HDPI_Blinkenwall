use super::ws::{self, Sender, Handler, CloseCode, Handshake, Error, Result, ErrorKind};
use std::sync::mpsc;
use super::Command;
use serde_json;
use server::ShaderData;

#[derive(Serialize, Deserialize)]
struct CommandHeader {
    cmd: String,
    req: String,
}

pub struct ResponseHandler {
    out: Sender,
    req: String,
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

    fn parse_shaderdata(obj : &serde_json::Value) -> Result<ShaderData> {
        if let (&serde_json::Value::String(ref title), &serde_json::Value::String(ref description), &serde_json::Value::String(ref source)) = (&obj["title"], &obj["description"], &obj["source"]) {
            Ok(ShaderData {
                title: title.to_string(),
                description: description.to_string(),
                source: source.to_string(),
            })
        } else {
            Err(Error::new(ErrorKind::Internal, "Invalid shader format"))
        }
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
                    let req = msg.req;
                    info!("[{}] Got command {} with id {}", self.address, cmd, req);
                    let obj: serde_json::Value = serde_json::from_str(&text).unwrap();
                    let resp = ResponseHandler { out: self.out.clone(), req: req, address: self.address.clone() };
                    match cmd.as_ref() {
                        "shader list" => self.channel.send((Command::ListShaders, resp)).unwrap(),
                        "shader read" => {
                            if let serde_json::Value::String(ref key) = obj["id"] {
                                self.channel.send((Command::ReadShader(key.clone()), resp)).unwrap();
                            } else {
                                error!("[{}] read message has invalid key, ignored.", self.address);
                            }
                        },
                        "shader write" => {
                            if let serde_json::Value::String(ref key) = obj["id"] {
                                match Connection::parse_shaderdata(&obj) {
                                    Ok(data) => self.channel.send((Command::WriteShader(key.clone(), data), resp)).unwrap(),
                                    Err(error) => resp.send_error(400, &error.details).unwrap()
                                }
                            } else {
                                resp.send_error(400, "Message has invalid key, ignored.").unwrap();
                            }
                        },
                        "shader create" => {
                            match Connection::parse_shaderdata(&obj) {
                                Ok(data) => self.channel.send((Command::CreateShader(data), resp)).unwrap(),
                                Err(error) => resp.send_error(400, &error.details).unwrap()
                            }
                        },
                        "shader remove" => {
                            if let serde_json::Value::String(ref key) = obj["id"] {
                                self.channel.send((Command::RemoveShader(key.clone()), resp)).unwrap();
                            } else {
                                resp.send_error(400, "Message has invalid key, ignored.").unwrap();
                            }
                        }
                        "shader activate" => {
                            if let serde_json::Value::String(ref key) = obj["id"] {
                                self.channel.send((Command::ActivateShader(key.clone()), resp)).unwrap();
                            } else {
                                resp.send_error(400, "Message has invalid key, ignored.").unwrap();
                            }
                        },
                        "video play" => {
                            if let serde_json::Value::String(ref url) = obj["url"] {
                                self.channel.send((Command::PlayVideo(url.clone()), resp)).unwrap();
                            } else {
                                error!("[{}] PlayVideo message was invalid, ignored.", self.address);
                            }
                        },
                        "video stop" => {
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
            "req": self.req,
            "ids": ids,
        }).to_string())
    }

    pub fn send_shader(&self, shader: &ShaderData) -> Result<()> {
        info!("[{}] Sending shader", self.address);
        self.out.send(json!({
            "req": self.req,
            "title": shader.title,
            "description": shader.description,
            "source": shader.source,
        }).to_string())
    }

    pub fn send_id(&self, id: &str) -> Result<()> {
        info!("[{}] Sending id", self.address);
        self.out.send(json!({
            "req": self.req,
            "key": id,
        }).to_string())
    }

    pub fn send_ok(&self) -> Result<()> {
        info!("[{}] Sending ok", self.address);
        self.out.send(json!({
            "id": self.req,
            "status": "ok",
        }).to_string())
    }

    pub fn send_error(&self, code: u16, message: &str) -> Result<()> {
        error!("[{}] Sending error {}: {}", self.address, code, message);
        self.out.send(json!({
            "id": self.req,
            "status": "error",
            "code": code,
            "message": message,
        }).to_string())
    }
}
