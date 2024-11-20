use std::fs;

#[cfg(feature = "serialize")]
use serde::Deserialize;

#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Deserialize))]
pub struct Drone {
    pub id: u64,
    pub connected_drone_ids: Vec<u64>,
    pub pdr: f64,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Deserialize))]
pub struct Client {
    pub id: u64,
    pub connected_drone_ids: Vec<u64>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Deserialize))]
pub struct Server {
    pub id: u64,
    pub connected_drone_ids: Vec<u64>,
}

#[derive(Debug)]
#[cfg_attr(feature = "serialize", derive(Deserialize))]
pub struct Config {
    pub drone: Vec<Drone>,
    pub client: Vec<Client>,
    pub server: Vec<Server>,
}

pub fn parse(file_path: &str) -> Config {
    let config_data = fs::read_to_string(file_path).expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us
    // to use the toml::from:str function to deserialize the config
    // file into each of them
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");
    println!("{:?}", config);
    return config
}
