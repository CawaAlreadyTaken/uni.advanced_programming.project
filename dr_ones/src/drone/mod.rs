use std::collections::HashMap;

use crate::types::packet::Packet;
use crate::{api::drone::DroneAble, types::NodeId};
use tokio::sync::mpsc::{Receiver, Sender};

pub struct Drone {
    id: NodeId,
    packet_drop_rate: f64,
    channels_map: MapFromNodeIdToChannels,
}

type MapFromNodeIdToChannels = HashMap<NodeId, (Receiver<Packet>, Sender<Packet>)>;

/*
map: MapFromNodeIdToChannels = {
    4: (receiver_channel_0, sender_channel_0),
    10: (receiver_channel_1, sender_channel_1),
    2: (receiver_channel_2, sender_channel_2),
    43: (receiver_channel_3, sender_channel_3),
}
*/

impl DroneAble for Drone {
    fn forward_packet(&self, packet: Packet) -> bool {
        let random_number = rand::random::<f64>();
        if random_number < self.packet_drop_rate {
            // TODO: when dropping, we need to notify the sender
            return false;
        }

        let hops: Vec<NodeId> = packet.routing_header.hops.clone();
        let next_hop_channel: &Sender<Packet> = self.find_next_hop_channel(hops);
        next_hop_channel.send(packet);
        true
    }
}

impl Drone {
    pub fn new(id: NodeId, pdr: f64) -> Self {
        Drone {
            id,
            packet_drop_rate: pdr,
            channels_map: HashMap::new(),
        }
    }

    fn find_next_hop_channel(&self, hops: Vec<NodeId>) -> &Sender<Packet> {
        // find the element following our nodeId in hops
        let next_hop = hops.iter().position(|&x| x == self.id).unwrap() + 1;
        let next_hop_node_id = hops[next_hop];
        let (_, next_hop_channel) = self.channels_map.get(&next_hop_node_id).unwrap();
        next_hop_channel
    }

    pub fn idle(&mut self) {
        loop {
            let mut packets_to_forward = Vec::new();

            for (receiving_channel, _) in self.channels_map.values_mut() {
                if let Ok(packet) = receiving_channel.try_recv() {
                    packets_to_forward.push(packet);
                }
            }

            for packet in packets_to_forward {
                self.forward_packet(packet);
            }

            // TODO: sleep for a while maybe?
        }
    }
}
