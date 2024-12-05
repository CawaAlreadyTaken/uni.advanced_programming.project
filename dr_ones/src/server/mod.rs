use crossbeam_channel::{Receiver, Sender};
use rand::prelude::ThreadRng;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{FloodResponse, NodeType};
use wg_2024::{
    config::Config,
    controller::DroneEvent,
    network::NodeId,
    packet::{Packet, PacketType},
};

pub struct ServerNode {
    id: NodeId,
    sim_contr_send: Sender<DroneEvent>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    seen_flood_ids: HashSet<u64>,
    topology: Option<Config>,
    random_generator: ThreadRng,
}

pub struct ServerOptions {
    pub id: NodeId,
    pub controller_send: Sender<DroneEvent>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

impl ServerNode {
    pub fn new(options: ServerOptions) -> Self {
        Self {
            id: options.id,
            sim_contr_send: options.controller_send,
            packet_recv: options.packet_recv,
            packet_send: options.packet_send,
            seen_flood_ids: HashSet::new(),
            topology: None,
            random_generator: thread_rng(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let received_packet = self.packet_recv.recv().unwrap();
            match received_packet.pack_type {
                PacketType::Nack(ref _nack) => eprintln!("[SERVER {}] Nack received.", self.id),
                PacketType::Ack(ref _ack) => eprintln!("[SERVER {}] Ack received.", self.id),
                PacketType::MsgFragment(ref _fragment) => {
                    eprintln!("[SERVER {}] MsgFragment received.", self.id)
                }
                PacketType::FloodRequest(ref _floodReq) => {
                    self.handle_flood_request(received_packet)
                }
                PacketType::FloodResponse(ref _floodRes) => {
                    eprintln!("[SERVER {}] FloodResponse received.", self.id)
                }
            }
        }
    }

    // TODO: DON'T KEEP DUPLICATED CODE
    fn build_flood_reponse(
        &mut self,
        packet: Packet,
        updated_path_trace: Vec<(NodeId, NodeType)>,
    ) -> Packet {
        // 1. Check that 'packet' is a flood request
        if let PacketType::FloodRequest(flood_request) = packet.pack_type.clone() {
            // 2. create the pack_type field of the packet to send back
            let flood_response: FloodResponse = FloodResponse {
                flood_id: flood_request.flood_id.clone(),
                path_trace: updated_path_trace.clone(),
            };

            // 3. create the route back to send the flood response to the initiator

            // Manually build the route back without using the method reverse_packet_routing_direction because the
            // hop_index does not matter. The route back is determined thanks to the path_trace attribute of the flood request

            let mut route_back: Vec<u8> = flood_response
                .path_trace
                .iter()
                .map(|tuple| tuple.0)
                .collect();
            // route_back.push(self.id.clone());
            route_back.reverse();

            let new_routing_header = SourceRoutingHeader {
                hop_index: 1,
                hops: route_back,
            };

            // 4. create the packet to send back
            let flood_response_packet = Packet {
                pack_type: PacketType::FloodResponse(flood_response),
                routing_header: new_routing_header,
                session_id: self.random_generator.gen(),
            };

            // 5. Return the packet
            flood_response_packet
        } else {
            eprintln!("Error ! Attempt of building a flood response over a packet that is not a flood request.");
            panic!();
        }
    }

    // TODO: DON'T KEEP DUPLICATED CODE
    // forward the packet to the neighbour node as specified in the routing header.
    fn forward_packet(&self, packet: Packet) {
        let next_hop_id = packet.routing_header.hops[packet.routing_header.hop_index];
        let sess_id = packet.session_id; //TODO: remove. This only needs to log what is happening

        // forward the packet to the next actor
        if let Some(sender) = self.packet_send.get(&next_hop_id) {
            //we are giving away the ownership of the packet
            sender.send(packet).expect("Failed to forward the packet");
        } else {
            println!("No channel found for next hop: {:?}", next_hop_id);
        }

        // eprintln!("{} -> {} : packet_session_id {}", self.id, next_hop_id, sess_id);
    }

    fn handle_flood_request(&mut self, packet: Packet) {
        if let PacketType::FloodRequest(mut flood_request) = packet.pack_type.clone() {
            flood_request.path_trace.push((self.id, NodeType::Server));
            eprintln!(
                "[SERVER {}] FloodRequest {} received with pathTrace: {:?}",
                self.id, flood_request.flood_id, flood_request.path_trace
            );
            //just generate a flood response and send it back
            let flood_response_packet: Packet =
                self.build_flood_reponse(packet, flood_request.path_trace);
            eprintln!(
                "[SERVER {}] Sending FloodResponse sess_id:{} whose path is: {:?}",
                self.id,
                flood_response_packet.session_id,
                flood_response_packet.routing_header.hops
            );
            self.forward_packet(flood_response_packet);
        }
    }
}
