use crossbeam_channel::{select, Receiver, Sender};
use std::collections::HashMap;
use std::thread;
use wg_2024::controller::Command;
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Packet, PacketType, Fragment, FragmentData};
use wg_2024::drone::DroneOptions;

/// Example of drone implementation
pub struct Dr_One {
    id: NodeId,
    sim_contr_send: Sender<Command>,
    sim_contr_recv: Receiver<Command>,
    packet_recv: Receiver<Packet>,
    pdr: u8,
    packet_send: HashMap<NodeId, Sender<Packet>>,
}

impl Drone for Dr_One {
    fn new(options: DroneOptions) -> Self {
        Self {
            id: options.id,
            sim_contr_send: options.sim_contr_send,
            sim_contr_recv: options.sim_contr_recv,
            packet_recv: options.packet_recv,
            pdr: (options.pdr * 100.0) as u8,
            packet_send: HashMap::new(),
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
                            PacketType::Nack(ref _nack) => self.send_packet_back(&packet), // unimplemented!(), // ,
                            PacketType::Ack(ref _ack) => unimplemented!(),
                            PacketType::MsgFragment(ref _fragment) => self.forward_packet(&packet),
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

     // send the a packet back (ack or nack) to the neighbour drone.
    // In the end, the packet should go to the client/node that routed the packet.
    fn send_packet_back(&self,packet: &Packet){
        println!("Nack received for packet.");
        // 1. get the index i of the current node in the SourceRoutingHeader of the packet
        let source_routing_header = &packet.routing_header;
        let drone_id = self.id;
        let current_drone_index = source_routing_header.hops.iter().position(|&x| x == drone_id).unwrap();
       
        // 2. get the crossbeam Sender of the drone to send the nack
        let target_drone_id = source_routing_header.hops[current_drone_index-1];
        let sender_target_drone:&Sender<Packet> = self.packet_send.get(&target_drone_id).unwrap();
       
        // 3. TODO: send the nack to the node of index i-1 in the SourceRoutingHeader of the packet
        //sender_target_drone.send(packet).unwrap();
    }

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
                            data: FragmentData{
                                length: 0,
                                data: [0; 80],
                            },
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
    
    // fn remove_channel(...) {...}
}