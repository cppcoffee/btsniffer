use std::net::SocketAddr;

// announcement message.
#[derive(Debug)]
pub struct Message {
    pub peer: SocketAddr,
    pub infohash: Vec<u8>,
}
