use std::{collections::HashMap, fs};

use wg_2024::{config::Config, network::NodeId};

fn check_parsed_config(config: &Config) {
    for drone in config.drone.iter() {
        for connected_id in drone.connected_node_ids.iter() {
            let node = find_drone_by_node_id(connected_id, &config);
            if node.is_some() {
                if !node.unwrap().connected_node_ids.contains(&drone.id) {
                    panic!(
                        "Drone with id {} is not connected to drone with id {}",
                        connected_id, drone.id
                    );
                }
                continue;
            }
            let node = find_server_by_node_id(connected_id, &config);
            if node.is_some() {
                if !node.unwrap().connected_drone_ids.contains(&drone.id) {
                    panic!(
                        "Server with id {} is not connected to drone with id {}",
                        connected_id, drone.id
                    );
                }
                continue;
            }
            let node = find_client_by_node_id(connected_id, &config);
            if node.is_some() {
                if !node.unwrap().connected_drone_ids.contains(&drone.id) {
                    panic!(
                        "Client with id {} is not connected to drone with id {}",
                        connected_id, drone.id
                    );
                }
                continue;
            }
            panic!("Node with id {} not found in config, but it was specified as a connected node for drone with id {}", connected_id, drone.id);
        }
    }
    for client in config.client.iter() {
        for connected_id in client.connected_drone_ids.iter() {
            let node = find_drone_by_node_id(connected_id, &config);
            if node.is_some() {
                if !node.unwrap().connected_node_ids.contains(&client.id) {
                    panic!(
                        "Drone with id {} is not connected to client with id {}",
                        connected_id, client.id
                    );
                }
            } else {
                panic!("Node with id {} not found in config, but it was specified as a connected drone for client with id {}", connected_id, client.id);
            }
        }
    }
    for server in config.server.iter() {
        for connected_id in server.connected_drone_ids.iter() {
            let node = find_drone_by_node_id(connected_id, &config);
            if node.is_some() {
                if !node.unwrap().connected_node_ids.contains(&server.id) {
                    panic!(
                        "Drone with id {} is not connected to server with id {}",
                        connected_id, server.id
                    );
                }
            } else {
                panic!("Node with id {} not found in config, but it was specified as a connected drone for server with id {}", connected_id, server.id);
            }
        }
    }
}

fn find_client_by_node_id<'a>(
    node_id: &NodeId,
    config: &'a Config,
) -> Option<&'a wg_2024::config::Client> {
    for client in config.client.iter() {
        if client.id == *node_id {
            return Some(client);
        }
    }
    return None;
}

fn find_server_by_node_id<'a>(
    node_id: &NodeId,
    config: &'a Config,
) -> Option<&'a wg_2024::config::Server> {
    for server in config.server.iter() {
        if server.id == *node_id {
            return Some(server);
        }
    }
    return None;
}

fn find_drone_by_node_id<'a>(
    node_id: &NodeId,
    config: &'a Config,
) -> Option<&'a wg_2024::config::Drone> {
    for drone in config.drone.iter() {
        if drone.id == *node_id {
            return Some(drone);
        }
    }
    return None;
}

pub fn parse(file_path: &str) -> Config {
    let config_data = fs::read_to_string(file_path).expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us
    // to use the toml::from:str function to deserialize the config
    // file into each of them
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");
    check_parsed_config(&config);

    return config;
}
