use crate::utils::NetworkUtils;
use crossbeam_channel::{select_biased, Receiver, Sender};
use indexmap::IndexSet;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use wg_2024::{
    config::{Client, Config, Drone, Server},
    controller::DroneEvent,
    network::{NodeId, SourceRoutingHeader},
    packet::{Ack,FloodRequest, NodeType, Packet, PacketType},
};
use wg_2024::packet::{Fragment, FRAGMENT_DSIZE};
use wg_2024::packet::NackType::ErrorInRouting;

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

pub struct ClientOptions {
    pub id: NodeId,
    pub controller_send: Sender<DroneEvent>,
    pub controller_recv: Receiver<ClientCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
}

pub enum ClientCommand {
    GetFilesList,
    //...
}

impl ClientNode {
    pub fn new(options: ClientOptions) -> Self {
        Self {
            id: options.id,
            sim_contr_send: options.controller_send,
            sim_contr_recv: options.controller_recv,
            packet_recv: options.packet_recv,
            packet_send: options.packet_send,
            seen_flood_ids: IndexSet::new(),
            topology: None,
            random_generator: StdRng::from_entropy(),
        }
    }
    
    pub fn run(&mut self) {
        //  Flooding
        self.initialize_topology(); //TODO: is this really the best approach? can't we initialize the topology like this in the constructor??
        self.send_flood_request();
        
        loop {
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
                            PacketType::MsgFragment(ref _fragment) => self.handle_fragment(packet),
                            PacketType::FloodRequest(ref _floodReq) => self.handle_flood_request(packet),
                            PacketType::FloodResponse(ref _floodRes) => self.update_topology(packet),
                        }
                    }
                }
            );
        }
    }
    
    fn send_flood_request(&mut self) {
        let random_id: u64 = self.random_generator.gen();
        
        //create the packets
        let flood_request = FloodRequest {
            flood_id: random_id,
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
        
        //send it to all adjacent nodes (that will be drones)
        let mut correct_send: bool = true;
        for (&node_id, sender) in self.packet_send.iter() {
            // Send a clone packet
            if let Err(e) = sender.send(packet.clone()) {
                println!(
                    "Failed to send floodRequest to NodeId {:?}: {:?}",
                    node_id, e
                );
                correct_send = false;
            }
        }
        if correct_send {
            self.seen_flood_ids.insert(random_id);
            // eprintln!("Client id: {} -> flood_request broadcasted to peers: {:?}", self.id, self.packet_send.keys());
        }
    }
    
    fn handle_flood_request(&mut self, packet: Packet) {
        if let PacketType::FloodRequest(mut flood_request) = packet.pack_type.clone() {
            flood_request.path_trace.push((self.id, NodeType::Client));
            eprintln!(
                // "[CLIENT {}] FloodRequest {} received with pathTrace: {:?}",
                // self.id, flood_request.flood_id, flood_request.path_trace
            );
            //just generate a flood response and send it back
            let flood_response_packet = self.build_flood_response(packet, flood_request.path_trace);
            eprintln!(
                // "[CLIENT {}] Sending FloodResponse sess_id:{} whose path is: {:?}",
                // self.id,
                // flood_response_packet.session_id,
                // flood_response_packet.routing_header.hops
            );
            self.forward_packet(flood_response_packet);
        }
    }
    
    ////! CODE DUPLICATE ALSO PRESENT FOR THE SERVER
    // Build an ack based on 'packet'
    fn build_ack(&self, packet: Packet) -> Packet {
        
        // 1. Keep in the ack the fragment index if the packet contains a fragment
        let frag_index: u64;
        
        if let PacketType::MsgFragment(fragment) = &packet.pack_type {
            frag_index = fragment.fragment_index;
        } else {
            eprintln!("Error : attempt of building an ack on a non-fragment packet.");
            panic!()
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
    
    ////! CODE DUPLICATE ALSO PRESENT FOR THE SERVER
    // reverse the packet route in order it to be sent back.
    // In the end, the packet should go to the node (server or client) that initially routed the packet.
    fn reverse_packet_routing_direction(&self, packet: &mut Packet) {
        // a. create the route back using the path trace of the packet
        let mut hops_vec: Vec<NodeId> = packet.routing_header.hops.clone();
        
        // remove the nodes that are not supposed to receive the packet anymore (between self and the original final destination of the packet)
        hops_vec.drain(packet.routing_header.hop_index..=hops_vec.len() - 1);
        
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
    
    ////! CODE DUPLICATE ALSO PRESENT FOR THE SERVER
    // handle received fragment
    fn handle_fragment(&mut self, packet: Packet){
        
        eprintln!("[CLIENT {}] MsgFragment received. Sending an ack...", self.id);
        
        // 1. Create an ack and forward it
        let ack = self.build_ack(packet);
        
        self.forward_packet(ack);
        
        //TODO: this function should be replaced by handle_routed_packet like in the drone implementation and handle acks, nacks and flood responses
        // handle_routed_packet could be a method of the NetworkUtils trait and be implemented according depending on the node type
        // Also, the assembler may be called in this function to check if all the fragments of a message have been received
        
    }
    
    fn initialize_topology(&mut self) {
        let neighbours_ids: Vec<NodeId> = self.packet_send.keys().cloned().collect();
        let mut initial_drone_vec: Vec<Drone> = Vec::new();
        for neighbour_id in &neighbours_ids {
            let temp_drone: Drone = Drone {
                id: *neighbour_id,
                connected_node_ids: vec![self.id],
                pdr: 0.27, //todo: see if we can put here its real pdr
            };
            initial_drone_vec.push(temp_drone);
        }
        
        let this_client: Client = Client {
            id: self.id,
            connected_drone_ids: neighbours_ids,
        };
        
        let new_topology: Config = Config {
            drone: initial_drone_vec,
            client: vec![this_client],
            server: vec![],
        };
        
        self.topology = Some(new_topology);
    }
    
    fn update_topology(&mut self, packet: Packet) {
        if let PacketType::FloodResponse(flood_response) = packet.pack_type {
            eprintln!(
                // "[CLIENT {}] FloodResponse sess_id:{} flood_id:{} received. path_trace: {:?}",
                // self.id, packet.session_id, flood_response.flood_id, flood_response.path_trace
            );
            if !self.seen_flood_ids.contains(&flood_response.flood_id) {
                //Panic because I shouldn't receive flood responses initiated by other nodes!
                eprintln!("I shouldn't receive flood responses initiated by other nodes! Panic!");
                panic!();
            } else if !self.seen_flood_ids.is_empty()
            && flood_response.flood_id == *self.seen_flood_ids.last().unwrap()
            {
                // check if the flood_response's flood_id matches the last one inserted in seen_flood_ids
                // Add everything to the topology -> scan the path trace knowing that adjacent entries are connected between themselves
                
                // Use a mutable borrow of `self.topology`
                if let Some(topology) = &mut self.topology {
                    for (i, current) in flood_response.path_trace.iter().enumerate() {
                        let mut current_index_in_topology: usize;
                        
                        match current.1 {
                            NodeType::Client => {
                                if let Some(index) =
                                topology.client.iter().position(|x| x.id == current.0)
                                {
                                    current_index_in_topology = index;
                                } else {
                                    topology.client.push(Client {
                                        id: current.0,
                                        connected_drone_ids: vec![],
                                    });
                                    current_index_in_topology = topology.client.len() - 1;
                                }
                                
                                if i > 0
                                && !topology.client[current_index_in_topology]
                                .connected_drone_ids
                                .contains(&flood_response.path_trace[i - 1].0)
                                {
                                    topology.client[current_index_in_topology]
                                    .connected_drone_ids
                                    .push(flood_response.path_trace[i - 1].0);
                                }
                                if i < flood_response.path_trace.len() - 1
                                && !topology.client[current_index_in_topology]
                                .connected_drone_ids
                                .contains(&flood_response.path_trace[i + 1].0)
                                {
                                    topology.client[current_index_in_topology]
                                    .connected_drone_ids
                                    .push(flood_response.path_trace[i + 1].0);
                                }
                            }
                            NodeType::Server => {
                                if let Some(index) =
                                topology.server.iter().position(|x| x.id == current.0)
                                {
                                    current_index_in_topology = index;
                                } else {
                                    topology.server.push(Server {
                                        id: current.0,
                                        connected_drone_ids: vec![],
                                    });
                                    current_index_in_topology = topology.server.len() - 1;
                                }
                                
                                if i > 0
                                && !topology.server[current_index_in_topology]
                                .connected_drone_ids
                                .contains(&flood_response.path_trace[i - 1].0)
                                {
                                    topology.server[current_index_in_topology]
                                    .connected_drone_ids
                                    .push(flood_response.path_trace[i - 1].0);
                                }
                                if i < flood_response.path_trace.len() - 1
                                && !topology.server[current_index_in_topology]
                                .connected_drone_ids
                                .contains(&flood_response.path_trace[i + 1].0)
                                {
                                    topology.server[current_index_in_topology]
                                    .connected_drone_ids
                                    .push(flood_response.path_trace[i + 1].0);
                                }
                            }
                            NodeType::Drone => {
                                if let Some(index) =
                                topology.drone.iter().position(|x| x.id == current.0)
                                {
                                    current_index_in_topology = index;
                                } else {
                                    topology.drone.push(Drone {
                                        id: current.0,
                                        connected_node_ids: vec![],
                                        pdr: 0.27,
                                    });
                                    current_index_in_topology = topology.drone.len() - 1;
                                }
                                
                                if i > 0
                                && !topology.drone[current_index_in_topology]
                                .connected_node_ids
                                .contains(&flood_response.path_trace[i - 1].0)
                                {
                                    topology.drone[current_index_in_topology]
                                    .connected_node_ids
                                    .push(flood_response.path_trace[i - 1].0);
                                }
                                if i < flood_response.path_trace.len() - 1
                                && !topology.drone[current_index_in_topology]
                                .connected_node_ids
                                .contains(&flood_response.path_trace[i + 1].0)
                                {
                                    topology.drone[current_index_in_topology]
                                    .connected_node_ids
                                    .push(flood_response.path_trace[i + 1].0);
                                }
                            }
                        }
                    }
                }
                
                if self.id == 1 {
                    self.print_topology(packet.session_id, flood_response.path_trace);
                }
            } else {
                //This is the case in which I receive a flood response that belongs to an old flood initiated by me
                eprintln!(
                    "[CLIENT {}] I'm not supposed to handle this OLD flood response. Skipping!",
                    self.id
                );
            }
        }
    }
    
    fn print_topology(
        &self,
        last_topology_update_message_session_id: u64,
        path_trace: Vec<(NodeId, NodeType)>,
    ) {
        if let Some(topology) = &self.topology {
            eprintln!("--------------------------------------");
            eprintln!(
                "NODE {} TOPOLOGY after message with sess_id:{} and path_trace:{:?}",
                self.id, last_topology_update_message_session_id, path_trace
            );
            eprintln!("---------------");
            eprintln!("CLIENTS");
            for client in &topology.client {
                eprintln!("{} -> {:?}", client.id, client.connected_drone_ids);
            }
            eprintln!("---------------");
            eprintln!("DRONES");
            for drone in &topology.drone {
                eprintln!("{} -> {:?}", drone.id, drone.connected_node_ids);
            }
            eprintln!("---------------");
            eprintln!("SERVERS");
            for server in &topology.server {
                eprintln!("{} -> {:?}", server.id, server.connected_drone_ids);
            }
            eprintln!("--------------------------------------");
        }
    }
    
    // ------------------------------------------------------------------------------------------------
    // -------------------------------------- TEST FUNCTIONS ------------------------------------------
    // ------------------------------------------------------------------------------------------------
    
    pub fn run_test_wrong_source_routing_header(&self) {
        // Define the log file path
        let log_path = "tests/wrong_source_routing_header/log.txt";
        
        // Open the log file in write mode
        let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("Failed to open or create log file");
        
        //Create a generic fragment packet with a wrong hardcoded source_routing_header and send it to the neighbour drone!
        let generic_fragment = Fragment {
            fragment_index: 0,
            total_n_fragments: 0,
            length: 0,
            data: [0; FRAGMENT_DSIZE],
        };
        
        let source_routing_header = SourceRoutingHeader {
            hop_index: 1,
            hops: vec![10, 20, 30, 40], //==> This client will be 10. The fact is that the drone n.40 doesn't exist! Let's see what happens
        };
        
        let packet = Packet {
            pack_type: PacketType::MsgFragment(generic_fragment),
            routing_header: source_routing_header,
            session_id: 0,
        };
        
        let log_msg = format!("[CLIENT {}] Message fragment sent. Source routing header hops: {:?}\n", self.id, packet.routing_header.hops);
        self.forward_packet(packet);
        eprintln!("{}", log_msg);
        log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
        
        // Process the first incoming packet (should be a Nack)
        select_biased!(
            recv(self.packet_recv) -> packet_res => {
                if let Ok(packet) = packet_res {
                    match packet.pack_type {
                        PacketType::Nack(ref nack) => {
                            if nack.nack_type == ErrorInRouting(40) {
                                let log_msg = format!("[CLIENT {}] Nack->ErrorInRouting(40) received. Source routing header hops: {:?}\n", self.id, packet.routing_header.hops);
                                eprintln!("{}", log_msg.trim());
                                log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
                            } else {
                                eprintln!("{:?}", nack);
                                let log_msg = format!("[CLIENT {}] Nack received, but of wrong type. Source routing header hops: {:?}\n", self.id, packet.routing_header.hops);
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
    
    //--------------------------------
    
    pub fn run_test_fragment_forward_send(&self) {
        // Define the log file path
        let log_path = "tests/fragment_forward/log.txt";
        
        // Open the log file in write mode
        let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("Failed to open or create log file");
        
        //Create a generic fragment packet with a hardcoded source_routing_header and send it to the neighbour drone!
        let generic_fragment = Fragment {
            fragment_index: 0,
            total_n_fragments: 0,
            length: 0,
            data: [0; FRAGMENT_DSIZE],
        };
        
        let source_routing_header = SourceRoutingHeader {
            hop_index: 1,
            hops: vec![10, 20, 30],
        };
        
        let packet = Packet {
            pack_type: PacketType::MsgFragment(generic_fragment),
            routing_header: source_routing_header,
            session_id: 0,
        };
        
        let log_msg = format!("[CLIENT {}] Message fragment sent. Source routing header hops: {:?}\n", self.id, packet.routing_header.hops);
        self.forward_packet(packet);
        eprintln!("{}", log_msg);
        log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
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
                        PacketType::MsgFragment(ref msg_fragment) => {
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
    
    //--------------------------------
    
    pub fn run_test_ack_sent_back(&self) {
        // Define the log file path
        let log_path = "tests/ack_sent_back/log.txt";

        // Open the log file in write mode
        let mut log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)
            .expect("Failed to open or create log file");

        //Create a generic fragment packet with a hardcoded source_routing_header and send it to the neighbour drone!
        let generic_fragment = Fragment {
            fragment_index: 0,
            total_n_fragments: 0,
            length: 0,
            data: [0; FRAGMENT_DSIZE],
        };

        let source_routing_header = SourceRoutingHeader {
            hop_index: 1,
            hops: vec![10, 20, 30, 40],
        };

        let packet = Packet {
            pack_type: PacketType::MsgFragment(generic_fragment),
            routing_header: source_routing_header,
            session_id: 0,
        };

        let log_msg = format!("[CLIENT {}] Message fragment sent. Source routing header hops: {:?}\n", self.id, packet.routing_header.hops);
        self.forward_packet(packet);
        eprintln!("{}", log_msg);
        log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");

        // Process the first incoming packet (should be a Nack)
        select_biased!(
            recv(self.packet_recv) -> packet_res => {
                if let Ok(packet) = packet_res {
                    match packet.pack_type {
                        PacketType::Ack(ref msg_fragment) => {
                            let log_msg = format!("[CLIENT {}] Ack received successfully. Packet path: {:?}\n", self.id, packet.routing_header.hops);
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
    
    //--------------------------------
    
    pub fn run_crash_test(&self){
        // Define the log file path
        let log_path = "tests/crash_test/log.txt";
        
        // Open the log file in write mode
        let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_path)
        .expect("Failed to open or create log file");
        
        //Create a generic fragment packet with a hardcoded source_routing_header and send it to the neighbour drone
        let generic_fragment = Fragment {
            fragment_index: 0,
            total_n_fragments: 0,
            length: 0,
            data: [0; FRAGMENT_DSIZE],
        };
        
        let source_routing_header = SourceRoutingHeader {
            hop_index: 1,
            hops: vec![ 20, 30, 40, 50],
        };
        
        let packet = Packet {
            pack_type: PacketType::MsgFragment(generic_fragment),
            routing_header: source_routing_header,
            session_id: 0,
        };
        
        let log_msg = format!("[CLIENT {}] Message fragment sent. Source routing header hops: {:?}\n", self.id, packet.routing_header.hops);
        self.forward_packet(packet);
        eprintln!("{}", log_msg);
        log_file.write_all(log_msg.as_bytes()).expect("Failed to write to log file");
        
    }
    
}
