use crate::errors::{Error, Result};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

const NODE_BYTES_LENGTH: usize = 26;

// decode nodes from bytes.
pub fn decode_nodes(s: &[u8]) -> Result<Vec<Node>> {
    if s.len() % NODE_BYTES_LENGTH != 0 {
        return Err(Error::Other(format!(
            "invalid replay 'nodes' length={}",
            s.len()
        )));
    }

    let n = s.len() / NODE_BYTES_LENGTH;
    let mut res = Vec::new();
    for i in 0..n {
        let pos = i * NODE_BYTES_LENGTH;
        let node = Node::from_bytes(&s[pos..pos + NODE_BYTES_LENGTH]);
        res.push(node);
    }
    Ok(res)
}

// DHT node
#[derive(Debug)]
pub struct Node {
    pub id: Vec<u8>,
    pub addr: SocketAddr,
}

impl Node {
    pub fn from_bytes(s: &[u8]) -> Self {
        assert!(s.len() >= NODE_BYTES_LENGTH);

        let id = s[..20].to_vec();
        let ip = Ipv4Addr::new(s[20], s[21], s[22], s[23]);
        let port = u16::from_be_bytes([s[24], s[25]]);
        let addr = SocketAddr::new(IpAddr::V4(ip), port);

        Self { id, addr }
    }
}
