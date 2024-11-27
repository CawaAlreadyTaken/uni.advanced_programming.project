use std::fs;

use wg_2024::config::Config;

pub fn parse(file_path: &str) -> Config {
    let config_data = fs::read_to_string(file_path).expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us
    // to use the toml::from:str function to deserialize the config
    // file into each of them
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");
    return config
}
