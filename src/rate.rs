use std::time::{Duration, Instant};

// 1 second.
const SECOND: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct Rate {
    limit: usize,
    num: usize,
    last: Instant,
}

impl Rate {
    pub fn new(n: usize) -> Self {
        Self {
            limit: n,
            num: 0,
            last: Instant::now(),
        }
    }

    pub fn allow(&mut self) -> bool {
        if self.num >= self.limit {
            let now = Instant::now();

            if now.duration_since(self.last) < SECOND {
                return false;
            }

            self.num = 0;
            self.last = now;
        }

        self.num += 1;
        true
    }
}
