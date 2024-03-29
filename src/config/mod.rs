use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct Database {
    pub repository: String,
}

#[derive(Serialize, Deserialize)]
pub struct Server {
    pub port: u16,
    pub address: String,
    pub hostname: String,
}

#[derive(Serialize, Deserialize)]
pub struct Display {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Poetry {
    pub font: String,
    pub speed: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Emulator {
    pub roms: String,
    #[serde(default)]
    pub dmg: bool,
    #[serde(default = "default_fps")]
    pub fps: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Mqtt {
    pub server: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub topic: String,
}

fn default_fps() -> u32 {
    60
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub logconfig: String,
    pub database: Database,
    pub server: Server,
    pub display: Display,
    pub poetry: Poetry,
    pub emulator: Emulator,
    pub mqtt: Option<Mqtt>,
}

impl Config {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
        let file = File::open(path)?;

        let u: Config = serde_json::from_reader(file)?;
        Ok(u)
    }
}
