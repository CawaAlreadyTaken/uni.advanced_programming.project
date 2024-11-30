use std::collections::HashMap;
mod cli;
mod gui;
use std::thread;
use std::sync::Arc;

use cli::cli::run_cli;
use gui::gui::run_gui;
use wg_2024::config::Config;
use wg_2024::controller::{DroneCommand, NodeEvent};
use wg_2024::packet::NodeType;
use wg_2024::network::NodeId;

use macroquad::prelude::*;

pub struct SimulationController {
    drones_map: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>,
    servers_map: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>,
    clients_map: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>,
    receiver: Arc<Option<crossbeam_channel::Receiver<NodeEvent>>>,
    topology: Arc<Config>,
}

impl SimulationController{
    pub fn new(configuration: Config) -> Self {
        SimulationController {
            drones_map: HashMap::new(),
            servers_map: HashMap::new(),
            clients_map: HashMap::new(),
            receiver: None.into(),
            topology: configuration.into(),
        }
    }

    fn get_channel_from_node_id(&self, node_id: NodeId) -> Option<&crossbeam_channel::Sender<DroneCommand>> {
        let trial_channel = self.drones_map.get(&node_id);
        if trial_channel.is_some() {
            return trial_channel;
        }
        let trial_channel = self.servers_map.get(&node_id);
        if trial_channel.is_some() {
            return trial_channel;
        }
        let trial_channel = self.clients_map.get(&node_id);
        if trial_channel.is_some() {
            return trial_channel;
        }
        None
    }

    fn make_crash(&mut self, node_id: NodeId) {
        let channel = self.get_channel_from_node_id(node_id);
        if channel.is_none() {
            println!("[SIMULATION CONTROLLER] Node with id {} not found. Ignoring command", node_id);
            return;
        }
        let channel = channel.unwrap();
        let _ = channel.send(DroneCommand::Crash);
        println!("[SIMULATION CONTROLLER] Sent crash command to node with id {}", node_id);
    }

    fn spawn_node(&mut self, node_id: NodeId, node_type: NodeType /*metadata*/) {

    }

    fn set_packet_drop_rate(&mut self, node_id: NodeId, rate: f32) {
        let channel = self.get_channel_from_node_id(node_id);
        if channel.is_none() {
            println!("[SIMULATION CONTROLLER] Node with id {} not found. Ignoring command", node_id);
            return;
        }
        let channel = channel.unwrap();
        let _ = channel.send(DroneCommand::SetPacketDropRate(rate));
        println!("[SIMULATION CONTROLLER] Sent set_packet_drop_rate command to node with id {}", node_id);
    }

    pub fn exit(&mut self) { // Maybe this is not needed but it would be cool
        for (id, drone) in self.drones_map.iter() {
            // TODO: Send a message to each drone to stop
        }
        for (id, server) in self.servers_map.iter() {
            // TODO: Send a message to each server to stop
        }
        for (id, client) in self.clients_map.iter() {
            // TODO: Send a message to each client to stop
        }
        println!("[SIMULATION CONTROLLER] Closed all nodes, exiting simulation...");
    }

    pub fn set_receiver(&mut self, receiver: crossbeam_channel::Receiver<NodeEvent>) {
        self.receiver = Some(receiver).into();
    }

    pub fn set_drones(&mut self, nodes: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>) {
        self.drones_map = nodes;    
    }

    pub fn set_servers(&mut self, nodes: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>) {
        self.servers_map = nodes;
    }

    pub fn set_clients(&mut self, nodes: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>) {
        self.clients_map = nodes;
    }

    pub async fn start(&mut self) {
        println!("[SIMULATION CONTROLLER] SimulationController started");

        // Wait for network initializer to set up everything
        while self.receiver.is_none() {}
        println!("[SIMULATION CONTROLLER] Received info from network initializer");

        let topology_arc = Arc::clone(&self.topology);
        let receiver_arc = Arc::clone(&self.receiver);

        println!("[SIMULATION CONTROLLER] GUI task started");
        run_gui(topology_arc, receiver_arc).await;
        println!("[SIMULATION CONTROLLER] GUI task ended (this shouldn't happen)");

        println!("[SIMULATION CONTROLLER] GUI thread started");

        println!("[SIMULATION CONTROLLER] Running CLI");
        run_cli(self);
    }
}
