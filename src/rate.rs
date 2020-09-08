use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

// 1 second.
const SECOND: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct Rate {
    limit: AtomicUsize,
    num: AtomicUsize,
    last: Mutex<Instant>,
}

impl Rate {
    pub fn new(n: usize) -> Self {
        Self {
            limit: AtomicUsize::new(n),
            num: AtomicUsize::new(0),
            last: Mutex::new(Instant::now()),
        }
    }

    pub fn allow(&self) -> bool {
        let mut last = self.last.lock().unwrap();

        if self.num.load(Ordering::Relaxed) >= self.limit.load(Ordering::Relaxed) {
            let now = Instant::now();

            if now.duration_since(*last) < SECOND {
                return false;
            }

            self.num.store(0, Ordering::Relaxed);
            *last = now;
        }

        self.num.fetch_add(1, Ordering::Relaxed);
        true
    }
}
