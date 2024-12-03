use std::collections::{HashMap, HashSet};
use crossbeam_channel::{Receiver, Sender};
use wg_2024::{config::Config, controller::DroneEvent, network::NodeId, packet::{Packet, PacketType}};

pub struct Server {
    id: NodeId,
    sim_contr_send: Sender<DroneEvent>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    seen_flood_ids: HashSet<u64>,
    topology: Option<Config>
}

pub struct ServerOptions {
    pub id: NodeId,
    pub controller_send: Sender<DroneEvent>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

impl Server {
    pub fn new(options: ServerOptions) -> Self {
        Self {
            id: options.id,
            sim_contr_send: options.controller_send,
            packet_recv: options.packet_recv,
            packet_send: options.packet_send,
            seen_flood_ids: HashSet::new(),
            topology: None
        }
    }

    pub fn run(&mut self) {
        loop {
            let received_packet = self.packet_recv.recv().unwrap();
            match received_packet.pack_type {
                PacketType::Nack(ref _nack) => eprintln!("[SERVER {}] Nack received.", self.id),
                PacketType::Ack(ref _ack) => eprintln!("[SERVER {}] Ack received.", self.id),
                PacketType::MsgFragment(ref _fragment) => eprintln!("[SERVER {}] MsgFragment received.", self.id),
                PacketType::FloodRequest(ref _floodReq) => eprintln!("[SERVER {}] FloodRequest received with pathTrace: {:?}", self.id, _floodReq.path_trace),
                PacketType::FloodResponse(ref _floodRes) => eprintln!("[SERVER {}] FloodResponse received.", self.id),
            }
        }
    }
}
