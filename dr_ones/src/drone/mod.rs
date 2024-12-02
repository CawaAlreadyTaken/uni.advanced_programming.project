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
                            // PacketType::Nack(ref _nack) => self.handle_nack(packet,&"Nack sent back."),
                            // PacketType::Ack(ref _ack) => self.handle_ack(packet,&"Ack sent back."),
                            // PacketType::MsgFragment(ref _fragment) => self.handle_msgFragment(packet),
                            PacketType::FloodRequest(ref _flood_req) => self.handle_flood_request(packet),
                            // PacketType::FloodResponse(ref _flood_res ) => self.handle_flood_response(packet),
                        _ => eprintln!("Received unhandled packet type: {:?}", packet.pack_type),}
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

    // handle the flood request depending on the neighbours of the drone and on the flood request
    fn handle_flood_request(&mut self, packet: Packet) {
        // Check if the flood request should be broadcast or turned into a flood response and sent back
        if let PacketType::FloodRequest(mut flood_request) = packet.pack_type.clone() {
            // Take who sent this floodRequest (test and logpurposes)
            let who_sent_me_this_flood_request = flood_request.path_trace.last().unwrap().0;

            // Add self to the path trace
            flood_request.path_trace.push((self.id, NodeType::Drone));

            // 1. Process some tests on the drone and its neighbours to know how to handle the flood request

            // a. Check if the drone has already received the flood request
            let flood_request_is_already_received: bool = self.seen_flood_ids.iter().any(|id| *id == flood_request.flood_id);

            // b. Check if the drone has a neighbour, excluding the one from which it received the flood request


            // Check if the updated neighbours list is empty
            // let has_no_neighbour: bool = neighbours_list.is_empty();

            //If I have only one neighbour, I must have received this message from it and i don't have anybody else to forward it to
            let has_no_neighbour:bool = (self.packet_send.len() == 1);

            // 3. Check if the flood request should be sent back as a flood response or broadcast as is
            if flood_request_is_already_received || has_no_neighbour {
                // A flood response should be created and sent

                // a. create the route back using the path trace of the flood_request
                let mut hops_vec: Vec<NodeId> = flood_request.path_trace.iter().map(|(node_id, _)| *node_id).collect();

                // reverse the order of the nodes to reach in comparison with the original routing header
                hops_vec.reverse();

                let route_back: SourceRoutingHeader = SourceRoutingHeader {
                    //THE SOURCEROUTINGHEADER SAYS THAT THE HOP INDEX SHOULD BE INITIALIZED AS 1, BUT
                    //KEEP IN MIND THAT IN THIS WAY THAT THE NODE THAT RECEIVES THIS PACKET WILL SEE ITSELF IN THE PATH_TRACE[HOP_INDEX]
                    hop_index: 1, // Start from the first hop
                    hops: hops_vec
                };

                // b. create the pack_type field of the packet to send back
                let flood_response: FloodResponse = FloodResponse {
                    flood_id: flood_request.flood_id,
                    path_trace: flood_request.path_trace,
                };

                // c. create the packet to send back
                let flood_response_packet = Packet {
                    pack_type: PacketType::FloodResponse(flood_response),
                    routing_header: route_back,
                    session_id: packet.session_id.clone(),
                };

                if flood_request_is_already_received {
                    eprintln!("[DRONE {}] Flood request {} (received from {}) has already been received", self.id, flood_request.flood_id, who_sent_me_this_flood_request);
                }

                // d. forward the packet
                self.forward_packet(flood_response_packet);
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
                self.broadcast_packet(updated_packet, who_sent_me_this_flood_request);
            }
        } else {
            eprintln!("Error: the packet to be broadcast is not a flood request.");
        }
    }

    fn handle_flood_response(&self, mut packet: Packet) {
        if let PacketType::FloodResponse(ref mut flood_response) = packet.pack_type {
            // Check if I'm the destination node
            if packet.routing_header.hop_index == packet.routing_header.hops.len()-1 &&
                packet.routing_header.hops[packet.routing_header.hop_index] == self.id {
                // If YES, fill my topology accordingly
                // TODO: implement
                eprintln!("my_ID: {}, I'm the receiver of the floodResponse", self.id);

            } else {
                // If NO, forward the packet
                packet.routing_header.hop_index += 1;
                self.forward_packet(packet);
            }
        }
    }


    // forward the packet to the neighbour node as specified in the routing header.
    fn forward_packet(&self, mut packet: Packet) {
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

    // forward the packet to all the neighbour nodes in a flooding context.
    // fn broadcast_packet(&self, packet: Packet) {
    //     // iterate on the neighbours list
    //     for (&node_id, sender) in self.packet_send.iter() {
    //
    //         // Send a clone packet
    //         if let Err(e) = sender.send(packet.clone()) {
    //             println!("Failed to send packet to NodeId {:?}: {:?}", node_id, e);
    //         }
    //     }
    // }

    // forward packet to a selected group of nodes in a flooding context
    fn broadcast_packet(&self, packet: Packet, who_i_received_the_packet_from:NodeId) {
        // Copy the list of the neighbours and remove the neighbour drone that sent the flood request
        let neighbours: HashMap<NodeId, Sender<Packet>> = self.packet_send
            .iter()
            .filter(|(&key, _)| key != who_i_received_the_packet_from)
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        // iterate on the neighbours list
        for (&node_id, sender) in neighbours.iter() {

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