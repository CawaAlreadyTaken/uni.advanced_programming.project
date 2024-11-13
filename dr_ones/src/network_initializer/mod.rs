use crate::types::message::Message;
mod parser;
//use crate::types::message::MessageContent;
//use crate::types::NodeId;
//use crate::types::SourceRoutingHeader;

pub struct NetworkInitializer {
    sender: tokio::sync::mpsc::Sender<Message>,
}

impl NetworkInitializer {
    pub fn new(sender: tokio::sync::mpsc::Sender<Message>) -> Self {
        NetworkInitializer { sender }
    }

    pub async fn start(&mut self) {
        println!("NetworkInitializer started");

        // Read and parse network intialization file
        let parsed_config = parser::parse("init.toml");

        // TODO: initialize nodes

        self.send_nodes_to_simulation_controller();

        // This can now die
    }

    pub async fn send_nodes_to_simulation_controller(&self /*nodes_vector*/) {
        // TODO: change this so that it sends nodes
        /*
        let routing_header: SourceRoutingHeader = [0; 16];
        let source_id: NodeId = 0;
        let session_id = 27;
        let content: MessageContent = MessageContent::ReqMessageSend {
            to: 0,
            message: vec![3; 3],
        };
        let new_message = Message::new(routing_header, source_id, session_id, content);
        self.sender.send(new_message).await.unwrap();
        */
    }
}
