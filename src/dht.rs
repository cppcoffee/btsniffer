use std::collections::HashMap;
use std::time::Duration;

use async_std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use async_std::sync::{channel, Arc, Mutex, Receiver, Sender};
use async_std::task;
use log::{debug, info};
use rand::prelude::*;

use crate::bencode::{self, Value};
use crate::node::decode_nodes;
use crate::util::{neighbor_id, rand_infohash_key, rand_transation_id};
use crate::{Error, Message, Rate, Result};

// recv buffer size.
const BUFFER_SIZE_MAX: usize = 2048;

// trackers
const SEEDS: [&'static str; 3] = [
    "router.bittorrent.com:6881",
    "dht.transmissionbt.com:6881",
    "router.utorrent.com:6881",
];

#[derive(Clone, Debug)]
pub struct DHT {
    laddr: Arc<String>,
    socket: Arc<Option<UdpSocket>>,
    local_id: Arc<Vec<u8>>,
    secret: Arc<Vec<u8>>,
    limiter: Arc<Mutex<Rate>>,
}

impl DHT {
    pub fn new(addr: &str, port: &str, limit: usize) -> Self {
        Self {
            laddr: Arc::new(format!("{}:{}", addr, port)),
            socket: Arc::new(None),
            local_id: Arc::new(rand_infohash_key()),
            secret: Arc::new(rand_infohash_key()),
            limiter: Arc::new(Mutex::new(Rate::new(limit))),
        }
    }

    pub async fn run(&mut self) -> Result<Receiver<Message>> {
        info!("DHT listen {}", self.laddr);

        let sock = UdpSocket::bind(self.laddr.as_ref()).await?;
        self.socket = Arc::new(Some(sock));

        let (tx, rx) = channel(2);

        self.start_message_handler(tx);
        self.start_join();

        Ok(rx)
    }

    fn start_join(&self) {
        const DHT_JOIN_COUNT: usize = 6;

        info!("start join DHT.");

        let this = self.clone();
        task::spawn(async move {
            for _ in 0..DHT_JOIN_COUNT {
                for seed in SEEDS.iter() {
                    match this.find_node(seed, &rand_infohash_key()).await {
                        Ok(n) => debug!("start_join find_node send {}, {} bytes", seed, n),
                        Err(e) => debug!("start_join find_node fail, {}", e),
                    }
                }

                let n = thread_rng().gen_range(2, 6);
                task::sleep(Duration::from_secs(n)).await;
            }
            debug!("join work leave.");
        });
    }

    fn start_message_handler(&mut self, tx: Sender<Message>) {
        let mut this = self.clone();

        task::spawn(async move {
            loop {
                match this.recv_message(&tx).await {
                    Ok(_) => {}
                    Err(e) => {
                        debug!("recv_message fail, {}", e);
                    }
                }
            }
        });
    }

    async fn recv_message(&mut self, tx: &Sender<Message>) -> Result<()> {
        let mut buf = [0; BUFFER_SIZE_MAX];

        let socket = (*self.socket).as_ref().ok_or(Error::InvalidUdpSocket)?;
        let (n, from) = socket.recv_from(&mut buf).await?;

        debug!("recv message {} bytes, from {}", n, from);

        // unpack bencode.
        let c = bencode::from_bytes(&buf[..n])?;
        let m = c
            .dict()?
            .get(b"y".as_ref())
            .ok_or(Error::DictNotFound("y".to_string()))?;

        match m.string()? {
            "q" => self.on_query(&c, &from, tx).await,
            "r" => self.on_reply(&c, &from).await,
            "e" => self.on_error(&c, &from),
            _ => Err(Error::InvalidPacket),
        }
    }

    fn on_error(&self, v: &Value, addr: &SocketAddr) -> Result<()> {
        let e = v
            .dict()?
            .get(b"e".as_ref())
            .ok_or(Error::DictNotFound("e".to_string()))?;

        let a = e.list()?;
        if a.len() != 2 {
            return Err(Error::InvalidPacket);
        }

        // decode error describe.
        let code = a[0].integer()?;
        let desc = a[1].string()?;

        debug!("on_error {} code: {}, description: {}", addr, code, desc);

        Ok(())
    }

    async fn on_query(&self, v: &Value, addr: &SocketAddr, tx: &Sender<Message>) -> Result<()> {
        // do check. is exist of the "t" field?
        v.dict()?.get(b"t".as_ref()).ok_or(Error::InvalidPacket)?;

        let q = v
            .dict()?
            .get(b"q".as_ref())
            .ok_or(Error::InvalidPacket)?
            .string()?;

        match q {
            "get_peers" => self.on_get_peers(v, addr).await,
            "announce_peer" => self.on_announce_peer(v, addr, tx).await,
            _ => Ok(()),
        }
    }

    async fn on_reply(&self, v: &Value, addr: &SocketAddr) -> Result<()> {
        let r = v
            .dict()?
            .get(b"r".as_ref())
            .ok_or(Error::DictNotFound("r".to_string()))?;

        let s = r
            .dict()?
            .get(b"nodes".as_ref())
            .ok_or(Error::DictNotFound("nodes".to_string()))?
            .bytes()?;

        let nodes = decode_nodes(s)?;
        debug!("on_reply {} decode {} nodes.", addr, nodes.len());

        let mut limiter = self.limiter.lock().await;
        for node in nodes {
            if !limiter.allow() {
                continue;
            }
            self.find_node(node.addr, &node.id).await?;
        }

        Ok(())
    }

    async fn on_get_peers(&self, v: &Value, addr: &SocketAddr) -> Result<()> {
        let tid = v
            .dict()?
            .get(b"t".as_ref())
            .ok_or(Error::DictNotFound("t".to_string()))?
            .bytes()?;

        let a = v
            .dict()?
            .get(b"a".as_ref())
            .ok_or(Error::DictNotFound("a".to_string()))?;

        let id = a
            .dict()?
            .get(b"id".as_ref())
            .ok_or(Error::DictNotFound("id".to_string()))?
            .bytes()?;

        let r = bencode::map!(
            b"id".to_vec() => Value::from(neighbor_id(id, self.local_id.as_ref())),
            b"nodes".to_vec() => Value::from(""),
            b"token".to_vec() => Value::from(self.make_token(addr))
        );

        let buf = self.make_reply(tid, r)?;
        if let Some(socket) = &*self.socket {
            socket.send_to(&buf, addr).await?;
        };

        Ok(())
    }

    async fn on_announce_peer(
        &self,
        v: &Value,
        addr: &SocketAddr,
        tx: &Sender<Message>,
    ) -> Result<()> {
        let a = v
            .dict()?
            .get(b"a".as_ref())
            .ok_or(Error::DictNotFound("a".to_string()))?;

        let token = a
            .dict()?
            .get(b"token".as_ref())
            .ok_or(Error::DictNotFound("token".to_string()))?
            .bytes()?;

        if !self.is_valid_token(token, addr) {
            return Err(Error::InvalidToken);
        }

        let ac = self.summarize(v, addr)?;
        tx.send(ac).await;

        Ok(())
    }

    fn summarize(&self, v: &Value, addr: &SocketAddr) -> Result<Message> {
        let a = v
            .dict()?
            .get(b"a".as_ref())
            .ok_or(Error::DictNotFound("a".to_string()))?
            .dict()?;

        let hash = a
            .get(b"info_hash".as_ref())
            .ok_or(Error::DictNotFound("info_hash".to_string()))?
            .bytes()?;

        // There is an optional argument called implied_port which value is either 0 or 1. If it is
        // present and non-zero, the port argument should be ignored and the source port of the UDP
        // packet should be used as the peer's port instead.
        let mut port = addr.port();
        if let Some(Value::Integer(0)) = a.get(b"implied_port".as_ref()) {
            port = a
                .get(b"port".as_ref())
                .ok_or(Error::DictNotFound("port".to_string()))?
                .integer()? as u16;
        }

        Ok(Message {
            peer: SocketAddr::new(addr.ip(), port),
            infohash: hash.to_vec(),
        })
    }

    async fn find_node<A: ToSocketAddrs>(&self, addr: A, target_id: &[u8]) -> Result<usize> {
        let a = bencode::map!(
            b"id".to_vec() => Value::from(neighbor_id(target_id, self.local_id.as_ref())),
            b"target".to_vec() => Value::from(rand_infohash_key())
        );

        let mut n = 0;
        let buf = self.make_query(&rand_transation_id(), b"find_node", a)?;
        if let Some(socket) = &*self.socket {
            n = socket.send_to(&buf, addr).await?;
        };
        Ok(n)
    }

    fn make_query(&self, tid: &[u8], qr: &[u8], a: HashMap<Vec<u8>, Value>) -> Result<Vec<u8>> {
        let m = bencode::map!(
            b"t".to_vec() => Value::from(tid),
            b"y".to_vec() => Value::from(b"q".as_ref()),
            b"q".to_vec() => Value::from(qr),
            b"a".to_vec() => Value::from(a)
        );
        bencode::to_bytes(&Value::from(m)).map_err(crate::Error::from)
    }

    fn make_reply(&self, tid: &[u8], r: HashMap<Vec<u8>, Value>) -> Result<Vec<u8>> {
        let m = bencode::map!(
            b"t".to_vec() => Value::from(tid),
            b"y".to_vec() => Value::from(b"r".as_ref()),
            b"r".to_vec() => Value::from(r)
        );
        bencode::to_bytes(&Value::from(m)).map_err(crate::Error::from)
    }

    fn make_token(&self, addr: &SocketAddr) -> Vec<u8> {
        let mut m = sha1::Sha1::new();
        m.update(addr.to_string().as_bytes());
        m.update(&self.secret);
        m.digest().bytes().to_vec()
    }

    fn is_valid_token(&self, token: &[u8], addr: &SocketAddr) -> bool {
        token == self.make_token(addr).as_slice()
    }
}
