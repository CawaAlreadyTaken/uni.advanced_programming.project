use std::collections::{HashMap, HashSet};
use crossbeam_channel::{select_biased, Receiver, Sender};
use wg_2024::{config::Config, controller::NodeEvent, network::NodeId, packet::{Packet, PacketType}};

pub struct Client {
    id: NodeId,
    sim_contr_send: Sender<NodeEvent>,
    sim_contr_recv: Receiver<ClientCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    seen_flood_ids: HashSet<u64>,
    topology: Option<Config>

}

pub struct ClientOptions {
    pub id: NodeId,
    pub controller_send: Sender<NodeEvent>,
    pub controller_recv: Receiver<ClientCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

pub enum ClientCommand {
    GetFilesList,
    //...
}


impl Client {
    pub fn new(options: ClientOptions) -> Self {
        Self {
            id: options.id,
            sim_contr_send: options.controller_send,
            sim_contr_recv: options.controller_recv,
            packet_recv: options.packet_recv,
            packet_send: options.packet_send,
            seen_flood_ids: HashSet::new(),
            topology: None
        }
    }

    pub fn run(&mut self) {
        // TODO: Flooding

        select_biased!(
            recv(self.sim_contr_recv) -> command_res => {
                if let Ok(command) = command_res {
                    match command {
                        ClientCommand::GetFilesList => println!("[CLIENT {}] GetFilesList command received.", self.id),
                    }
                }
            },
            recv(self.packet_recv) -> packet_res => {
                if let Ok(packet) = packet_res {
                    // each match branch may call a function to handle it to make it more readable
                    match packet.pack_type {
                        PacketType::Nack(ref _nack) => println!("[CLIENT {}] Nack received.", self.id),
                        PacketType::Ack(ref _ack) => println!("[CLIENT {}] Ack received.", self.id),
                        PacketType::MsgFragment(ref _fragment) => println!("[CLIENT {}] MsgFragment received.", self.id),
                        PacketType::FloodRequest(ref _floodReq) => println!("[CLIENT {}] FloodRequest received.", self.id),
                        PacketType::FloodResponse(ref _floodRes) => println!("[CLIENT {}] FloodResponse received.", self.id),
                    }
                }
            }
        );
    }

    fn send_flood_request() {
        //create the packet
        

        //send it
    }

    fn update_topology () {

    }



}
