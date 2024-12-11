//! Server node implementation module.
//! Handles server-side network operations and packet processing.

use crate::utils::NetworkUtils;
use crossbeam_channel::{select_biased, Receiver, Sender};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::{HashMap, HashSet};
use wg_2024::{
    config::Config,
    controller::DroneEvent,
    network::{NodeId, SourceRoutingHeader},
    packet::{Ack, NodeType, Packet, PacketType},
};

/// Server node implementation
pub struct ServerNode {
    id: NodeId,
    sim_contr_send: Sender<DroneEvent>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    seen_flood_ids: HashSet<u64>,
    topology: Option<Config>,
    random_generator: StdRng,
}

impl NetworkUtils for ServerNode {
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

impl ServerNode {
    /// Creates a new server node with the given parameters
    pub fn new(
        id: NodeId,
        controller_send: Sender<DroneEvent>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        Self {
            id,
            sim_contr_send: controller_send,
            packet_recv,
            packet_send,
            seen_flood_ids: HashSet::new(),
            topology: None,
            random_generator: StdRng::from_entropy(),
        }
    }

    /// Main event loop for the server node
    pub fn run(&mut self) {
        loop {
            if let Ok(packet) = self.packet_recv.recv() {
                self.handle_packet(packet);
            }
        }
    }

    /// Handles incoming packets based on their type
    fn handle_packet(&mut self, packet: Packet) {
        match &packet.pack_type {
            PacketType::Nack(ref _nack) => {
                log_status(self.id, "Nack received.");
            }
            PacketType::Ack(ref _ack) => {
                log_status(self.id, "Ack received.");
            }
            PacketType::MsgFragment(ref _fragment) => {
                self.handle_fragment(packet);
            }
            PacketType::FloodRequest(ref _flood_req) => {
                self.handle_flood_request(packet);
            }
            PacketType::FloodResponse(ref _flood_res) => {
                log_status(self.id, "FloodResponse received.");
            }
        }
    }

    // TODO: Code duplication with client node
    /// Handles incoming flood requests
    fn handle_flood_request(&mut self, packet: Packet) {
        if let PacketType::FloodRequest(mut flood_request) = packet.clone().pack_type {
            flood_request.path_trace.push((self.id, NodeType::Server));

            // Create and send flood response
            let flood_response_packet = self.build_flood_response(packet, flood_request.path_trace);
            self.forward_packet(flood_response_packet);
        }
    }

    // TODO: Code duplication with client node
    /// Handles incoming message fragments
    fn handle_fragment(&mut self, packet: Packet) {
        log_status(self.id, "MsgFragment received. Sending an ack...");
        let ack = self.build_ack(packet);
        self.forward_packet(ack);
    }

    // TODOL Code duplication with client node
    /// Builds an acknowledgment packet
    fn build_ack(&self, packet: Packet) -> Packet {
        let frag_index = if let PacketType::MsgFragment(fragment) = &packet.pack_type {
            fragment.fragment_index
        } else {
            // TODO: Handle this error case more gracefully
            panic!("Error: attempt of building an ack on a non-fragment packet.");
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

    // TODO: Code duplication with client and drone node
    /// Reverses the routing direction of a packet
    fn reverse_packet_routing_direction(&self, packet: &mut Packet) {
        let mut hops = packet.routing_header.hops[..packet.routing_header.hop_index + 1].to_vec();
        hops.reverse();

        packet.routing_header = SourceRoutingHeader { hop_index: 1, hops };
    }
}

/// Test-specific server implementations
impl ServerNode {
    /// Runs a server node in test mode for client flooding tests
    pub fn run_client_flooding_test(&mut self) {
        use std::fs::OpenOptions;
        use std::io::Write;

        let log_path = "tests/client_flooding/log.txt";
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Failed to open log file");

        loop {
            select_biased!(
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        match packet.pack_type {
                            PacketType::FloodRequest(ref _flood_req) => {
                                log_status(self.id, "Flood request received");
                                self.handle_flood_request(packet);
                            }
                            _ => {
                                log_status(self.id, "Wrong packet received.");
                                log_file.write_all(format!("[SERVER {}] Wrong packet received.\n", self.id).as_bytes())
                                    .expect("Failed to write to log file");
                            },
                        }
                    }
                }
            );
        }
    }

    /// Runs a server node in test mode for testing acknowledgment behavior
    pub fn run_test_ack_sent_back(&mut self) {
        use std::fs::OpenOptions;
        use std::io::Write;

        let log_path = "tests/ack_sent_back/log.txt";
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Failed to open log file");

        select_biased!(
            recv(self.packet_recv) -> packet_res => {
                if let Ok(packet) = packet_res {
                    match packet.pack_type {
                        PacketType::MsgFragment(ref _msg_fragment) => {
                            let ack = self.build_ack(packet);
                            log_status(self.id, &format!(
                                "Message fragment received. Sending ack back following this path: {:?}",
                                ack.routing_header.hops
                            ));
                            self.forward_packet(ack);
                        }
                        _ => {
                            log_status(self.id, "Wrong packet received.");
                            log_file.write_all(format!("[SERVER {}] Wrong packet received.\n", self.id).as_bytes())
                                .expect("Failed to write to log file");
                        },
                    }
                }
            }
        );
    }
}

/// Helper function for consistent status logging
fn log_status(node_id: NodeId, message: &str) {
    println!("[SERVER {}] {}", node_id, message);
}
