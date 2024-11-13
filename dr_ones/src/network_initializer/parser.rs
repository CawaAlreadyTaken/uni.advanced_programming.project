use serde::Deserialize;
use wgl_repo_2024::types::source_routing_header::NodeId;
use wgl_repo_2024::types::topology::nodes::ServerType;
use std::fs;

#[derive(Debug, Deserialize)]
struct ParsedDrone {
    id: NodeId,
    packet_drop_rate: f64,
    connected_node_ids: Vec<NodeId>
}

#[derive(Debug, Deserialize)]
struct ParsedClient {
    id: NodeId,
    connected_node_ids: Vec<NodeId>
}

#[derive(Debug, Deserialize)]
struct ParsedServer {
    id: NodeId,
    connected_node_ids: Vec<NodeId>,
    server_type: ServerType
}

#[derive(Debug, Deserialize)]
pub struct Config {
    drones: Vec<ParsedDrone>,
    clients: Vec<ParsedClient>,
    servers: Vec<ParsedServer>,
}

pub fn parse(file_path: &str) -> Config {
    let config_data = fs::read_to_string(file_path).expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us
    // to use the toml::from:str function to deserialize the config
    // file into each of them
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");
    println!("{:?}", config);
    println!("{:?}", config.drones[0]);
    return config
}
