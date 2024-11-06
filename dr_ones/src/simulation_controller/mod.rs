//use crate::client;
use super::types::message::Message;
use crate::types::topology::Node;
use super::api::simulation_controller::SimulationController as SimContrTrait;

pub struct SimulationController {
    receiver: tokio::sync::mpsc::Receiver<Message>,
}

impl SimContrTrait for SimulationController {
    fn crash(&mut self, crashed: &str) {

    }
    fn spawn_node(&mut self, new_node: Node /*metadata*/) {

    }
    fn message_sent<'a>(source: &'a str, target: &'a str /*metadata*/) {

    }
}

impl SimulationController {
    pub fn new(receiver: tokio::sync::mpsc::Receiver<Message>) -> Self {
        SimulationController { receiver }
    }

    pub async fn start(&mut self) {
        println!("SimulationController started");

        // Wait for network initializer
        let clients = self.receive_client_information();

        // TODO Create gui
    }

    fn receive_client_information(&mut self) {
        //tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let result = self.receiver.blocking_recv();
        // TODO return list of nodes
    }
}
