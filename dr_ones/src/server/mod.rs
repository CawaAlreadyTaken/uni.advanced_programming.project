use crate::utils::NetworkUtils;
use crossbeam_channel::{Receiver, Sender};
use rand::prelude::ThreadRng;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use wg_2024::{
    config::Config,
    controller::DroneEvent,
    network::NodeId,
    packet::{NodeType, Packet, PacketType},
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

impl NetworkUtils for ServerNode {
    fn get_id(&self) -> NodeId {
        self.id
    }

    fn get_packet_senders(&self) -> &HashMap<NodeId, Sender<Packet>> {
        &self.packet_send
    }

    fn get_random_generator(&mut self) -> &mut ThreadRng {
        &mut self.random_generator
    }
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

    fn handle_flood_request(&mut self, packet: Packet) {
        if let PacketType::FloodRequest(mut flood_request) = packet.pack_type.clone() {
            flood_request.path_trace.push((self.id, NodeType::Server));
            eprintln!(
                // "[SERVER {}] FloodRequest {} received with pathTrace: {:?}",
                // self.id, flood_request.flood_id, flood_request.path_trace
            );
            //just generate a flood response and send it back
            let flood_response_packet = self.build_flood_response(packet, flood_request.path_trace);
            eprintln!(
                // "[SERVER {}] Sending FloodResponse sess_id:{} whose path is: {:?}",
                // self.id,
                // flood_response_packet.session_id,
                // flood_response_packet.routing_header.hops
            );
            self.forward_packet(flood_response_packet);
        }
    }
}
