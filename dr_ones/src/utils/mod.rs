use wg_2024::packet::{NodeType, Packet, PacketType};

//TODO: put duplicate "generate_flood_response" here and import it elsewhere
pub fn handle_flood_request(&mut self, packet:Packet) {
    if let PacketType::FloodRequest(mut flood_request) = packet.pack_type.clone() {
        flood_request.path_trace.push((self.id, NodeType::Client));
        eprintln!("[CLIENT {}] FloodRequest {} received with pathTrace: {:?}", self.id, flood_request.flood_id, flood_request.path_trace);
        //just generate a flood response and send it back
        let flood_response_packet:Packet = self.build_flood_reponse(packet, flood_request.path_trace);
        eprintln!("[CLIENT {}] Sending FloodResponse sess_id:{} whose path is: {:?}", self.id, flood_response_packet.session_id, flood_response_packet.routing_header.hops);
        self.forward_packet(flood_response_packet);
    }
}
