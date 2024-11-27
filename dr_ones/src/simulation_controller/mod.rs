use wg_2024::packet::{Message, NodeType};
use wg_2024::network::NodeId;
use crossbeam_channel;

pub trait SimContrTrait {
    fn crash(&mut self, crashed: &str);
    fn spawn_node(&mut self, node_id: NodeId, node_type: NodeType /*metadata*/);
    fn message_sent(source: &str, target: &str /*metadata*/);
}

pub struct SimulationController {
    receiver: crossbeam_channel::Receiver<Message>,
}

impl SimContrTrait for SimulationController {
    fn crash(&mut self, crashed: &str) {
        // Implement crash logic
    }

    fn spawn_node(&mut self, node_id: NodeId, node_type: NodeType /*metadata*/) {
        // Implement spawn_node logic
    }

    fn message_sent<'a>(source: &'a str, target: &'a str /*metadata*/) {
        // Implement message_sent logic
    }
}

impl SimulationController{
    pub fn new(receiver: crossbeam_channel::Receiver<Message>) -> Self {
        SimulationController { receiver }
    }

    pub fn start(&mut self) {
        println!("SimulationController started");

        // Wait for network initializer
        let clients = self.receive_client_information();

        // TODO: Create GUI
    }

    fn receive_client_information(&mut self) {
        match self.receiver.recv() {
            Ok(message) => {
                println!("Received message: {:?}", message);
                // TODO: Process the message and return list of nodes
                //return elements;
            }
            Err(err) => {
                eprintln!("Error receiving message: {:?}", err);
                // Handle the error if necessary
            }
        }
    }
}
