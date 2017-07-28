use serde_json;
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
    pub address: String,
    pub port: u16,
    pub font: String,
    pub speed: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub database: Database,
    pub server: Server,
    pub display: Display,
    pub poetry: Poetry,
}

impl Config {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Config, Box<Error>> {
        let file = File::open(path)?;

        let u : Config = serde_json::from_reader(file)?;
        Ok(u)
    }
}
