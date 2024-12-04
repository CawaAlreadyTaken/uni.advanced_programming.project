use wg_2024::config::Config;
use crossbeam_channel::{self, Sender};
use wg_2024::network::NodeId;
use std::{collections::HashMap, thread};
pub mod parser;
use crate::client::{ClientNode, ClientCommand, ClientOptions};
use crate::drone::Dr_One;
use crate::server::{ServerNode, ServerOptions};
use crate::simulation_controller::SimulationController;
use wg_2024::drone::Drone;

pub struct NetworkInitializer {}

impl NetworkInitializer {
    pub fn new() -> NetworkInitializer {
        NetworkInitializer {}
    }

    pub async fn start(&mut self) {
        println!("[NETWORK INITIALIZER] NetworkInitializer started");

        // Read and parse network initialization file
        let parsed_config: Config = parser::parse("topologies/init.toml");
        println!("[NETWORK INITIALIZER] Initializing network with config: {:?}", parsed_config);

        let mut controller_drones = HashMap::new();
        let mut controller_clients: HashMap<NodeId, Sender<ClientCommand>> = HashMap::new();

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

        for drone in parsed_config.clone().drone.into_iter() {
            // controller channel
            let (
                controller_drone_send,
                controller_drone_recv
            ) = crossbeam_channel::unbounded();
            controller_drones.insert(drone.id, controller_drone_send);
            let node_event_send = node_event_send.clone();

            // packet chanels
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
                let mut drone = Dr_One::new(
                     drone.id,
                     node_event_send,
                     controller_drone_recv,
                    packet_recv,
                    packet_send,
                    drone.pdr,
                );

                drone.run();
            }));
        }

        for client in parsed_config.clone().client.into_iter() {
            // controller channel
            let (
                controller_client_send,
                controller_client_recv
            ) = crossbeam_channel::unbounded();
            controller_clients.insert(client.id, controller_client_send);
            let node_event_send = node_event_send.clone();

            // packet chanels
            let packet_recv = packet_channels[&client.id].1.clone();
            let packet_send = client
                .connected_drone_ids
                .into_iter()
                .map(|id| (id, packet_channels[&id].0.clone()))
                .collect();

            println!(
                "[NETWORK INITIALIZER] Spawning client with id: {}",
                client.id
            );

            handles.push(thread::spawn(move || {
                let mut client = ClientNode::new(ClientOptions {
                    id: client.id,
                    controller_recv: controller_client_recv,
                    controller_send: node_event_send,
                    packet_recv,
                    packet_send,
                });

                client.run();
            }));
        }

        for server in parsed_config.clone().server.into_iter() {
            // controller channel
            let node_event_send = node_event_send.clone();

            // packet chanels
            let packet_recv = packet_channels[&server.id].1.clone();
            let packet_send = server
                .connected_drone_ids
                .into_iter()
                .map(|id| (id, packet_channels[&id].0.clone()))
                .collect();

            println!(
                "[NETWORK INITIALIZER] Spawning server with id: {}",
                server.id
            );

            handles.push(thread::spawn(move || {
                let mut server = ServerNode::new(ServerOptions {
                    id: server.id,
                    //controller_recv: controller_server_recv,
                    controller_send: node_event_send,
                    packet_recv,
                    packet_send,
                });

                server.run();
            }));
        }

        let mut simulation_controller_element = SimulationController::new(parsed_config);

        println!("[NETWORK INITIALIZER] Passing nodes to simulation controller...");
        simulation_controller_element.set_drones(controller_drones);
        simulation_controller_element.set_clients(controller_clients);
        // We don't need to send commands to servers
        println!("[NETWORK INITIALIZER] Passed nodes to simulation controller");

        println!("[NETWORK INITIALIZER] Passing receiving channel to simulation controller...");
        simulation_controller_element.set_receiver(node_event_recv);
        println!("[NETWORK INITIALIZER] Passed receiving channel to simulation controller");

        println!("[NETWORK INITIALIZER] Spawning simulation controller");
        simulation_controller_element.start().await;

        println!("[NETWORK INITIALIZER] Exiting");
    }
}
