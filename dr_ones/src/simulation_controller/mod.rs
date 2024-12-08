//! Simulation controller module.
//! Manages the network simulation, including node control and topology management.

use std::collections::HashMap;
use std::sync::Arc;

mod cli;
mod gui;

use cli::cli::run_cli;
use gui::gui::run_gui;
use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    network::NodeId,
};

use macroquad::prelude::*;

use crate::client::ClientCommand;

/// Manages and controls the network simulation.
///
/// The SimulationController is responsible for:
/// - Managing network topology
/// - Controlling drone, server, and client nodes
/// - Handling simulation events
/// - Coordinating between GUI and CLI interfaces
pub struct SimulationController {
    drones_map: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>,
    servers_map: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>,
    clients_map: HashMap<NodeId, crossbeam_channel::Sender<ClientCommand>>,
    receiver: Arc<Option<crossbeam_channel::Receiver<DroneEvent>>>,
    topology: Arc<Config>,
}

impl SimulationController {
    /// Creates a new SimulationController with the given configuration.
    pub fn new(configuration: Config) -> Self {
        SimulationController {
            drones_map: HashMap::new(),
            servers_map: HashMap::new(),
            clients_map: HashMap::new(),
            receiver: None.into(),
            topology: configuration.into(),
        }
    }

    /// Gets the command channel for a drone by its ID.
    fn get_channel_from_drone_id(
        &self,
        node_id: NodeId,
    ) -> Option<&crossbeam_channel::Sender<DroneCommand>> {
        self.drones_map.get(&node_id)
    }

    /// Gets the command channel for a client by its ID.
    fn get_channel_from_client_id(
        &self,
        node_id: NodeId,
    ) -> Option<&crossbeam_channel::Sender<ClientCommand>> {
        self.clients_map.get(&node_id)
    }

    /// Sends a crash command to a specified node.
    pub fn make_crash(&mut self, node_id: NodeId) {
        let channel = self.get_channel_from_drone_id(node_id);

        match channel {
            Some(channel) => {
                if let Err(e) = channel.send(DroneCommand::Crash) {
                    println!(
                        "[SIMULATION CONTROLLER] Failed to send crash command to node {}: {}",
                        node_id, e
                    );
                    return;
                }
                println!(
                    "[SIMULATION CONTROLLER] Sent crash command to node {}",
                    node_id
                );
            }
            None => {
                println!(
                    "[SIMULATION CONTROLLER] Node with id {} not found. Ignoring command",
                    node_id
                );
            }
        }
    }

    /// Spawns a new node with specified connections.
    pub fn spawn_node(&mut self, connected_node_ids: Vec<NodeId>) -> Result<(), String> {
        // TODO: Implement node spawning logic
        Ok(())
    }

    /// Sets the packet drop rate for a specified node.
    pub fn set_packet_drop_rate(&mut self, node_id: NodeId, rate: f32) {
        let channel = self.get_channel_from_drone_id(node_id);

        match channel {
            Some(channel) => {
                if let Err(e) = channel.send(DroneCommand::SetPacketDropRate(rate)) {
                    println!(
                        "[SIMULATION CONTROLLER] Failed to set packet drop rate for node {}: {}",
                        node_id, e
                    );
                    return;
                }
                println!(
                    "[SIMULATION CONTROLLER] Set packet drop rate for node {} to {}",
                    node_id, rate
                );
            }
            None => {
                println!(
                    "[SIMULATION CONTROLLER] Node with id {} not found. Ignoring command",
                    node_id
                );
            }
        }
    }

    /// Performs cleanup and shuts down the simulation.
    pub fn exit(&mut self) {
        println!("[SIMULATION CONTROLLER] Starting shutdown sequence...");

        // TODO: Send stop messages to each node type
        for (id, _) in self.drones_map.iter() {
            // Send stop message to drone
        }

        for (id, _) in self.servers_map.iter() {
            // Send stop message to server
        }

        for (id, _) in self.clients_map.iter() {
            // Send stop message to client
        }

        println!("[SIMULATION CONTROLLER] Closed all nodes, exiting simulation...");
    }

    /// Sets the event receiver for the simulation.
    pub fn set_receiver(&mut self, receiver: crossbeam_channel::Receiver<DroneEvent>) {
        self.receiver = Some(receiver).into();
    }

    /// Sets the drone command channels.
    pub fn set_drones(&mut self, nodes: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>) {
        self.drones_map = nodes;
    }

    /// Sets the client command channels.
    pub fn set_clients(
        &mut self,
        nodes: HashMap<NodeId, crossbeam_channel::Sender<ClientCommand>>,
    ) {
        self.clients_map = nodes;
    }

    /// Starts the simulation controller.
    pub async fn start(&mut self) {
        println!("[SIMULATION CONTROLLER] Starting...");

        // Wait for network initializer to set up everything
        while self.receiver.is_none() {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        println!("[SIMULATION CONTROLLER] Received info from network initializer");

        let topology_arc = Arc::clone(&self.topology);
        let receiver_arc = Arc::clone(&self.receiver);

        // TODO: Fix concurrent GUI and CLI execution
        println!("[SIMULATION CONTROLLER] GUI task starting...");
        //run_gui(topology_arc, receiver_arc).await;
        println!("[SIMULATION CONTROLLER] GUI task unavailable - running CLI only");

        println!("[SIMULATION CONTROLLER] Running CLI");
        run_cli(self);
    }
}
