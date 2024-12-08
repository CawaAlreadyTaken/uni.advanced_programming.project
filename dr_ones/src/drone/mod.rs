//! Drone implementation module.
//! Handles packet routing, flooding, and network management for drone nodes.

use crate::utils::NetworkUtils;
use crossbeam_channel::{select, Receiver, Sender};
use indexmap::IndexSet;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashMap;
use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::{NodeId, SourceRoutingHeader},
    packet::{Nack, NackType, NodeType, Packet, PacketType},
};

/// Implementation of a drone node in the network.
/// Responsible for routing packets and managing network connections.
#[allow(non_camel_case_types)] // Keeping name for compatibility
pub struct Dr_One {
    id: NodeId,
    sim_contr_send: Sender<DroneEvent>,
    sim_contr_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: f32,
    seen_flood_ids: IndexSet<u64>,
    random_generator: StdRng,
    should_exit: bool,
}

impl NetworkUtils for Dr_One {
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

impl Drone for Dr_One {
    fn new(
        id: NodeId,
        controller_send: Sender<DroneEvent>,
        controller_recv: Receiver<DroneCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        pdr: f32,
    ) -> Self {
        Self {
            id,
            sim_contr_send: controller_send,
            sim_contr_recv: controller_recv,
            packet_recv,
            packet_send,
            pdr,
            seen_flood_ids: IndexSet::new(),
            random_generator: StdRng::from_entropy(),
            should_exit: false,
        }
    }

    fn run(&mut self) {
        self.run_internal();
    }
}

impl Dr_One {
    /// Main event loop for the drone.
    fn run_internal(&mut self) {
        while !self.should_exit {
            select! {
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        self.handle_packet(packet);
                    }
                },

                recv(self.sim_contr_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        self.handle_command(command);
                    }
                }
            }
        }
    }

    /// Handles incoming packets based on their type.
    fn handle_packet(&mut self, packet: Packet) {
        match packet.pack_type {
            PacketType::FloodRequest(_) => self.handle_flood_request(packet),
            _ => self.handle_routed_packet(packet),
        }
    }

    /// Handles simulation controller commands.
    fn handle_command(&mut self, command: DroneCommand) {
        match command {
            DroneCommand::AddSender(node_id, sender) => self.add_channel(node_id, sender),
            DroneCommand::SetPacketDropRate(new_pdr) => self.set_pdr(new_pdr),
            DroneCommand::Crash => self.crash(),
            DroneCommand::RemoveSender(node_id) => self.remove_channel(node_id),
        }
    }

    /// Handles a flood request packet.
    fn handle_flood_request(&mut self, packet: Packet) {
        if let PacketType::FloodRequest(mut flood_request) = packet.pack_type.clone() {
            let sender_id = flood_request
                .path_trace
                .last()
                .map(|node| node.0)
                .unwrap_or_default();

            flood_request.path_trace.push((self.id, NodeType::Drone));

            if self.should_respond_to_flood(&flood_request) {
                let response = self.build_flood_response(packet, flood_request.path_trace);
                self.forward_packet(response);
            } else {
                let updated_packet = Packet {
                    pack_type: PacketType::FloodRequest(flood_request),
                    routing_header: packet.routing_header,
                    session_id: packet.session_id,
                };
                self.broadcast_packet(updated_packet, sender_id);
            }
        }
    }

    /// Determines if we should respond to a flood request instead of broadcasting it.
    fn should_respond_to_flood(&self, flood_request: &wg_2024::packet::FloodRequest) -> bool {
        self.seen_flood_ids.contains(&flood_request.flood_id) || self.packet_send.len() == 1
        // Only one neighbor means we can't forward
    }

    /// Handles packets that require routing.
    fn handle_routed_packet(&mut self, mut packet: Packet) {
        // Verify routing
        if !self.verify_routing(&packet) {
            return;
        }

        packet.routing_header.hop_index += 1;

        // Handle final destination
        if packet.routing_header.hop_index == packet.routing_header.hops.len() {
            let nack = self.build_nack(packet, NackType::DestinationIsDrone);
            self.forward_packet(nack);
            return;
        }

        // Process based on packet type
        match packet.pack_type {
            PacketType::MsgFragment(_) => self.handle_message_fragment(packet),
            _ => self.forward_packet(packet),
        }
    }

    /// Verifies if this drone is the correct recipient for the packet.
    fn verify_routing(&mut self, packet: &Packet) -> bool {
        let index = packet.routing_header.hop_index;
        if self.id != packet.routing_header.hops[index] {
            let mut packet = packet.clone();
            packet.routing_header.hop_index += 1;
            let nack = self.build_nack(packet, NackType::UnexpectedRecipient(self.id));
            self.forward_packet(nack);
            return false;
        }
        true
    }

    /// Handles message fragment packets, including PDR-based dropping.
    fn handle_message_fragment(&mut self, packet: Packet) {
        let next_hop_id = packet.routing_header.hops[packet.routing_header.hop_index];

        if !self.packet_send.contains_key(&next_hop_id) {
            let nack = self.build_nack(packet, NackType::ErrorInRouting(next_hop_id));
            self.forward_packet(nack);
            return;
        }

        if self.should_drop_packet() {
            let nack = self.build_nack(packet, NackType::Dropped);
            self.forward_packet(nack);
            return;
        }

        self.forward_packet(packet);
    }

    /// Determines if a packet should be dropped based on PDR.
    fn should_drop_packet(&self) -> bool {
        let pdr_scaled = (self.pdr * 100.0) as i32;
        rand::thread_rng().gen_range(0..=100) < pdr_scaled
    }

    // Network management methods

    fn add_channel(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.packet_send.insert(id, sender);
    }

    fn remove_channel(&mut self, id: NodeId) {
        if !self.packet_send.contains_key(&id) {
            eprintln!(
                "Error! The current node {} has no neighbour node {}.",
                self.id, id
            );
            return;
        }
        self.packet_send.remove(&id);
    }

    fn set_pdr(&mut self, new_pdr: f32) {
        self.pdr = new_pdr;
    }

    fn crash(&mut self) {
        println!("[DRONE {}] Starting crash sequence", self.id);

        // Process remaining packets
        println!("[DRONE {}] Processing remaining packets...", self.id);
        while let Ok(packet) = self.packet_recv.try_recv() {
            self.handle_packet(packet);
        }

        self.should_exit = true;
        println!("[DRONE {}] Crashed", self.id);
    }

    /// Broadcasts a packet to all neighbors except the sender.
    fn broadcast_packet(&self, packet: Packet, exclude_id: NodeId) {
        let eligible_neighbors: HashMap<_, _> = self
            .packet_send
            .iter()
            .filter(|(&id, _)| id != exclude_id)
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        for (node_id, sender) in eligible_neighbors {
            if let Err(e) = sender.send(packet.clone()) {
                println!("Failed to send packet to NodeId {:?}: {:?}", node_id, e);
            }
        }
    }

    /// Builds a NACK packet in response to a received packet.
    fn build_nack(&self, packet: Packet, nack_type: NackType) -> Packet {
        let fragment_index = match &packet.pack_type {
            PacketType::MsgFragment(fragment) => fragment.fragment_index,
            _ => 0,
        };

        let nack = Nack {
            fragment_index,
            nack_type,
        };

        let mut response = Packet {
            pack_type: PacketType::Nack(nack),
            routing_header: packet.routing_header,
            session_id: packet.session_id,
        };

        self.reverse_packet_routing_direction(&mut response);
        response
    }

    /// Reverses the routing direction of a packet for sending responses.
    fn reverse_packet_routing_direction(&self, packet: &mut Packet) {
        let mut hops = packet.routing_header.hops[..packet.routing_header.hop_index].to_vec();
        hops.reverse();

        packet.routing_header = SourceRoutingHeader { hop_index: 1, hops };
    }

    #[cfg(test)]
    fn crash_with_logging(&mut self) -> std::io::Result<()> {
        let log_file = LogFile::new("tests/crash_test/log.txt")?;

        // Log crash sequence start
        log_file.log_message(&format!("[DRONE {}] Starting crash sequence", self.id))?;

        // Log packet processing
        log_file.log_message(&format!(
            "[DRONE {}] Processing remaining packets...",
            self.id
        ))?;

        // Process remaining packets
        while let Ok(packet) = self.packet_recv.recv() {
            self.handle_packet(packet);
        }

        // Set exit flag and log final status
        self.should_exit = true;
        log_file.log_message(&format!("[DRONE {}] CRASHED.", self.id))?;

        Ok(())
    }

    // Test methods are kept separate at the end
    #[cfg(test)]
    pub fn run_crash_test(&mut self) {
        while !self.should_exit {
            select! {
                // handle receiving a packet from another drone
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        match packet.pack_type{
                            // flood request are particular because the recipient is not specified
                            PacketType::FloodRequest(ref _flood_req) => self.handle_flood_request(packet),
                            _ => self.handle_routed_packet(packet),
                        }
                    }
                },

                // handle receiving a message from the simulation controller
                recv(self.sim_contr_recv) -> command_res => {
                    if let Ok(command) = command_res {

                        // each match branch may call a routine to handle it to make it more readable
                        match command {
                            DroneCommand::AddSender(node_id,sender) => self.add_channel(node_id,sender),
                            DroneCommand::SetPacketDropRate(new_pdr) => self.set_pdr(new_pdr),
                            DroneCommand::Crash => {
                                if let Err(e) = self.crash_with_logging() {
                                    eprintln!("Error during crash logging: {}", e);
                                }
                            },
                            DroneCommand::RemoveSender(node_id) => self.remove_channel(node_id),
                        }
                    }
                }
            }
        }
    }
}

/// Helper struct for test logging
#[cfg(test)]
struct LogFile {
    file: std::fs::File,
}

#[cfg(test)]
impl LogFile {
    fn new(path: &str) -> std::io::Result<Self> {
        use std::fs::OpenOptions;

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        Ok(LogFile { file })
    }

    fn log_message(&self, message: &str) -> std::io::Result<()> {
        use std::io::Write;

        eprintln!("{}", message);
        self.file.try_clone()?.write_all(message.as_bytes())?;
        self.file.try_clone()?.write_all(b"\n")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Test implementations...
}
