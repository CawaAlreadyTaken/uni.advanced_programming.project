use crossbeam_channel::{select, Receiver, Sender};
use std::collections::HashMap;
use wg_2024::controller::{DroneCommand, NodeEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{NodeType, Packet, PacketType};
use wg_2024::drone::DroneOptions;

/// Example of drone implementation
pub struct Dr_One {
    id: NodeId,
    sim_contr_send: Sender<NodeEvent>,
    sim_contr_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: u8,
}

impl Drone for Dr_One {
    fn new(options: DroneOptions) -> Self {
        Self {
            id: options.id,
            sim_contr_send: options.controller_send,
            sim_contr_recv: options.controller_recv,
            packet_recv: options.packet_recv,
            pdr: (options.pdr * 100.0) as u8,
            packet_send: options.packet_send,
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
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        // each match branch may call a function to handle it to make it more readable
                        match packet.pack_type {
                            PacketType::Nack(ref _nack) => self.send_packet_back(packet,&"Nack sent back."), 
                            PacketType::Ack(ref _ack) => self.send_packet_back(packet,&"Ack sent back."),
                            PacketType::MsgFragment(ref _fragment) => self.forward_packet(packet),
                            PacketType::FloodRequest(ref _floodReq) => self.broadcast_packet(packet),
                            PacketType::FloodResponse(ref _floodRes ) => self.send_packet_back(packet, &"Flood request sent back."), 
                        }
                    }
                },
                recv(self.sim_contr_recv) -> command_res => {
                    if let Ok(command) = command_res {

                        // TODO
                        // each match branch may call a function to handle it to make it more readable
                        // match command {
                        //     DroneCommand::AddSender,
                        //     DroneCommand
                        //     DroneCommand:: => !unimplemented,
                        //     ,
                        //     ,
                            // PacketType::Nack(ref _nack) => self.send_packet_back(packet,&"Nack sent back."), 
                            // PacketType::Ack(ref _ack) => self.send_packet_back(packet,&"Ack sent back."),
                            // PacketType::MsgFragment(ref _fragment) => self.forward_packet(packet),
                            // PacketType::FloodRequest(ref _floodReq) => self.broadcast_packet(packet),
                            // PacketType::FloodResponse(ref _floodRes ) => self.send_packet_back(packet, &"Flood request sent back."), 
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
        self.packet_send.get(&id).expect(&format!("Error ! The current node {} has no neighbour node {}.",self.id,id)); 
        self.packet_send.remove(&id);
    }
    
    // send the a packet back (e.g. ack, nack or flood request) to the neighbour node.
    // In the end, the packet should go to the node (server or client) that initially routed the packet.
    fn send_packet_back(&self, mut packet: Packet, message: &str){
        println!("{message}");

        //1. incrementing the hop index of the packet
        packet.routing_header.hop_index+=1;

        // 2. get the index i of the current node in the SourceRoutingHeader of the packet
        let source_routing_header = &packet.routing_header;
        let drone_id = self.id.clone();
        let current_drone_index = source_routing_header.hops.iter().position(|&x| x == drone_id).unwrap();
        
        // 3. get the crossbeam Sender of the target node
        let target_node_id = source_routing_header.hops[current_drone_index-1];
        
        // 4. send the packet to the node of index i-1 in the SourceRoutingHeader of the packet
        if let Some(sender) = self.packet_send.get(&target_node_id) {
            sender.send(
                packet
            ).expect("Failed to send the packet pack");
        } else {
            println!("No channel found for next hop: {:?}", target_node_id);
        }
    }
    
    // forward the packet to the neighbour drone as specified in the routing header.
    fn forward_packet(&self, mut packet: Packet) {
        // check if the packet.routing_header.hops[packet.routing_header.hop_index] corresponds to my id
        let index = packet.routing_header.hop_index;
        if self.id == packet.routing_header.hops[index] {
            
            //we have ownership of the packet now
            packet.routing_header.hop_index += 1;
            let next_hop_id = packet.routing_header.hops[packet.routing_header.hop_index];
            
            // forward the packet to the next actor
            if let Some(sender) = self.packet_send.get(&next_hop_id) {
                //we are giving away the ownership of the packet
                sender.send(packet).expect("Failed to forward the packet");
            } else {
                println!("No channel found for next hop: {:?}", next_hop_id);
            }
        }
    }
    
    /*
    // forward the packet to all the neighbour nodes in a flooding context.
    fn broadcast_packet(&self, mut packet: Packet) {
        // Ensure the packet is a FloodRequest
        if let PacketType::FloodRequest(mut flood_request) = packet.pack_type {
            // TODO: send 'packet' to all the nodes that fullfill these requirements:
            //       o their NodeId is a key in the Hashmap attribute of self named 'packet_send' (the node is a neighbour of self)
            //       o their NodeId is not present in the attribute path_trace of 'packet'  (the node hasn't received the flood request yet)

            // I put myself in the path_trace
           flood_request.path_trace.push((self.id, NodeType::Drone));

            // Iterate over all drone neighbors in the packet_send map
            for (node_id, sender) in self.packet_send.iter() {
                // Check if the node is not already in the path_trace
                if !flood_request.path_trace.iter().any(|(id, _)| *id == node_id) {
                    // Send the packet to this neighbor
                    if let Err(e) = sender.send(packet.clone()) {
                        println!("Failed to send packet to NodeId {:?}: {:?}", node_id, e);
                    }
                }
            } 
        } else {
            println!("Packet is not a FloodRequest, skipping.");
        }
    }*/

    fn broadcast_packet(&self, mut packet: Packet) {

        // TODO: THINK MORE ABOUT WHAT TO DO WHEN RECEIVING A FLOOD REQUEST.
        // BY NOT CARING, WE RISK A SHIT LOAD OF MESSAGES GOING AROUND !!

        // Ensure the packet is a FloodRequest
        if let PacketType::FloodRequest(ref mut flood_request) = packet.pack_type {
            // Add self to the path_trace
            flood_request.path_trace.push((self.id, NodeType::Drone));

            // Iterate over all drone neighbors in the packet_send map
            for (&node_id, sender) in self.packet_send.iter() {
                // Check if the node is not already in the path_trace
                if !flood_request.path_trace.iter().any(|(id, _)| *id == node_id) {
                    // Clone the modified packet for each neighbor
                    // let mut neighbor_packet = packet.clone();

                    // // Add the neighbor to the path_trace of the cloned packet
                    // if let PacketType::FloodRequest(ref mut neighbor_flood_request) =
                    //     neighbor_packet.pack_type
                    // {
                    //     neighbor_flood_request.path_trace.push((node_id, NodeType::Drone));
                    // }

                    // // Send the cloned packet
                    // if let Err(e) = sender.send(neighbor_packet) {
                    //     println!("Failed to send packet to NodeId {:?}: {:?}", node_id, e);
                    // }
                }
            }
        } else {
            println!("Packet is not a FloodRequest, skipping.");
        }
    }
}