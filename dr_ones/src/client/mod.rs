use std::collections::{HashMap, HashSet};
use crossbeam_channel::{select_biased, Receiver, Sender};
use wg_2024::{config::Config, controller::NodeEvent, network::{NodeId, SourceRoutingHeader}, packet::{FloodRequest, NodeType, Packet, PacketType}};

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
        //  Flooding
        self.send_flood_request();

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
                        PacketType::Nack(ref _nack) => eprintln!("[CLIENT {}] Nack received.", self.id),
                        PacketType::Ack(ref _ack) => eprintln!("[CLIENT {}] Ack received.", self.id),
                        PacketType::MsgFragment(ref _fragment) => eprintln!("[CLIENT {}] MsgFragment received.", self.id),
                        PacketType::FloodRequest(ref _floodReq) => eprintln!("[CLIENT {}] FloodRequest received from (maybe) DRONE {}.", self.id, _floodReq.path_trace.last().unwrap().0),
                        PacketType::FloodResponse(ref _floodRes) => eprintln!("[CLIENT {}] FloodResponse received.", self.id),
                        // PacketType::FloodResponse(ref _floodRes) => self.update_topology(packet),
                    }
                }
            }
        );
    }

    fn send_flood_request(&self) {
        //create the packets
        let flood_request = FloodRequest {
            flood_id: 21324, //TODO: random (maybe check it to be different to the previously generated ones)
            initiator_id: self.id,
            path_trace: vec![(self.id, NodeType::Client)],
        };

        let source_routing_header = SourceRoutingHeader {
            hop_index: 0,
            hops: vec![self.id]
        };

        let packet = Packet {
            pack_type: PacketType::FloodRequest(flood_request),
            routing_header: source_routing_header,
            session_id: 12345, //TODO: make it random enough not to conflict with other packets (dunno if this makes sense)
        };

        //send it to all adjacent nodes (that will be drones)
        for (&node_id, sender) in self.packet_send.iter() {
            // Send a clone packet
            if let Err(e) = sender.send(packet.clone()) {
                println!("Failed to send floodRequest to NodeId {:?}: {:?}", node_id, e);
            }
        }
        eprintln!("Client id: {} -> flood_request broadcasted to peers: {:?}", self.id, self.packet_send.keys());
    }

    //function that handles the flood_Response
    fn update_topology (&self, packet: Packet) {
        //TODO
    }

}
