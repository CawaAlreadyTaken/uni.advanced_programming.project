use wg_2024::config::Config;
use wg_2024::controller::DroneCommand;
use wg_2024::controller::NodeEvent;
use wg_2024::drone::DroneOptions;
use wg_2024::network::NodeId;
use wg_2024::packet::Message;
use wg_2024::packet::Packet;
use crossbeam_channel;
use std::{collections::HashMap, thread};
mod parser;
use crate::drone::Dr_One;
use crate::simulation_controller::SimulationController;
use wg_2024::drone::Drone;
//use crate::types::message::MessageContent;
//use crate::types::NodeId;
//use crate::types::SourceRoutingHeader;

pub struct NetworkInitializer {}

impl NetworkInitializer {
    pub fn new() -> NetworkInitializer {
        NetworkInitializer {}
    }

    pub fn start(&mut self) {
        println!("[NETWORK INITIALIZER] NetworkInitializer started");

        // Read and parse network initialization file
        let parsed_config: Config = parser::parse("init.toml");
        println!("[NETWORK INITIALIZER] Initializing network with config: {:?}", parsed_config);

        let mut controller_drones = HashMap::new();
        let (node_event_send, node_event_recv) = crossbeam_channel::unbounded();

        let mut packet_channels = HashMap::new();
        for drone in parsed_config.drone.iter() {
            packet_channels.insert(drone.id, crossbeam_channel::unbounded());
        }
        for client in parsed_config.client.iter() {
            packet_channels.insert(client.id, crossbeam_channel::unbounded());
        }
        for server in parsed_config.server.iter() {
            packet_channels.insert(server.id, crossbeam_channel::unbounded());
        }

        let mut handles = Vec::new();

        for drone in parsed_config.drone.into_iter() {
            // controller
            let (
                controller_drone_send,
                controller_drone_recv
            ) = crossbeam_channel::unbounded();
            controller_drones.insert(drone.id, controller_drone_send);
            let node_event_send = node_event_send.clone();

            // packet
            let packet_recv = packet_channels[&drone.id].1.clone();
            let packet_send = drone
                .connected_node_ids
                .into_iter()
                .map(|id| (id, packet_channels[&id].0.clone()))
                .collect();

            println!(
                "[NETWORK INITIALIZER] Spawning drone with id: {}",
                drone.id
            );

            handles.push(thread::spawn(move || {
                let mut drone = Dr_One::new(DroneOptions {
                    id: drone.id,
                    controller_recv: controller_drone_recv,
                    controller_send: node_event_send,
                    packet_recv,
                    packet_send,
                    pdr: drone.pdr,
                });

                drone.run();
            }));
        }

        // TODO: spawn servers and clients

        let mut simulation_controller_element = SimulationController::new();

        println!("[NETWORK INITIALIZER] Passing nodes to simulation controller...");
        simulation_controller_element.set_drones(controller_drones);
        println!("[NETWORK INITIALIZER] Passed nodes to simulation controller");

        println!("[NETWORK INITIALIZER] Passing receiving channel to simulation controller...");
        simulation_controller_element.set_receiver(node_event_recv);
        println!("[NETWORK INITIALIZER] Passed receiving channel to simulation controller");

        thread::spawn(move || {
            run_simulation_controller(simulation_controller_element);
        });
        println!("[NETWORK INITIALIZER] Simulation controller thread spawned");

        println!("[NETWORK INITIALIZER] Waiting for nodes to finish...");
        while let Some(handle) = handles.pop() {
            handle.join().unwrap();
        }
        println!("[NETWORK INITIALIZER] All nodes finished. Exiting");
    }
}


fn run_simulation_controller(mut simulation_controller_element: SimulationController) {
    simulation_controller_element.start();
}