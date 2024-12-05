use crossbeam_channel::Sender;
use rand::{prelude::ThreadRng, Rng};
use std::collections::HashMap;
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{FloodRequest, FloodResponse, NodeType, Packet, PacketType},
};

pub trait NetworkUtils {
    fn get_id(&self) -> NodeId;
    fn get_packet_senders(&self) -> &HashMap<NodeId, Sender<Packet>>;
    fn get_random_generator(&mut self) -> &mut ThreadRng;

    fn forward_packet(&self, packet: Packet) {
        let next_hop_id = packet.routing_header.hops[packet.routing_header.hop_index];

        if let Some(sender) = self.get_packet_senders().get(&next_hop_id) {
            sender.send(packet).expect("Failed to forward the packet");
        } else {
            println!("No channel found for next hop: {:?}", next_hop_id);
        }
    }

    fn build_flood_response(
        &mut self,
        packet: Packet,
        updated_path_trace: Vec<(NodeId, NodeType)>,
    ) -> Packet {
        if let PacketType::FloodRequest(flood_request) = packet.pack_type {
            let flood_response = FloodResponse {
                flood_id: flood_request.flood_id,
                path_trace: updated_path_trace,
            };

            let mut route_back: Vec<NodeId> = flood_response
                .path_trace
                .iter()
                .map(|tuple| tuple.0)
                .collect();
            route_back.reverse();

            let new_routing_header = SourceRoutingHeader {
                hop_index: 1,
                hops: route_back,
            };

            Packet {
                pack_type: PacketType::FloodResponse(flood_response),
                routing_header: new_routing_header,
                session_id: self.get_random_generator().gen(),
            }
        } else {
            panic!("Error! Attempt to build flood response from non-flood request packet");
        }
    }
}
