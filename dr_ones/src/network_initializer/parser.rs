//! Network configuration parser module.
//! Handles parsing and validation of network topology configuration files.

use std::fs;
use wg_2024::{
    config::{Client, Config, Drone, Server},
    network::NodeId,
};

/// Error types for network configuration parsing
#[derive(Debug)]
pub enum ConfigError {
    FileRead(std::io::Error),
    TomlParse(toml::de::Error),
    Validation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileRead(e) => write!(f, "Failed to read config file: {}", e),
            ConfigError::TomlParse(e) => write!(f, "Failed to parse TOML: {}", e),
            ConfigError::Validation(msg) => write!(f, "Configuration validation failed: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Parses and validates a network configuration file.
///
/// # Arguments
/// * `file_path` - Path to the TOML configuration file
///
/// # Returns
/// * `Result<Config, ConfigError>` - The parsed and validated configuration
pub fn parse(file_path: &str) -> Result<Config, ConfigError> {
    // Read and parse configuration file
    let config_data = fs::read_to_string(file_path).map_err(ConfigError::FileRead)?;

    let config: Config = toml::from_str(&config_data).map_err(ConfigError::TomlParse)?;

    // Validate configuration
    validate_config(&config)?;

    Ok(config)
}

/// Validates the entire network configuration.
fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // Validate drone connections
    for drone in &config.drone {
        validate_drone_connections(drone, config)?;
    }

    // Validate client connections
    for client in &config.client {
        validate_client_connections(client, config)?;
    }

    // Validate server connections
    for server in &config.server {
        validate_server_connections(server, config)?;
    }

    Ok(())
}

/// Validates connections for a drone node.
fn validate_drone_connections(drone: &Drone, config: &Config) -> Result<(), ConfigError> {
    for connected_id in &drone.connected_node_ids {
        // Check if the connected node exists and has reciprocal connection
        if let Some(connected_drone) = find_drone_by_node_id(connected_id, config) {
            if !connected_drone.connected_node_ids.contains(&drone.id) {
                return Err(ConfigError::Validation(format!(
                    "Drone {} is not connected to drone {}",
                    connected_id, drone.id
                )));
            }
        } else if let Some(connected_server) = find_server_by_node_id(connected_id, config) {
            if !connected_server.connected_drone_ids.contains(&drone.id) {
                return Err(ConfigError::Validation(format!(
                    "Server {} is not connected to drone {}",
                    connected_id, drone.id
                )));
            }
        } else if let Some(connected_client) = find_client_by_node_id(connected_id, config) {
            if !connected_client.connected_drone_ids.contains(&drone.id) {
                return Err(ConfigError::Validation(format!(
                    "Client {} is not connected to drone {}",
                    connected_id, drone.id
                )));
            }
        } else {
            return Err(ConfigError::Validation(format!(
                "Node {} not found in config, but specified as connection for drone {}",
                connected_id, drone.id
            )));
        }
    }
    Ok(())
}

/// Validates connections for a client node.
fn validate_client_connections(client: &Client, config: &Config) -> Result<(), ConfigError> {
    for connected_id in &client.connected_drone_ids {
        if let Some(drone) = find_drone_by_node_id(connected_id, config) {
            if !drone.connected_node_ids.contains(&client.id) {
                return Err(ConfigError::Validation(format!(
                    "Drone {} is not connected to client {}",
                    connected_id, client.id
                )));
            }
        } else {
            return Err(ConfigError::Validation(format!(
                "Drone {} not found in config, but specified as connection for client {}",
                connected_id, client.id
            )));
        }
    }
    Ok(())
}

/// Validates connections for a server node.
fn validate_server_connections(server: &Server, config: &Config) -> Result<(), ConfigError> {
    for connected_id in &server.connected_drone_ids {
        if let Some(drone) = find_drone_by_node_id(connected_id, config) {
            if !drone.connected_node_ids.contains(&server.id) {
                return Err(ConfigError::Validation(format!(
                    "Drone {} is not connected to server {}",
                    connected_id, server.id
                )));
            }
        } else {
            return Err(ConfigError::Validation(format!(
                "Drone {} not found in config, but specified as connection for server {}",
                connected_id, server.id
            )));
        }
    }
    Ok(())
}

/// Finds a client node by its ID.
fn find_client_by_node_id<'a>(node_id: &NodeId, config: &'a Config) -> Option<&'a Client> {
    config.client.iter().find(|client| client.id == *node_id)
}

/// Finds a server node by its ID.
fn find_server_by_node_id<'a>(node_id: &NodeId, config: &'a Config) -> Option<&'a Server> {
    config.server.iter().find(|server| server.id == *node_id)
}

/// Finds a drone node by its ID.
fn find_drone_by_node_id<'a>(node_id: &NodeId, config: &'a Config) -> Option<&'a Drone> {
    config.drone.iter().find(|drone| drone.id == *node_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_nodes() {
        let config = Config {
            drone: vec![Drone {
                id: 1,
                connected_node_ids: vec![],
                pdr: 0.0,
            }],
            client: vec![Client {
                id: 2,
                connected_drone_ids: vec![],
            }],
            server: vec![Server {
                id: 3,
                connected_drone_ids: vec![],
            }],
        };

        assert!(find_drone_by_node_id(&1, &config).is_some());
        assert!(find_client_by_node_id(&2, &config).is_some());
        assert!(find_server_by_node_id(&3, &config).is_some());
        assert!(find_drone_by_node_id(&4, &config).is_none());
    }
}
