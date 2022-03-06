use std::{sync::mpsc::Sender, time::Duration};

use rumqttc::{
    AsyncClient, ClientError, ConnectionError, Event, EventLoop, MqttOptions, Packet, QoS,
};
use serde::Deserialize;
use tokio::{select, sync::mpsc::UnboundedReceiver};

use crate::server::{connection, Command};

pub enum State {
    PlayVideo(String),
    Tox,
    Poetry,
    ShaderToy(String),
    Emulator,
    Stopped,
    Shutdown,
    Volume(u8),
}

enum Error {
    ClientError(ClientError),
    ConnectionError(ConnectionError),
}

impl From<ClientError> for Error {
    fn from(err: ClientError) -> Self {
        Self::ClientError(err)
    }
}

impl From<ConnectionError> for Error {
    fn from(err: ConnectionError) -> Self {
        Self::ConnectionError(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientError(err) => err.fmt(f),
            Self::ConnectionError(err) => err.fmt(f),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "command", content = "args", rename_all = "snake_case")]
enum MqttCommand {
    Play,
    Stop,
    VolumeSet { volume: String },
}

async fn main_loop(
    client: &AsyncClient,
    topic: &str,
    mut eventloop: EventLoop,
    state_receiver: &mut UnboundedReceiver<State>,
    command_sender: &Sender<(Command, Option<connection::ResponseHandler>)>,
) -> Result<(), Error> {
    loop {
        select!(
            event = eventloop.poll() => {
                if let Event::Incoming(Packet::Publish(event)) = event? {
                    if let Some(json) = String::from_utf8(event.payload.to_vec()).ok().and_then(|json| serde_json::from_str::<MqttCommand>(&json).ok()) {
                        match json {
                            MqttCommand::Stop => {
                                command_sender.send((Command::TurnOff, None)).ok();
                            }
                            MqttCommand::Play => {
                                log::info!("Got play command!");
                            }
                            MqttCommand::VolumeSet { volume } => {
                                if let Ok(value) = volume.parse() {
                                    command_sender.send((Command::SetVolume(value), None)).ok();
                                }
                            }
                        }
                    }
                }
            }
            state = state_receiver.recv() => {
                match state {
                    Some(State::PlayVideo(url)) => {
                        client.publish(format!("{topic}/TITLE"), QoS::AtLeastOnce, true, url.as_bytes()).await?;
                        client.publish(format!("{topic}/STATUS"), QoS::AtLeastOnce, true, r"playing").await?;
                    }
                    Some(State::Tox) => {
                        client.publish(format!("{topic}/TITLE"), QoS::AtLeastOnce, true, r"Tox").await?;
                        client.publish(format!("{topic}/STATUS"), QoS::AtLeastOnce, true, r"playing").await?;
                    }
                    Some(State::Poetry) => {
                        client.publish(format!("{topic}/TITLE"), QoS::AtLeastOnce, true, r"Poetry").await?;
                        client.publish(format!("{topic}/STATUS"), QoS::AtLeastOnce, true, r"playing").await?;
                    }
                    Some(State::ShaderToy(title)) => {
                        client.publish(format!("{topic}/TITLE"), QoS::AtLeastOnce, true, title.as_bytes()).await?;
                        client.publish(format!("{topic}/STATUS"), QoS::AtLeastOnce, true, r"playing").await?;
                    }
                    Some(State::Emulator) => {
                        client.publish(format!("{topic}/TITLE"), QoS::AtLeastOnce, true, r"Emulator").await?;
                        client.publish(format!("{topic}/STATUS"), QoS::AtLeastOnce, true, r"playing").await?;
                    }
                    Some(State::Stopped) => {
                        client.publish(format!("{topic}/STATUS"), QoS::AtLeastOnce, true, r"stopped").await?;
                        client.publish(format!("{topic}/TITLE"), QoS::AtLeastOnce, true, []).await?;
                    }
                    Some(State::Volume(value)) => {
                        client.publish(format!("{topic}/VOLUME"), QoS::AtLeastOnce, true, value.to_string()).await?;
                    }
                    Some(State::Shutdown) | None => {
                        client.publish(format!("{topic}/STATUS"), QoS::AtLeastOnce, true, r"off").await?;
                        client.disconnect().await.ok();
                        return Ok(());
                    }
                }
            }
        );
    }
}

pub fn run_mqtt(
    config: crate::config::Mqtt,
    command_sender: Sender<(Command, Option<connection::ResponseHandler>)>,
    mut state_receiver: UnboundedReceiver<State>,
) {
    let topic = config.topic;
    let mut mqttoptions =
        MqttOptions::new("blinkenwall", config.server, config.port.unwrap_or(1883));
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    if let (Some(username), Some(password)) = (config.username, config.password) {
        mqttoptions.set_credentials(username, password);
    }

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            loop {
                let (client, eventloop) = AsyncClient::new(mqttoptions.clone(), 10);
                client
                    .subscribe(format!("{topic}/command"), QoS::AtMostOnce)
                    .await
                    .expect("Failed to subscribe to MQTT topic");
                if let Err(err) = main_loop(
                    &client,
                    &topic,
                    eventloop,
                    &mut state_receiver,
                    &command_sender,
                )
                .await
                {
                    log::error!("MQTT connection error: {err}");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                } else {
                    break;
                }
            }
        });
}
