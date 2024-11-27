use wg_2024::drone::DroneOptions;
use wg_2024::packet::Message;
use wg_2024::packet::Packet;
use crossbeam_channel;
use std::{collections::HashMap, thread};
mod parser;
use crate::drone::Dr_One;
use wg_2024::drone::Drone;
//use crate::types::message::MessageContent;
//use crate::types::NodeId;
//use crate::types::SourceRoutingHeader;

pub struct NetworkInitializer {
    sender: crossbeam_channel::Sender<Message>,
}

impl NetworkInitializer {
    pub fn new(sender: crossbeam_channel::Sender<Message>) -> Self {
        NetworkInitializer { sender }
    }

    pub fn start(&mut self) {
        println!("NetworkInitializer started");

        // Read and parse network initialization file
        let parsed_config: parser::Config = parser::parse("init.toml");

        let handler = thread::spawn(move || {
            let id = 1;
            let (controller_send, _) = crossbeam_channel::unbounded();
            let (_, controller_recv) = crossbeam_channel::unbounded();
            let ( _packet_send, packet_recv) = crossbeam_channel::unbounded();

            let mut packet_send = HashMap::<u8, crossbeam_channel::Sender<Packet>>::new();
            packet_send.insert(1, _packet_send);

            let mut drone = Dr_One::new(DroneOptions {
                id,
                controller_recv,
                controller_send,
                packet_send,
                packet_recv,
                pdr: 0.1,
            });

            drone.run();
        });
        handler.join().ok();

        self.send_nodes_to_simulation_controller();

        // This can now die
    }

    pub fn send_nodes_to_simulation_controller(&self /*nodes_vector*/) {
        // TODO: Change this so that it sends nodes
        /*
        let routing_header: SourceRoutingHeader = [0; 16];
        let source_id: NodeId = 0;
        let session_id = 27;
        let content: MessageContent = MessageContent::ReqMessageSend {
            to: 0,
            message: vec![3; 3],
        };
        let new_message = Message::new(routing_header, source_id, session_id, content);
        self.sender.send(new_message).unwrap();
        */
    }
}
