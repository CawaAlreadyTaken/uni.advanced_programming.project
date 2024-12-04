use crossbeam_channel::{select, Receiver, Sender};
use std::collections::{HashMap, HashSet};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{ FloodRequest,FloodResponse, NodeType, Packet, PacketType, Nack, NackType};
use rand::{thread_rng, Rng};
use indexmap::IndexSet;
use rand::prelude::ThreadRng;

/// Example of drone implementation
pub struct Dr_One {
    id: NodeId,
    sim_contr_send: Sender<DroneEvent>,
    sim_contr_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: f32,
    seen_flood_ids: IndexSet<u64>,
    random_generator: ThreadRng
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
            pdr,
            packet_send,
            seen_flood_ids: IndexSet::new(),
            random_generator: thread_rng(),
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
                // handle receiving a packet from another drone
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        match packet.pack_type{
                            PacketType::FloodRequest(ref _flood_req) => self.handle_flood_request(packet), // flood request are particular because the recipient is not specified
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
                            DroneCommand::Crash => self.crash(),   
                            DroneCommand::RemoveSender(node_id) => self.remove_channel(node_id),
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
    
    // Handle routed packet and check if its routing is correct. All the packets are routed except flood requests.
    // If the routing is correct then process it depending on its type else send back a nack.
    fn handle_routed_packet(& mut self, mut packet: Packet) {
        
        // eprintln!("[DRONE {}] I am handling packet {}.", self.id, packet.session_id.clone());
        
        // 1. Check if the drone is the expected recipient of the packet 
        
        let index = packet.routing_header.hop_index;
        if self.id != packet.routing_header.hops[index]{
            // the drone is not the expected recipient. A nack of type UnexpectedRecipient needs to be sent back
            
            packet.routing_header.hop_index += 1;
            
            let packet = self.build_nack(packet,NackType::UnexpectedRecipient(self.id.clone()));
        
            self.forward_packet(packet);
            
            //TODO: print what is going on for debugging purpose ?
            
            return;   
        }
        
        // the drone is the expected recipient of the packet
        
        // 2. Increment hop_index by 1
        packet.routing_header.hop_index += 1;
        
        // 3. Determine if the drone is the final destination of the packet
        if packet.routing_header.hop_index == packet.routing_header.hops.len(){
            // the drone is the final destination of the packet. A nack needs to be sent back
            
            let packet = self.build_nack(packet,NackType::DestinationIsDrone);
            
            self.forward_packet(packet);
            
            //TODO: print what is going on for debugging purpose ?
            
            return; 
        }
        
        // the drone is not the final destination of the packet
        
        // 4. Identify the next hop using hops[hop_index], this node is called next_hop. 
        // If next_hop is not a neighbour of self then a nack needs to be sent back.
        
        let next_hop_id = packet.routing_header.hops[packet.routing_header.hop_index];
        
        let is_not_a_neighbour:bool = matches!(self.packet_send.get(&next_hop_id),None);
        
        if is_not_a_neighbour{
            // next_hop is not a neighbour of self
            
            let packet = self.build_nack(packet,NackType::ErrorInRouting(next_hop_id));
            
            self.forward_packet(packet);
            
            //TODO: print what is going on for debugging purpose ?
            
            return; 
        }
        
        // next_hop is a neighbour of self
        
        // 5. Proceed based on the packet type
        
        match packet.pack_type {
            PacketType::Nack(ref _nack) => self.forward_packet(packet),
            PacketType::Ack(ref _ack) => self.forward_packet(packet),
            PacketType::FloodResponse(ref _flood_res ) => self.forward_packet(packet),
            PacketType::MsgFragment(ref _fragment) => {
                
                // a. Determine whether to drop the packet based on the drone's Packet Drop Rate (PDR).
                
                let pdr_scaled = (self.pdr *100.0) as i32;
                let random_number = rand::thread_rng().gen_range(0..=100);
                
                let is_dropped:bool = random_number < pdr_scaled;
                
                if is_dropped{
                    // the packet is dropped. A nack needs to be sent back
                    
                    let mut packet = self.build_nack(packet,NackType::Dropped);
                    
                    self.reverse_packet_routing_direction(&mut packet);
                    
                    self.forward_packet(packet);
                    
                    //TODO: print what is going on for debugging purpose ?
                    
                    return;
                }
                
                // the packet is not dropped
                
                // b. Send the packet to the neighbour
                
                self.forward_packet(packet);
                
            },
            _ => eprintln!("Received unhandled packet type: {:?}", packet.pack_type), //for debugging purpose
        } 
    }
    
    // Return a packet which pack_type attribute is nack of type NackType
    fn build_nack(&self, packet: Packet, nack_type : NackType) -> Packet{
        
        // 1. Keep in the nack the fragment index if the packet contains a fragment
        let frag_index:u64;
        
        if let PacketType::MsgFragment(fragment) = &packet.pack_type {
            frag_index = fragment.fragment_index;
        }
        else{
            frag_index = 0;
        }
        
        // 2. Build the Nack instance of the packet to return 
        let nack:Nack = Nack { 
            fragment_index: frag_index, 
            nack_type 
        };
        
        // 3. Build the packet
        let packet_type = PacketType::Nack(nack);
        
        let mut packet:Packet = Packet { 
            pack_type: packet_type,
            routing_header: packet.routing_header, 
            session_id: packet.session_id,
        };
        
        // 4. Reverse the routing direction of the packet because nacks need to be sent back

        self.reverse_packet_routing_direction(&mut packet);
        
        // 5. Return the packet
        packet
    }
    
    // Return a packet which pack_type attribute is FloodResponse
    fn build_flood_reponse(&mut self, packet: Packet, updated_path_trace:Vec<(NodeId, NodeType)>) -> Packet{

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

            let mut route_back:Vec<u8> = flood_response.path_trace.iter().map(|tuple| tuple.0).collect();
            // route_back.push(self.id.clone());
            route_back.reverse();

            let new_routing_header = SourceRoutingHeader{
                hop_index:1,
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
        }
        else{
            eprintln!("Error ! Attempt of building a flood response over a packet that is not a flood request.");
            panic!();
        }
    }

    
    // handle a received flood request depending on the neighbours of the drone and on the flood request
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

            // If I have only one neighbour, I must have received this message from it and i don't have anybody else to forward it to
            let has_no_neighbour:bool = self.packet_send.len() == 1;
            
            // 2. Check if the flood request should be sent back as a flood response or broadcast as is
            if flood_request_is_already_received || has_no_neighbour {
                // A flood response should be created and sent

                // a. Create a build response based on the build request

                let flood_response_packet = self.build_flood_reponse(packet, flood_request.path_trace);

                if flood_request_is_already_received {
                    // eprintln!("[DRONE {}] Flood request {} (received from {}) has already been received", self.id, flood_request.flood_id, who_sent_me_this_flood_request);
                }
                
                // b. forward the flood response back
                eprintln!("[DRONE {}] Sending FloodResponse sess_id:{} whose path is: {:?}", self.id, flood_response_packet.session_id, flood_response_packet.routing_header.hops);
                self.forward_packet(flood_response_packet);
            }
            else {
                // The packet should be broadcast
                // eprintln!("Drone id: {} -> flood_request with path_trace: {:?} broadcasted to peers: {:?}", self.id, flood_request.path_trace, self.packet_send.keys());
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
    

    // forward the packet to the neighbour node as specified in the routing header.
    fn forward_packet(&self, packet: Packet) {
        let next_hop_id = packet.routing_header.hops[packet.routing_header.hop_index];
        let sess_id = packet.session_id; //TODO: remove. This only needs to log what is happening

        if let PacketType::FloodResponse(flood_response) = &packet.pack_type {
            //test
            // eprintln!("{} -> {} : packet_session_id {}", self.id, next_hop_id, sess_id);
        }

        // forward the packet to the next actor
        if let Some(sender) = self.packet_send.get(&next_hop_id) {
            //we are giving away the ownership of the packet
            sender.send(packet).expect("Failed to forward the packet");
        } else {
            println!("No channel found for next hop: {:?}", next_hop_id);
        }

    }
    
    // reverse the packet route in order it to be sent back.
    // In the end, the packet should go to the node (server or client) that initially routed the packet.
    fn reverse_packet_routing_direction(&self, packet:&mut Packet){
        
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
            hops: hops_vec
        };

        // b. update the packet's routing header
        packet.routing_header = route_back;            
    }

    // @Fede, I did not modify you work below

   /*  forward the packet to all the neighbour nodes in a flooding context.
    fn broadcast_packet(&self, packet: Packet) {
        // iterate on the neighbours list
        for (&node_id, sender) in self.packet_send.iter() {
    
            // Send a clone packet
            if let Err(e) = sender.send(packet.clone()) {
                println!("Failed to send packet to NodeId {:?}: {:?}", node_id, e);
            }
        }
    } */
    
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