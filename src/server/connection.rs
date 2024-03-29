use super::Command;
use crate::server::ShaderData;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{path::Path, sync::mpsc};
use ws::{self, CloseCode, Error, ErrorKind, Handler, Handshake, Result, Sender};

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
    channel: mpsc::Sender<(Command, Option<ResponseHandler>)>,
    address: String,
}

impl Connection {
    pub fn new(
        out: Sender,
        channel: mpsc::Sender<(Command, Option<ResponseHandler>)>,
    ) -> Connection {
        Connection {
            out,
            channel,
            address: "<unknown>".to_string(),
        }
    }

    fn parse_shaderdata(obj: &serde_json::Value) -> Result<ShaderData> {
        if let (
            &serde_json::Value::String(ref title),
            &serde_json::Value::String(ref description),
            &serde_json::Value::String(ref source),
        ) = (&obj["title"], &obj["description"], &obj["source"])
        {
            Ok(ShaderData {
                title: title.to_string(),
                description: description.to_string(),
                source: source.to_string(),
                commit: "".to_string(),
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
                    let resp = ResponseHandler {
                        out: self.out.clone(),
                        req,
                        address: self.address.clone(),
                    };
                    match cmd.as_ref() {
                        "shader list" => self
                            .channel
                            .send((Command::ListShaders, Some(resp)))
                            .unwrap(),
                        "shader read" => {
                            if let serde_json::Value::String(ref key) = obj["id"] {
                                self.channel
                                    .send((Command::ReadShader(key.clone()), Some(resp)))
                                    .unwrap();
                            } else {
                                error!("[{}] read message has invalid id, ignored.", self.address);
                            }
                        }
                        "shader write" => {
                            if let (
                                serde_json::Value::String(key),
                                serde_json::Value::String(commit),
                            ) = (&obj["id"], &obj["commit"])
                            {
                                match Connection::parse_shaderdata(&obj) {
                                    Ok(data) => self
                                        .channel
                                        .send((
                                            Command::WriteShader(
                                                key.clone(),
                                                data,
                                                commit.to_string(),
                                            ),
                                            Some(resp),
                                        ))
                                        .unwrap(),
                                    Err(error) => resp.send_error(400, &error.details).unwrap(),
                                }
                            } else {
                                resp.send_error(400, "Message has invalid id or commit, ignored.")
                                    .unwrap();
                            }
                        }
                        "shader create" => match Connection::parse_shaderdata(&obj) {
                            Ok(data) => self
                                .channel
                                .send((Command::CreateShader(data), Some(resp)))
                                .unwrap(),
                            Err(error) => resp.send_error(400, &error.details).unwrap(),
                        },
                        "shader remove" => {
                            if let serde_json::Value::String(key) = &obj["id"] {
                                self.channel
                                    .send((Command::RemoveShader(key.clone()), Some(resp)))
                                    .unwrap();
                            } else {
                                resp.send_error(400, "Message has invalid id, ignored.")
                                    .unwrap();
                            }
                        }
                        "shader activate" => {
                            if let serde_json::Value::String(key) = &obj["id"] {
                                self.channel
                                    .send((Command::ActivateShader(key.clone()), Some(resp)))
                                    .unwrap();
                            } else {
                                resp.send_error(400, "Message has invalid id, ignored.")
                                    .unwrap();
                            }
                        }
                        "video play" => {
                            if let serde_json::Value::String(url) = &obj["url"] {
                                self.channel
                                    .send((Command::PlayVideo(url.clone()), Some(resp)))
                                    .unwrap();
                            } else {
                                error!(
                                    "[{}] PlayVideo message was invalid, ignored.",
                                    self.address
                                );
                            }
                        }
                        "turnoff" => {
                            self.channel.send((Command::TurnOff, Some(resp))).unwrap();
                        }
                        "show poetry" => {
                            if let serde_json::Value::String(text) = &obj["text"] {
                                self.channel
                                    .send((Command::ShowPoetry(text.clone()), Some(resp)))
                                    .unwrap();
                            } else {
                                self.channel
                                    .send((Command::ShowPoetry(String::new()), Some(resp)))
                                    .unwrap();
                            }
                        }
                        "tox start" => {
                            self.channel.send((Command::StartTox, Some(resp))).unwrap();
                        }
                        "tox message" => {
                            if let serde_json::Value::String(text) = &obj["text"] {
                                self.channel
                                    .send((Command::ToxMessage(text.clone()), Some(resp)))
                                    .unwrap();
                            } else {
                                self.channel
                                    .send((Command::ToxMessage(String::new()), Some(resp)))
                                    .unwrap();
                            }
                        }
                        "emulator list" => {
                            resp.send_emulator_list(crate::ROMS_PATH.get().unwrap())
                                .unwrap();
                        }
                        "emulator start" => {
                            if let serde_json::Value::String(rom) = &obj["rom"] {
                                self.channel
                                    .send((Command::StartEmulator(rom.clone()), Some(resp)))
                                    .unwrap();
                            } else {
                                resp.send_error(400, "Message needs string in rom key, ignored.")
                                    .unwrap();
                            }
                        }
                        "emulator input" => {
                            if let (
                                serde_json::Value::String(key),
                                serde_json::Value::Bool(press),
                            ) = (&obj["key"], &obj["press"])
                            {
                                self.channel
                                    .send((Command::EmulatorInput(key.clone(), *press), Some(resp)))
                                    .unwrap();
                            } else {
                                resp.send_error(
                                    400,
                                    "Message needs key and press entries, ignored.",
                                )
                                .unwrap();
                            }
                        }
                        _ => resp.send_error(404, "Unknown command").unwrap(),
                    }
                } else {
                    error!(
                        "[{}] Received invalid header in websocket message, ignored.",
                        self.address
                    );
                }
            }
            ws::Message::Binary(_) => error!(
                "[{}] Received binary websocket message, ignored.",
                self.address
            ),
        };
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => {
                info!("[{}] The client is done with the connection.", self.address)
            }
            CloseCode::Away => info!("[{}] The client is leaving the site.", self.address),
            _ => error!(
                "[{}] The client encountered an error: {}",
                self.address, reason
            ),
        }
    }

    fn on_error(&mut self, err: Error) {
        error!(
            "[{}] The server encountered an error: {:?}",
            self.address, err
        );
    }
}

impl ResponseHandler {
    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn send_list(&self, ids: Vec<String>) -> Result<()> {
        info!("[{}] Sending list", self.address);
        self.out.send(
            json!({
                "req": self.req,
                "ids": ids,
            })
            .to_string(),
        )
    }

    pub fn send_shader(&self, shader: &ShaderData) -> Result<()> {
        info!("[{}] Sending shader", self.address);
        self.out.send(
            json!({
                "req": self.req,
                "title": shader.title,
                "description": shader.description,
                "source": shader.source,
                "commit": shader.commit,
            })
            .to_string(),
        )
    }

    // pub fn send_id(&self, id: &str) -> Result<()> {
    //     info!("[{}] Sending id", self.address);
    //     self.out.send(
    //         json!({
    //             "req": self.req,
    //             "id": id,
    //         })
    //         .to_string(),
    //     )
    // }

    pub fn send_commit(&self, id: &str, commit: &str) -> Result<()> {
        info!("[{}] Sending commit", self.address);
        self.out.send(
            json!({
                "req": self.req,
                "id": id,
                "commit": commit,
            })
            .to_string(),
        )
    }

    pub fn send_emulator_list(&self, path: impl AsRef<Path>) -> Result<()> {
        self.out.send(
            json!({
                "req": self.req,
                "roms": crate::emulator::Emulator::available_roms(path)?,
                "status": "ok"
            })
            .to_string(),
        )
    }

    pub fn send_ok(&self) -> Result<()> {
        info!("[{}] Sending ok", self.address);
        self.out.send(
            json!({
                "req": self.req,
                "status": "ok",
            })
            .to_string(),
        )
    }

    pub fn send_error(&self, code: u16, message: &str) -> Result<()> {
        error!("[{}] Sending error {}: {}", self.address, code, message);
        self.out.send(
            json!({
                "req": self.req,
                "status": "error",
                "code": code,
                "message": message,
            })
            .to_string(),
        )
    }
}
