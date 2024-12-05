use crate::utils::NetworkUtils;
use crossbeam_channel::{select_biased, Receiver, Sender};
use indexmap::IndexSet;
use rand::prelude::ThreadRng;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use wg_2024::{
    config::{Client, Config, Drone, Server},
    controller::DroneEvent,
    network::{NodeId, SourceRoutingHeader},
    packet::{FloodRequest, NodeType, Packet, PacketType},
};

pub struct ClientNode {
    id: NodeId,
    sim_contr_send: Sender<DroneEvent>,
    sim_contr_recv: Receiver<ClientCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    seen_flood_ids: IndexSet<u64>,
    topology: Option<Config>,
    random_generator: ThreadRng,
}

impl NetworkUtils for ClientNode {
    fn get_id(&self) -> NodeId {
        self.id
    }

    fn get_packet_senders(&self) -> &HashMap<NodeId, Sender<Packet>> {
        &self.packet_send
    }

    fn get_random_generator(&mut self) -> &mut ThreadRng {
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
            random_generator: thread_rng(),
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
                            PacketType::MsgFragment(ref _fragment) => eprintln!("[CLIENT {}] MsgFragment received.", self.id),
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
}
