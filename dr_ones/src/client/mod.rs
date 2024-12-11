//! Client node implementation module.
//! Handles client-side network operations, topology discovery, and packet management.

use crate::utils::NetworkUtils;
use crossbeam_channel::{select_biased, Receiver, Sender};
use indexmap::IndexSet;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{collections::HashMap, fs::OpenOptions, io::Write};
use wg_2024::{
    config::{Client, Config, Drone},
    controller::DroneEvent,
    network::{NodeId, SourceRoutingHeader},
    packet::{Ack, FloodRequest, NodeType, Packet, PacketType},
};

/// Available commands that can be sent to a client node
pub enum ClientCommand {
    GetFilesList,
    // Add more commands as needed
}

/// Client node implementation
pub struct ClientNode {
    id: NodeId,
    sim_contr_send: Sender<DroneEvent>,
    sim_contr_recv: Receiver<ClientCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    seen_flood_ids: IndexSet<u64>,
    topology: Option<Config>,
    random_generator: StdRng,
}

impl NetworkUtils for ClientNode {
    fn get_id(&self) -> NodeId {
        self.id
    }

    fn get_packet_senders(&self) -> &HashMap<NodeId, Sender<Packet>> {
        &self.packet_send
    }

    fn get_random_generator(&mut self) -> &mut StdRng {
        &mut self.random_generator
    }
}

impl ClientNode {
    /// Creates a new client node with the given parameters
    pub fn new(
        id: NodeId,
        controller_send: Sender<DroneEvent>,
        controller_recv: Receiver<ClientCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        Self {
            id,
            sim_contr_send: controller_send,
            sim_contr_recv: controller_recv,
            packet_recv,
            packet_send,
            seen_flood_ids: IndexSet::new(),
            topology: None,
            random_generator: StdRng::from_entropy(),
        }
    }

    /// Main event loop for the client node
    pub fn run(&mut self) {
        self.initialize_topology(); // TODO: Is this really the best approach? Can't we initialize the topology like this in the constructor?
        self.send_flood_request();

        loop {
            select_biased!(
                recv(self.sim_contr_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        self.handle_command(command);
                    }
                },
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        self.handle_packet(packet);
                    }
                }
            );
        }
    }

    /// Handles incoming commands
    fn handle_command(&mut self, command: ClientCommand) {
        match command {
            ClientCommand::GetFilesList => {
                log_status(self.id, "GetFilesList command received.");
            }
        }
    }

    /// Handles incoming packets
    fn handle_packet(&mut self, packet: Packet) {
        // TODO: Distinguish between routed and non-routed packets
        match &packet.pack_type {
            PacketType::Nack(ref _nack) => {
                log_status(self.id, "Nack received.");
            }
            PacketType::Ack(ref _ack) => {
                log_status(self.id, "Ack received.");
            }
            PacketType::MsgFragment(ref _fragment) => self.handle_fragment(packet),
            PacketType::FloodRequest(ref _flood_req) => self.handle_flood_request(packet),
            PacketType::FloodResponse(ref _flood_res) => self.update_topology(packet),
        }
    }

    /// Sends a flood request to discover network topology
    fn send_flood_request(&mut self) {
        let flood_id = self.random_generator.gen();

        let flood_request = FloodRequest {
            flood_id,
            initiator_id: self.id,
            path_trace: vec![(self.id, NodeType::Client)],
        };

        let source_routing_header = SourceRoutingHeader {
            hop_index: 0,
            hops: vec![self.id],
        };

        let packet = Packet {
            pack_type: PacketType::FloodRequest(flood_request),
            routing_header: source_routing_header,
            session_id: self.random_generator.gen(),
        };

        // Broadcast to all connected nodes
        let mut success = true;
        for (&node_id, sender) in &self.packet_send {
            if let Err(e) = sender.send(packet.clone()) {
                log_status(self.id, &format!(
                    "Failed to send flood request to NodeId {}: {:?}",
                    node_id, e
                ));
                success = false;
            }
        }

        if success {
            self.seen_flood_ids.insert(flood_id);
        }
    }

    // TODO: Code duplication with server node
    /// Handles an incoming flood request
    fn handle_flood_request(&mut self, packet: Packet) {
        if let PacketType::FloodRequest(mut flood_request) = packet.pack_type.clone() {
            flood_request.path_trace.push((self.id, NodeType::Client));
            let response_packet = self.build_flood_response(packet, flood_request.path_trace);
            self.forward_packet(response_packet);
        }
    }

    // TODO: Code duplication with server node
    /// Handles incoming message fragments
    fn handle_fragment(&mut self, packet: Packet) {
        log_status(self.id, "MsgFragment received. Sending an ack...");
        let ack = self.build_ack(packet);
        self.forward_packet(ack);
    }

    // TODO: Code duplication with server node
    /// Builds an acknowledgment packet
    fn build_ack(&self, packet: Packet) -> Packet {
        let frag_index = if let PacketType::MsgFragment(fragment) = &packet.pack_type {
                    fragment.fragment_index
                } else {
                    log_status(self.id, "Error: attempt of building an ack on a non-fragment packet.");
                    return packet; // TODO: or handle the error appropriately
                };

        let ack = Ack {
            fragment_index: frag_index,
        };

        let mut response = Packet {
            pack_type: PacketType::Ack(ack),
            routing_header: packet.routing_header,
            session_id: packet.session_id,
        };

        self.reverse_packet_routing_direction(&mut response);
        response
    }

    // TODO: Code duplication with server and drone node
    /// Reverses the routing direction of a packet
    fn reverse_packet_routing_direction(&self, packet: &mut Packet) {
        let mut hops = packet.routing_header.hops[..packet.routing_header.hop_index + 1].to_vec();
        hops.reverse();

        packet.routing_header = SourceRoutingHeader { hop_index: 1, hops };
    }

    /// Initializes the topology with known connections
    fn initialize_topology(&mut self) {
        let neighbour_ids: Vec<NodeId> = self.packet_send.keys().copied().collect();

        let drones: Vec<Drone> = neighbour_ids
            .iter()
            .map(|&id| Drone {
                id,
                connected_node_ids: vec![self.id],
                pdr: 0.27, // TODO: Make configurable
            })
            .collect();

        let this_client = Client {
            id: self.id,
            connected_drone_ids: neighbour_ids,
        };

        self.topology = Some(Config {
            drone: drones,
            client: vec![this_client],
            server: vec![],
        });
    }

    /// Updates topology based on flood response
    fn update_topology(&mut self, packet: Packet) {
        if let PacketType::FloodResponse(flood_response) = packet.pack_type {
            if !self.seen_flood_ids.contains(&flood_response.flood_id) {
                // TODO: Handle this error more gracefully
                panic!("Received flood response for unknown flood request!");
            }

            if self.seen_flood_ids.is_empty()
                || flood_response.flood_id != *self.seen_flood_ids.last().unwrap()
            {
                log_status(self.id, "Ignoring old flood response");
                return;
            }

            if let Some(topology) = self.topology.clone().as_mut() {
                self.update_topology_with_response(topology, &flood_response.path_trace);
                log_status(self.id, &self.get_topology_print_string(packet.session_id));
            }
        }
    }

    /// Updates topology with information from a flood response
    fn update_topology_with_response(
        &self,
        topology: &mut Config,
        path_trace: &[(NodeId, NodeType)],
    ) {
        for (i, current) in path_trace.iter().enumerate() {
            match current.1 {
                NodeType::Client => {
                    self.update_client_connections(topology, current.0, i, path_trace)
                }
                NodeType::Server => {
                    self.update_server_connections(topology, current.0, i, path_trace)
                }
                NodeType::Drone => {
                    self.update_drone_connections(topology, current.0, i, path_trace)
                }
            }
        }
    }

    /// Updates client connections in the topology
    fn update_client_connections(
        &self,
        topology: &mut Config,
        client_id: NodeId,
        index: usize,
        path_trace: &[(NodeId, NodeType)],
    ) {
        if let Some(client) = topology.client.iter_mut().find(|c| c.id == client_id) {
            if index > 0 {
                if let Some(prev_node) = path_trace.get(index - 1) {
                    if (!client.connected_drone_ids.contains(&prev_node.0)) {
                        client.connected_drone_ids.push(prev_node.0);
                    }
                }
            }
            if index < path_trace.len() - 1 {
                if let Some(next_node) = path_trace.get(index + 1) {
                    if (!client.connected_drone_ids.contains(&next_node.0)) {
                        client.connected_drone_ids.push(next_node.0);
                    }
                }
            }
        }
    }

    /// Updates server connections in the topology
    fn update_server_connections(
        &self,
        topology: &mut Config,
        server_id: NodeId,
        index: usize,
        path_trace: &[(NodeId, NodeType)],
    ) {
        if let Some(server) = topology.server.iter_mut().find(|s| s.id == server_id) {
            if index > 0 {
                if let Some(prev_node) = path_trace.get(index - 1) {
                    if (!server.connected_drone_ids.contains(&prev_node.0)) {
                        server.connected_drone_ids.push(prev_node.0);
                    }
                }
            }
            if index < path_trace.len() - 1 {
                if let Some(next_node) = path_trace.get(index + 1) {
                    if (!server.connected_drone_ids.contains(&next_node.0)) {
                        server.connected_drone_ids.push(next_node.0);
                    }
                }
            }
        }
    }

    /// Updates drone connections in the topology
    fn update_drone_connections(
        &self,
        topology: &mut Config,
        drone_id: NodeId,
        index: usize,
        path_trace: &[(NodeId, NodeType)],
    ) {
        if let Some(drone) = topology.drone.iter_mut().find(|d| d.id == drone_id) {
            if index > 0 {
                if let Some(prev_node) = path_trace.get(index - 1) {
                    if (!drone.connected_node_ids.contains(&prev_node.0)) {
                        drone.connected_node_ids.push(prev_node.0);
                    }
                }
            }
            if index < path_trace.len() - 1 {
                if let Some(next_node) = path_trace.get(index + 1) {
                    if (!drone.connected_node_ids.contains(&next_node.0)) {
                        drone.connected_node_ids.push(next_node.0);
                    }
                }
            }
        }
    }

    /// Generates a string representation of the current topology
    fn get_topology_print_string(&self, session_id: u64) -> String {
        let mut output = String::new();

        if let Some(topology) = &self.topology {
            output.push_str("--------------------------------------\n");
            output.push_str(&format!(
                "NODE {} TOPOLOGY after message with sess_id:{}\n",
                self.id, session_id
            ));

            // Add clients section
            output.push_str("---------------\nCLIENTS\n");
            let mut clients = topology.client.clone();
            clients.sort_by_key(|c| c.id);
            for client in clients {
                let mut drone_ids = client.connected_drone_ids;
                drone_ids.sort();
                output.push_str(&format!("{} -> {:?}\n", client.id, drone_ids));
            }

            // Add drones section
            output.push_str("---------------\nDRONES\n");
            let mut drones = topology.drone.clone();
            drones.sort_by_key(|d| d.id);
            for drone in drones {
                let mut node_ids = drone.connected_node_ids;
                node_ids.sort();
                output.push_str(&format!("{} -> {:?}\n", drone.id, node_ids));
            }

            // Add servers section
            output.push_str("---------------\nSERVERS\n");
            let mut servers = topology.server.clone();
            servers.sort_by_key(|s| s.id);
            for server in servers {
                let mut drone_ids = server.connected_drone_ids;
                drone_ids.sort();
                output.push_str(&format!("{} -> {:?}\n", server.id, drone_ids));
            }

            output.push_str("--------------------------------------\n");
        }

        output
    }
}

impl ClientNode {
    pub fn run_test_ack_sent_back(&mut self) {
        let log_path = "tests/ack_sent_back/log.txt";
        let mut log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)
            .expect("Failed to open or create log file");

        let source_routing_header = wg_2024::network::SourceRoutingHeader {
            hop_index: 1,
            hops: vec![10, 20, 30, 40],
        };

        let packet = wg_2024::packet::Packet {
            pack_type: PacketType::MsgFragment(wg_2024::packet::Fragment {
                fragment_index: 0,
                total_n_fragments: 0,
                length: 0,
                data: [0; wg_2024::packet::FRAGMENT_DSIZE],
            }),
            routing_header: source_routing_header,
            session_id: 0,
        };

        let log_msg = format!(
            "[CLIENT {}] Message fragment sent. Source routing header hops: {:?}\n",
            self.id, packet.routing_header.hops
        );
        self.forward_packet(packet);
        eprintln!("{}", log_msg);
        log_file
            .write_all(log_msg.as_bytes())
            .expect("Failed to write to log file");

        select_biased!(
            recv(self.packet_recv) -> packet_res => {
                if let Ok(packet) = packet_res {
                    match packet.pack_type {
                        PacketType::Ack(ref _ack) => {
                            let log_msg = format!(
                                "[CLIENT {}] Ack received successfully. Packet path: {:?}\n",
                                self.id,
                                packet.routing_header.hops
                            );
                            eprintln!("{}", log_msg.trim());
                            log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                        },
                        _ => {
                            let log_msg = format!("[CLIENT {}] Wrong packet received.\n", self.id);
                            eprintln!("{}", log_msg.trim());
                            log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                        },
                    }
                }
            }
        );
    }

    pub fn run_test_wrong_source_routing_header(&self) {
        let log_path = "tests/wrong_source_routing_header/log.txt";
        let mut log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)
            .expect("Failed to open or create log file");

        let source_routing_header = wg_2024::network::SourceRoutingHeader {
            hop_index: 1,
            hops: vec![10, 20, 30, 40],
        };

        let packet = wg_2024::packet::Packet {
            pack_type: PacketType::MsgFragment(wg_2024::packet::Fragment {
                fragment_index: 0,
                total_n_fragments: 0,
                length: 0,
                data: [0; wg_2024::packet::FRAGMENT_DSIZE],
            }),
            routing_header: source_routing_header,
            session_id: 0,
        };

        let log_msg = format!(
            "[CLIENT {}] Message fragment sent. Source routing header hops: {:?}\n",
            self.id, packet.routing_header.hops
        );
        self.forward_packet(packet);
        eprintln!("{}", log_msg);
        log_file
            .write_all(log_msg.as_bytes())
            .expect("Failed to write to log file");

        select_biased!(
            recv(self.packet_recv) -> packet_res => {
                if let Ok(packet) = packet_res {
                    match packet.pack_type {
                        PacketType::Nack(ref nack) => {
                            if nack.nack_type == wg_2024::packet::NackType::ErrorInRouting(40) {
                                let log_msg = format!(
                                    "[CLIENT {}] Nack->ErrorInRouting(40) received. Source routing header hops: {:?}\n",
                                    self.id,
                                    packet.routing_header.hops
                                );
                                eprintln!("{}", log_msg.trim());
                                log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                            } else {
                                eprintln!("{:?}", nack);
                                let log_msg = format!(
                                    "[CLIENT {}] Nack received, but of wrong type. Source routing header hops: {:?}\n",
                                    self.id,
                                    packet.routing_header.hops
                                );
                                eprintln!("{}", log_msg.trim());
                                log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                            }
                        },
                        _ => {
                            let log_msg = format!("[CLIENT {}] Wrong packet received.\n", self.id);
                            eprintln!("{}", log_msg.trim());
                            log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                        },
                    }
                }
            }
        );
    }

    pub fn run_test_fragment_forward_send(&self) {
        let log_path = "tests/fragment_forward/log.txt";
        let mut log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)
            .expect("Failed to open or create log file");

        let source_routing_header = wg_2024::network::SourceRoutingHeader {
            hop_index: 1,
            hops: vec![10, 20, 30],
        };

        let packet = wg_2024::packet::Packet {
            pack_type: PacketType::MsgFragment(wg_2024::packet::Fragment {
                fragment_index: 0,
                total_n_fragments: 0,
                length: 0,
                data: [0; wg_2024::packet::FRAGMENT_DSIZE],
            }),
            routing_header: source_routing_header,
            session_id: 0,
        };

        let log_msg = format!(
            "[CLIENT {}] Message fragment sent. Source routing header hops: {:?}\n",
            self.id, packet.routing_header.hops
        );
        self.forward_packet(packet);
        eprintln!("{}", log_msg);
        log_file
            .write_all(log_msg.as_bytes())
            .expect("Failed to write to log file");
    }

    pub fn run_test_fragment_forward_recv(&self) {
        // Define the log file path
        let log_path = "tests/fragment_forward/log.txt";

        // Open the log file in append mode
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Failed to open or create log file");

        // Process the first incoming packet (should be a Nack)
        select_biased!(
            recv(self.packet_recv) -> packet_res => {
                if let Ok(packet) = packet_res {
                    match packet.pack_type {
                        PacketType::MsgFragment(ref _msg_fragment) => {
                            let log_msg = format!("[CLIENT {}] Message fragment received successfully. Packet path: {:?}\n", self.id, packet.routing_header.hops);
                            eprintln!("{}", log_msg.trim());
                            log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                        },
                        _ => {
                            let log_msg = format!("[CLIENT {}] Wrong packet received.\n", self.id);
                            eprintln!("{}", log_msg.trim());
                            log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                        },
                    }
                }
            }
        );
    }

    pub fn run_client_flooding_test(&mut self) {
        // Define the log file path
        let log_path = "tests/fragment_forward/log.txt";

        // Open the log file in append mode
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Failed to open or create log file");

        // Process the first incoming packet (should be a Nack)
        select_biased!(
            recv(self.packet_recv) -> packet_res => {
                if let Ok(packet) = packet_res {
                    match packet.pack_type {
                        PacketType::MsgFragment(ref _msg_fragment) => {
                            let log_msg = format!("[CLIENT {}] Message fragment received successfully. Packet path: {:?}\n", self.id, packet.routing_header.hops);
                            eprintln!("{}", log_msg.trim());
                            log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                        },
                        _ => {
                            let log_msg = format!("[CLIENT {}] Wrong packet received.\n", self.id);
                            eprintln!("{}", log_msg.trim());
                            log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                        },
                    }
                }
            }
        );
    }

    pub fn run_crash_test(&mut self) {
        let log_path = "tests/crash_test/log.txt";
        let mut log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)
            .expect("Failed to open or create log file");

        let source_routing_header = wg_2024::network::SourceRoutingHeader {
            hop_index: 1,
            hops: vec![20, 30, 40, 50],
        };

        let packet = wg_2024::packet::Packet {
            pack_type: PacketType::MsgFragment(wg_2024::packet::Fragment {
                fragment_index: 0,
                total_n_fragments: 0,
                length: 0,
                data: [0; wg_2024::packet::FRAGMENT_DSIZE],
            }),
            routing_header: source_routing_header,
            session_id: 0,
        };

        let log_msg = format!(
            "[CLIENT {}] Message fragment sent. Source routing header hops: {:?}\n",
            self.id, packet.routing_header.hops
        );
        self.forward_packet(packet);
        eprintln!("{}", log_msg);
        log_file
            .write_all(log_msg.as_bytes())
            .expect("Failed to write to log file");
    }
}

/// Helper function for consistent status logging
fn log_status(node_id: NodeId, message: &str) {
    println!("[CLIENT {}] {}", node_id, message);
}
