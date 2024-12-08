use crate::utils::NetworkUtils;
use crossbeam_channel::{select_biased, Receiver, Sender};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use wg_2024::{
    config::Config,
    controller::DroneEvent,
    network::{NodeId,SourceRoutingHeader},
    packet::{Ack,NodeType, Packet, PacketType},
};

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
            random_generator: StdRng::from_entropy(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let received_packet = self.packet_recv.recv().unwrap();
            match received_packet.pack_type {
                PacketType::Nack(ref _nack) => eprintln!("[SERVER {}] Nack received.", self.id),
                PacketType::Ack(ref _ack) => eprintln!("[SERVER {}] Ack received.", self.id),
                PacketType::MsgFragment(ref _fragment) => self.handle_fragment(received_packet),
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

    ////! CODE DUPLICATE ALSO PRESENT FOR THE CLIENT
    // Build an ack based on 'packet'
    fn build_ack(&self, packet: Packet) -> Packet {

        // 1. Keep in the ack the fragment index if the packet contains a fragment
        let frag_index: u64;
        
        if let PacketType::MsgFragment(fragment) = &packet.pack_type {
            frag_index = fragment.fragment_index;
        } else {
            eprintln!("Error : attempt of building an ack on a non-fragment packet.");
            panic!();
        }
        
        // 2. Build the Aack instance of the packet to return
        let ack: Ack = Ack {
            fragment_index: frag_index,
        };
        
        // 3. Build the packet
        let packet_type = PacketType::Ack(ack);
        
        let mut packet: Packet = Packet {
            pack_type: packet_type,
            routing_header: packet.routing_header,
            session_id: packet.session_id,
        };
        
        // 4. Reverse the routing direction of the packet because nacks need to be sent back
        
        self.reverse_packet_routing_direction(&mut packet);
        
        // 5. Return the packet
        packet
    }

    ////! CODE DUPLICATE ALSO PRESENT FOR THE CLIENT
    // reverse the packet route in order it to be sent back.
    // In the end, the packet should go to the node (server or client) that initially routed the packet.
    fn reverse_packet_routing_direction(&self, packet: &mut Packet) {
        // a. create the route back using the path trace of the packet
        let mut hops_vec: Vec<NodeId> = packet.routing_header.hops.clone();

        if packet.routing_header.hop_index != hops_vec.len()-1 {
            // remove the nodes that are not supposed to receive the packet anymore (between self and the original final destination of the packet)
            hops_vec.drain(packet.routing_header.hop_index..=hops_vec.len() - 1);
        }

        // reverse the order of the nodes to reach in comparison with the original routing header
        hops_vec.reverse();

        let route_back: SourceRoutingHeader = SourceRoutingHeader {
            //THE SOURCEROUTINGHEADER SAYS THAT THE HOP INDEX SHOULD BE INITIALIZED AS 1, BUT
            //KEEP IN MIND THAT IN THIS WAY THE NODE THAT RECEIVES THIS PACKET WILL SEE ITSELF IN THE PATH_TRACE[HOP_INDEX]
            hop_index: 1, // Start from the first hop
            hops: hops_vec,
        };

        // b. update the packet's routing header
        packet.routing_header = route_back;
    }
    
    ////! CODE DUPLICATE ALSO PRESENT FOR THE CLIENT
    // handle received fragment
    fn handle_fragment(&mut self, mut packet: Packet){
        eprintln!("[SERVER {}] MsgFragment received. Sending an ack...", self.id);

        // 1. Create an ack and forward it
        let ack = self.build_ack(packet);
        let log_msg = format!("[SERVER {}] Message fragment received. Sending ack back in following this path: {:?}\n", self.id, ack.routing_header.hops);
        eprintln!("{}", log_msg.trim());

        self.forward_packet(ack);

        //TODO: this function should be replaced by handle_routed_packet like in the drone implementation and handle acks, nacks and flood responses
        // handle_routed_packet could be a method of the NetworkUtils trait and be implemented according depending on the node type
        // Also, the assembler may be called in this function to check if all the fragments of a message have been received
        
    }

    // ------------------------------------------------------------------------------------------------
    // -------------------------------------- TEST FUNCTIONS ------------------------------------------
    // ------------------------------------------------------------------------------------------------

    pub fn run_test_ack_sent_back(&mut self) {
        // Define the log file path
        let log_path = "tests/ack_sent_back/log.txt";

        // Open the log file in write mode
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .expect("Failed to open or create log file");

        let generic_ack = Ack {
            fragment_index: 0,
        };

        // Process the first incoming packet (should be a Nack)
        select_biased!(
            recv(self.packet_recv) -> packet_res => {
                if let Ok(packet) = packet_res {
                    match packet.pack_type {
                        PacketType::MsgFragment(ref msg_fragment) => {
                            let ack = self.build_ack(packet);

                            let log_msg = format!("[SERVER {}] Message fragment received. Sending ack back in following this path: {:?}\n", self.id, ack.routing_header.hops);
                            // let log_msg = format!("[SERVER {}] Message fragment received. Sending ack back" , self.id);
                            eprintln!("{}", log_msg.trim());
                            // log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file"); //TODO: fix, don't know why this message doesn't get printed

                            self.forward_packet(ack);
                        }
                        _ => {
                            let log_msg = format!("[SERVER {}] Wrong packet received.\n", self.id);
                            eprintln!("{}", log_msg.trim());
                            log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                        },
                    }
                }
            }
        );
    }

    //--------------------------------
}
