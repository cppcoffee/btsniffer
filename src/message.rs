use std::net::{IpAddr, SocketAddr};

// announcement message.
#[derive(Debug)]
pub struct Message {
    pub peer: SocketAddr,
    pub infohash: Vec<u8>,
}

impl Message {
    pub fn new(ip: IpAddr, port: u16, hash: &[u8]) -> Self {
        Self {
            peer: SocketAddr::new(ip, port),
            infohash: hash.to_vec(),
        }
    }
}
