use std::collections::HashMap;
mod cli;

use cli::cli::run_cli;
use wg_2024::controller::{DroneCommand, NodeEvent};
use wg_2024::packet::NodeType;
use wg_2024::network::NodeId;
use crossbeam_channel;

pub trait SimContrTrait {
    fn crash(&mut self, crashed: &str);
    fn spawn_node(&mut self, node_id: NodeId, node_type: NodeType /*metadata*/);
    fn message_sent(source: &str, target: &str /*metadata*/);
}

pub struct SimulationController {
    drones_map: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>,
    servers_map: HashMap<NodeId, crossbeam_channel::Sender<NodeEvent>>,
    clients_map: HashMap<NodeId, crossbeam_channel::Sender<NodeEvent>>,
    receiver: Option<crossbeam_channel::Receiver<NodeEvent>>,
}

impl SimContrTrait for SimulationController {
    fn crash(&mut self, crashed: &str) {
        // Implement crash logic
    }

    fn spawn_node(&mut self, node_id: NodeId, node_type: NodeType /*metadata*/) {
        // Implement spawn_node logic
    }

    fn message_sent<'a>(source: &'a str, target: &'a str /*metadata*/) {
        // Implement message_sent logic
    }
}

impl SimulationController{
    pub fn new() -> Self {
        SimulationController {
            drones_map: HashMap::new(),
            servers_map: HashMap::new(),
            clients_map: HashMap::new(),
            receiver: None
        }
    }

    pub fn exit(&mut self) {
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
        self.receiver = Some(receiver);
    }

    pub fn set_drones(&mut self, nodes: HashMap<NodeId, crossbeam_channel::Sender<DroneCommand>>) {
        self.drones_map = nodes;    
    }

    pub fn set_servers(&mut self, nodes: HashMap<NodeId, crossbeam_channel::Sender<NodeEvent>>) {
        self.servers_map = nodes;
    }

    pub fn set_clients(&mut self, nodes: HashMap<NodeId, crossbeam_channel::Sender<NodeEvent>>) {
        self.clients_map = nodes;
    }

    pub fn start(&mut self) {
        println!("[SIMULATION CONTROLLER] SimulationController started");

        // Wait for network initializer to set up everything
        while self.receiver.is_none() {}
        println!("[SIMULATION CONTROLLER] Received info from network initializer");
        
        run_cli(self);

        // TODO: Create GUI. Receive cli commands for the moment
    }
}
