use crossbeam_channel::{select, Receiver, Sender};
use std::collections::HashMap;
use std::thread;
use wg_2024::controller::{DroneCommand as Command, NodeEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Packet, PacketType, Fragment};
use wg_2024::drone::DroneOptions;

/// Example of drone implementation
pub struct Dr_One {
    id: NodeId,
    sim_contr_send: Sender<NodeEvent>,
    sim_contr_recv: Receiver<Command>,
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
            pdr: (options.pdr * 100.0) as u8, // why scaling ?
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
                            PacketType::Nack(ref _nack) => self.send_packet_back(&packet,&"Nack sent back."), 
                            PacketType::Ack(ref _ack) => self.send_packet_back(&packet,&"Ack sent back."),
                            PacketType::MsgFragment(ref _fragment) => self.forward_packet(&packet),
                            PacketType::FloodRequest(ref _floodReq) => self.broadcast_packet(&packet),
                            PacketType::FloodResponse(ref _floodRes ) => self.send_packet_back(&packet, &"Flood request sent back."), 
                        }
                    }
                },
                recv(self.sim_contr_recv) -> command_res => {
                    if let Ok(_command) = command_res {
                        // handle the simulation controller's command
                    }
                }
            }
        }
    }
    
    fn add_channel(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.packet_send.insert(id, sender);
    }
    
    // send the a packet back (e.g. ack, nack or flood request) to the neighbour node.
    // In the end, the packet should go to the node (server or client) that initially routed the packet.
    fn send_packet_back(&self,packet: &Packet,message: &str){
        println!("{message}");
        // 1. get the index i of the current node in the SourceRoutingHeader of the packet
        let source_routing_header = &packet.routing_header;
        let drone_id = self.id.clone();
        let current_drone_index = source_routing_header.hops.iter().position(|&x| x == drone_id).unwrap();
        
        // 2. get the crossbeam Sender of the target node
        let target_node_id = source_routing_header.hops[current_drone_index-1];
        
        // 3. send the packet to the node of index i-1 in the SourceRoutingHeader of the packet
        if let Some(sender) = self.packet_send.get(&target_node_id) {
            sender.send(
                packet.clone()
            ).expect("Failed to forward the packet");
        } else {
            println!("No channel found for next hop: {:?}", target_node_id);
        }
        // sender_target_node.send(packet).unwrap();
    }
    
    // forward the packet to the neighbour drone as specified in the routing header.
    fn forward_packet(&self, packet: &Packet) {
        // check if the packet.routing_header.hops[packet.routing_header.hop_index] corresponds to my id
        let index = packet.routing_header.hop_index;
        if self.id == packet.routing_header.hops[index] {
            
            // forward the packet to the next actor
            let next_hop_id = packet.routing_header.hops[packet.routing_header.hop_index + 1];
            
            // TODO: we are just creating a new packet for the moment. Maybe need to use Smart Pointers?
            if let Some(sender) = self.packet_send.get(&next_hop_id) {
                sender.send(
                    Packet{
                        pack_type: PacketType::MsgFragment(Fragment{
                            fragment_index: 0,
                            total_n_fragments: 1,
                            length:0,
                            data: [0;80],
                        }),
                        routing_header: SourceRoutingHeader{
                            hops: packet.routing_header.hops.clone(),
                            hop_index: packet.routing_header.hop_index + 1,
                        },
                        session_id: 12342,
                    }
                ).expect("Failed to forward the packet");
            } else {
                println!("No channel found for next hop: {:?}", next_hop_id);
            }
        }
    }
    
    // forward the packet to all the neighbour nodes in a flooding context.
    fn broadcast_packet(&self,packet: &Packet){
        // TODO: send 'packet' to all the nodes that fullfill these requirements:
        //       o their NodeId is a key in the Hashmap attribute of self named 'packet_send' (the node is a neighbour of self)
        //       o their NodeId is not present in the attribute path_trace of 'packet'  (the node hasn't received the flood request yet)
        unimplemented!();
    }
    
    // fn remove_channel(...) {...}
}