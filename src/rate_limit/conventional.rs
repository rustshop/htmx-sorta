use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

struct RateLimiterInner {
    threshold: usize,
    buckets: [HashMap<IpAddr, AtomicU16>; 2],
    curr_bucket: u8,
}

impl RateLimiterInner {
    pub(crate) fn new(threshold: usize) -> Self {
        Self {
            threshold,
            buckets: [HashMap::new(), HashMap::new()],
            curr_bucket: 0,
        }
    }

    pub(crate) fn tick(&mut self) {
        self.curr_bucket = (self.curr_bucket + 1) % 2;

        self.buckets[self.curr_bucket as usize].clear();
    }
}

#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<RwLock<RateLimiterInner>>,
}

impl RateLimiter {
    pub fn new(threshold: usize, window_secs: u64) -> Self {
        let s = Self {
            inner: Arc::new(RwLock::new(RateLimiterInner::new(threshold))),
        };

        s.start_timer_thread(window_secs);

        s
    }

    pub fn rate_limit(&self, peer_ip: std::net::IpAddr) -> bool {
        loop {
            let read = self.inner.read().expect("locking failed");

            if let Some(entry) = read.buckets[read.curr_bucket as usize].get(&peer_ip) {
                let curr = entry.load(Ordering::Relaxed) as usize;
                let prev = read.buckets[(read.curr_bucket as usize + 1) % 2]
                    .get(&peer_ip)
                    .map(|entry| entry.load(Ordering::Relaxed))
                    .unwrap_or(0) as usize;

                if curr + prev < read.threshold {
                    entry.fetch_add(1, Ordering::Relaxed);
                    return false;
                }

                return true;
            }

            drop(read);

            // slow path: insert the entry and try again
            let mut write = self.inner.write().expect("locking failed");
            let curr_bucket = write.curr_bucket;
            write.buckets[curr_bucket as usize]
                .entry(peer_ip)
                .or_default();
        }
    }
}

impl RateLimiter {
    pub fn start_timer_thread(&self, window_secs: u64) {
        let s = Arc::downgrade(&self.inner);
        let tick = (window_secs / 2) + 1;
        std::thread::spawn(move || {
            while let Some(s) = s.upgrade() {
                std::thread::sleep(Duration::from_secs(tick));
                s.write().expect("locking failed").tick();
            }
        });
    }
}
