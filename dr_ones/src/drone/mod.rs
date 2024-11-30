use crossbeam_channel::{select, Receiver, Sender};
use std::collections::{HashMap, HashSet};
use std::fmt::format;
use wg_2024::controller::{DroneCommand, NodeEvent};
use wg_2024::drone::Drone;
use wg_2024::drone::DroneOptions;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{ FloodResponse, NodeType, Packet, PacketType};

/// Example of drone implementation
pub struct Dr_One {
    id: NodeId,
    sim_contr_send: Sender<NodeEvent>,
    sim_contr_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: f32,
    seen_flood_ids: HashSet<u64>,
}

impl Drone for Dr_One {
    fn new(options: DroneOptions) -> Self {
        Self {
            id: options.id,
            sim_contr_send: options.controller_send,
            sim_contr_recv: options.controller_recv,
            packet_recv: options.packet_recv,
            pdr: options.pdr,
            packet_send: options.packet_send,
            seen_flood_ids: HashSet::new(),
        }
    }
    
    fn run(&mut self) {
        self.run_internal();
    }
}

impl Dr_One {
    fn run_internal(&mut self) {
        loop {
            select! {
                // handle receiving a packet
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        // each match branch may call a routine to handle it to make it more readable
                        match packet.pack_type {
                            PacketType::Nack(ref _nack) => self.send_packet_back(packet,&"Nack sent back."),
                            PacketType::Ack(ref _ack) => self.send_packet_back(packet,&"Ack sent back."),
                            PacketType::MsgFragment(ref _fragment) => self.forward_packet(packet),
                            PacketType::FloodRequest(ref _flood_req) => self.handle_flood_request(packet),
                            PacketType::FloodResponse(ref _flood_res ) => self.send_packet_back(packet, &"Flood request sent back."),
                        }
                    }
                },
                
                // handle receiving a message from the simulation controller
                recv(self.sim_contr_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        
                        //each match branch may call a routine to handle it to make it more readable
                        match command {
                            DroneCommand::AddSender(node_id,sender) => self.add_channel(node_id,sender),
                            DroneCommand::SetPacketDropRate(new_pdr) => self.set_pdr(new_pdr),
                            DroneCommand::Crash => self.crash(),   
                        }
                    }
                }
            }
        }
    }
    
    // add the node with identifier 'id' and crossbeam channel sender 'sender' to the list of neighbour nodes of self
    fn add_channel(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.packet_send.insert(id, sender);
    }
    
    // remove the neighbour node of id 'id' from the list of neighbour nodes of self
    fn remove_channel(&mut self, id: NodeId) {
        self.packet_send.get(&id).expect(&format!(
            "Error ! The current node {} has no neighbour node {}.",
            self.id, id
        ));
        self.packet_send.remove(&id);
    }
    
    // send the a packet back (e.g. ack, nack or flood request) to the neighbour node.
    // In the end, the packet should go to the node (server or client) that initially routed the packet.
    fn send_packet_back(&self, mut packet: Packet, message: &str) {
        eprintln!("{message}");
        
        //1. incrementing the hop index of the packet
        packet.routing_header.hop_index += 1;
        
        // 2. get the index i of the current node in the SourceRoutingHeader of the packet
        let source_routing_header = &packet.routing_header;
        let drone_id = self.id.clone();
        let current_drone_index = source_routing_header
        .hops
        .iter()
        .position(|&x| x == drone_id)
        .unwrap();
        
        // 3. get the crossbeam Sender of the target node
        let target_node_id = source_routing_header.hops[current_drone_index - 1];
        
        // 4. send the packet to the node of index i-1 in the SourceRoutingHeader of the packet
        if let Some(sender) = self.packet_send.get(&target_node_id) {
            sender.send(packet).expect("Failed to send the packet pack");
        } else {
            println!("No channel found for next hop: {:?}", target_node_id);
        }
    }
    
    // forward the packet to the neighbour drone as specified in the routing header.
    fn forward_packet(&self, mut packet: Packet) {
        eprintln!("[DRONE {}] I am forwarding", self.id);
        // check if the packet.routing_header.hops[packet.routing_header.hop_index] corresponds to my id
        let index = packet.routing_header.hop_index;
        if self.id == packet.routing_header.hops[index] {
            //we have ownership of the packet now
            packet.routing_header.hop_index += 1;
            let next_hop_id = packet.routing_header.hops[packet.routing_header.hop_index];
            let sess_id = packet.session_id; //TODO: remove. This only needs to log what is happening
            
            // forward the packet to the next actor
            if let Some(sender) = self.packet_send.get(&next_hop_id) {
                //we are giving away the ownership of the packet
                sender.send(packet).expect("Failed to forward the packet");
            } else {
                println!("No channel found for next hop: {:?}", next_hop_id);
            }

            eprintln!("{} -> {} : packet_session_id {}", self.id, next_hop_id, sess_id);
        }
    }
    
    // handle the flood request depending on the neighbours of the drone and on the flood request
    fn handle_flood_request(&mut self, packet: Packet) {
        // Check if the flood request should be broadcast or turned into a flood response and sent back
        if let PacketType::FloodRequest(mut flood_request) = packet.pack_type.clone() {
            // Add self to the path trace
            flood_request.path_trace.push((self.id, NodeType::Drone));

            // 1. Process some tests on the drone and its neighbours to know how to handle the flood request

            // a. Check if the drone has already received the flood request
            let flood_request_is_already_received: bool = self.seen_flood_ids.iter().any(|id| *id == flood_request.flood_id);

            // b. Check if the drone has a neighbour, excluding the one from which it received the flood request

            // Copy the list of the neighbours and remove the neighbour drone that sent the flood request
            let neighbours_list: HashMap<u8, Sender<Packet>> = self.packet_send
                .iter()
                .filter(|(&key, _)| key != flood_request.path_trace[flood_request.path_trace.len() - 1].0)
                .map(|(k, v)| (*k, v.clone()))
                .collect();

            // Check if the updated neighbours list is empty
            let has_no_neighbour: bool = neighbours_list.is_empty();

            // 3. Check if the flood request should be sent back as a flood response or broadcast as is
            if flood_request_is_already_received || has_no_neighbour {
                // A flood response should be created and sent

                // a. create the route back of the packet to send back

                // copy the beginning of the routing vector, until the current drone
                let mut hops_vec: Vec<NodeId> = packet.routing_header.hops
                    .iter()
                    .take_while(|&node_id| *node_id != self.id)
                    .cloned()
                    .collect();

                // reverse the order of the nodes to reach in comparison with the original routing header
                hops_vec.reverse();

                // DES: Actually we don't need to add ourselves back in... we are already there and just need to explore the hops backwards
                // add the initiator of the flooding as the last receiver of the response
                // hops_vec.push(flood_request.initiator_id);

                let route_back: SourceRoutingHeader = SourceRoutingHeader {
                    hop_index: 0, // Start from the first hop
                    hops: hops_vec
                };

                // b. create the pack_type field of the packet to send back
                let flood_response: FloodResponse = FloodResponse {
                    flood_id: flood_request.flood_id,
                    path_trace: flood_request.path_trace,
                };

                // c. create the packet to send back
                let packet_back = Packet {
                    pack_type: PacketType::FloodResponse(flood_response),
                    routing_header: route_back,
                    session_id: packet.session_id.clone(),
                };

                // d. send the packet back
                // TODO: it is commented for now only because I'm testing other stuff
                // self.send_packet_back(packet_back, &format!("[DRONE {}] Created a flood RESPONSE that I just sent back", self.id));

                if flood_request_is_already_received {
                    eprintln!("[DRONE {}] Flood request {} is already received", self.id, flood_request.flood_id);
                }
            }
            else {
                // The packet should be broadcast
                eprintln!("Drone id: {} -> flood_request with path_trace: {:?} broadcasted to peers: {:?}", self.id, flood_request.path_trace, self.packet_send.keys());
                self.seen_flood_ids.insert(flood_request.flood_id);

                // Create the new packet with the updated flood_request
                let updated_packet = Packet {
                    pack_type: PacketType::FloodRequest(flood_request),
                    routing_header: packet.routing_header,
                    session_id: packet.session_id,
                };

                // Broadcast the updated packet
                self.broadcast_packet(updated_packet);
            }
        } else {
            println!("Error: the packet to be broadcast is not a flood request.");
        }
    }



    // forward the packet to all the neighbour nodes in a flooding context.
    fn broadcast_packet(&self, packet: Packet) {
        
        // iterate on the neighbours list
        for (&node_id, sender) in self.packet_send.iter() {
            
            // Send a clone packet
            if let Err(e) = sender.send(packet.clone()) {
                println!("Failed to send packet to NodeId {:?}: {:?}", node_id, e);
            }
        }
    }
    
    // set the packet drop rate of the drone as 'new_packet_drop_rate'
    fn set_pdr(&mut self, new_packet_drop_rate:f32){
        self.pdr = new_packet_drop_rate;
    }
    
    // crash the drone
    fn crash(&mut self){
        unimplemented!()
    }
}