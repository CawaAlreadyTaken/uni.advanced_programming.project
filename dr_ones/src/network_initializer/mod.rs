//! Network initialization module.
//! Handles the setup and initialization of the entire network topology,
//! including drones, clients, and servers.

use crossbeam_channel::{self, Sender};
use std::{collections::HashMap, thread};
use wg_2024::{config::Config, drone::Drone, network::NodeId};

mod parser;

use crate::{
    client::{ClientCommand, ClientNode},
    drone::Dr_One,
    server::{ServerNode},
    simulation_controller::SimulationController,
};

/// Manages the initialization of the network topology and components.
pub struct NetworkInitializer {}

impl NetworkInitializer {
    /// Creates a new NetworkInitializer instance.
    pub fn new() -> Self {
        NetworkInitializer {}
    }

    /// Starts the network initialization process.
    ///
    /// # Steps:
    /// 1. Parses network configuration
    /// 2. Creates communication channels
    /// 3. Spawns network nodes (drones, clients, servers)
    /// 4. Initializes the simulation controller
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log_status("NetworkInitializer started");

        // Parse network configuration
        let config = self.load_configuration()?;
        log_status(&format!("Initializing network with config: {:?}", config));

        // Initialize channels and data structures
        let (node_event_send, node_event_recv) = crossbeam_channel::unbounded();
        let mut controller_drones = HashMap::new();
        let mut controller_clients: HashMap<NodeId, Sender<ClientCommand>> = HashMap::new();
        let packet_channels = self.create_packet_channels(&config);

        // Spawn network components
        let handles = vec![
            self.spawn_drones(
                &config,
                &packet_channels,
                &mut controller_drones,
                node_event_send.clone(),
            )?,
            self.spawn_clients(
                &config,
                &packet_channels,
                &mut controller_clients,
                node_event_send.clone(),
            )?,
            self.spawn_servers(&config, &packet_channels, node_event_send.clone())?,
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        // Initialize and start simulation controller
        self.initialize_controller(
            config,
            controller_drones,
            controller_clients,
            node_event_recv,
        )
        .await?;

        log_status("Exiting");
        Ok(())
    }

    // Private helper methods

    fn load_configuration(&self) -> Result<Config, Box<dyn std::error::Error>> {
        parser::parse("topologies/init.toml").map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }

    fn create_packet_channels(
        &self,
        config: &Config,
    ) -> HashMap<
        NodeId,
        (
            Sender<wg_2024::packet::Packet>,
            crossbeam_channel::Receiver<wg_2024::packet::Packet>,
        ),
    > {
        let mut channels = HashMap::new();

        // Create channels for all node types
        for drone in &config.drone {
            channels.insert(drone.id, crossbeam_channel::unbounded());
        }
        for client in &config.client {
            channels.insert(client.id, crossbeam_channel::unbounded());
        }
        for server in &config.server {
            channels.insert(server.id, crossbeam_channel::unbounded());
        }

        channels
    }

    fn spawn_drones(
        &self,
        config: &Config,
        packet_channels: &HashMap<
            NodeId,
            (
                Sender<wg_2024::packet::Packet>,
                crossbeam_channel::Receiver<wg_2024::packet::Packet>,
            ),
        >,
        controller_drones: &mut HashMap<NodeId, Sender<wg_2024::controller::DroneCommand>>,
        node_event_send: Sender<wg_2024::controller::DroneEvent>,
    ) -> Result<Vec<thread::JoinHandle<()>>, Box<dyn std::error::Error>> {
        let mut handles = Vec::new();

        for drone in &config.drone {
            // Create controller channel
            let (controller_drone_send, controller_drone_recv) = crossbeam_channel::unbounded();
            controller_drones.insert(drone.id, controller_drone_send);

            // Set up packet channels
            let packet_recv = packet_channels[&drone.id].1.clone();
            let packet_send = drone
                .connected_node_ids
                .iter()
                .map(|id| (*id, packet_channels[id].0.clone()))
                .collect();

            log_status(&format!("Spawning drone with id: {}", drone.id));

            let node_event_send = node_event_send.clone();
            let drone_id = drone.id;
            let pdr = drone.pdr;

            handles.push(thread::spawn(move || {
                let mut drone = Dr_One::new(
                    drone_id,
                    node_event_send,
                    controller_drone_recv,
                    packet_recv,
                    packet_send,
                    pdr,
                );
                drone.run();
            }));
        }

        Ok(handles)
    }

    fn spawn_clients(
        &self,
        config: &Config,
        packet_channels: &HashMap<
            NodeId,
            (
                Sender<wg_2024::packet::Packet>,
                crossbeam_channel::Receiver<wg_2024::packet::Packet>,
            ),
        >,
        controller_clients: &mut HashMap<NodeId, Sender<ClientCommand>>,
        node_event_send: Sender<wg_2024::controller::DroneEvent>,
    ) -> Result<Vec<thread::JoinHandle<()>>, Box<dyn std::error::Error>> {
        let mut handles = Vec::new();

        for client in &config.client {
            // Create controller channel
            let (controller_client_send, controller_client_recv) = crossbeam_channel::unbounded();
            controller_clients.insert(client.id, controller_client_send);

            // Set up packet channels
            let packet_recv = packet_channels[&client.id].1.clone();
            let packet_send = client
                .connected_drone_ids
                .iter()
                .map(|id| (*id, packet_channels[id].0.clone()))
                .collect();

            log_status(&format!("Spawning client with id: {}", client.id));

            let node_event_send = node_event_send.clone();
            let client_id = client.id;

            handles.push(thread::spawn(move || {
                let mut client = ClientNode::new(
                    client_id,
                    node_event_send,
                    controller_client_recv,
                    packet_recv,
                    packet_send,
                );
                client.run();
            }));
        }

        Ok(handles)
    }

    fn spawn_servers(
        &self,
        config: &Config,
        packet_channels: &HashMap<
            NodeId,
            (
                Sender<wg_2024::packet::Packet>,
                crossbeam_channel::Receiver<wg_2024::packet::Packet>,
            ),
        >,
        node_event_send: Sender<wg_2024::controller::DroneEvent>,
    ) -> Result<Vec<thread::JoinHandle<()>>, Box<dyn std::error::Error>> {
        let mut handles = Vec::new();

        for server in &config.server {
            // Set up packet channels
            let packet_recv = packet_channels[&server.id].1.clone();
            let packet_send = server
                .connected_drone_ids
                .iter()
                .map(|id| (*id, packet_channels[id].0.clone()))
                .collect();

            log_status(&format!("Spawning server with id: {}", server.id));

            let node_event_send = node_event_send.clone();
            let server_id = server.id;

            handles.push(thread::spawn(move || {
                let mut server = ServerNode::new(
                    server_id,
                    node_event_send,
                    packet_recv,
                    packet_send,
                );
                server.run();
            }));
        }

        Ok(handles)
    }

    async fn initialize_controller(
        &self,
        config: Config,
        controller_drones: HashMap<NodeId, Sender<wg_2024::controller::DroneCommand>>,
        controller_clients: HashMap<NodeId, Sender<ClientCommand>>,
        node_event_recv: crossbeam_channel::Receiver<wg_2024::controller::DroneEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_status("Passing nodes to simulation controller...");

        let mut simulation_controller = SimulationController::new(config);
        simulation_controller.set_drones(controller_drones);
        simulation_controller.set_clients(controller_clients);

        log_status("Passing receiving channel to simulation controller...");
        simulation_controller.set_receiver(node_event_recv);

        log_status("Starting simulation controller");
        simulation_controller.start().await;

        Ok(())
    }
}

/// Helper function for consistent status logging
fn log_status(message: &str) {
    println!("[NETWORK INITIALIZER] {}", message);
}
