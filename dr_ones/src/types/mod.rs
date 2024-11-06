// Should be u8
pub type NodeId = u64;

// False type to make cargo run happy
#[derive(Debug)]
pub struct SourceRoutingHeader {
    // Vector of nodes with initiator and nodes to which the packet will be forwarded to.
    pub hops: Vec<u64>,
}

pub mod message;
pub mod packet;
pub mod topology;
