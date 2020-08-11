use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;

use lru_cache::LruCache;

const EXPIRE_DURATION: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Debug)]
pub struct BlackList {
    cache: Arc<Mutex<LruCache<SocketAddr, Instant>>>,
}

impl BlackList {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
        }
    }

    pub fn contains(&self, addr: &SocketAddr) -> bool {
        self.scan_expired();

        let mut cache = self.cache.lock().unwrap();
        cache.contains_key(addr)
    }

    pub fn insert(&mut self, addr: SocketAddr) {
        self.scan_expired();

        let mut cache = self.cache.lock().unwrap();
        cache.insert(addr, Instant::now());
    }

    fn scan_expired(&self) {
        let mut count = 0;
        let now = Instant::now();
        let mut cache = self.cache.lock().unwrap();

        for (_, v) in cache.iter() {
            if now.duration_since(*v) > EXPIRE_DURATION {
                count += 1;
            } else {
                break;
            }
        }

        for _ in 0..count {
            cache.remove_lru();
        }
    }
}
