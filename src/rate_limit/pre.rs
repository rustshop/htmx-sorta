//! A (hopefully - benchmarks missing) fast, but imprecise
//! rate limiter.
//! It hashes every IP as an index to multiple buckets to
//! avoid any locking.
//!
//! It might be actually stupid. I just had an idea
//! and ran with it.

use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::rate_limit::xor_hash;

struct FastPreRateLimiterInner {
    threshold: usize,
    buckets: Vec<AtomicU8>,
}

impl FastPreRateLimiterInner {
    pub fn rate_limit(&self, peer_ip: std::net::IpAddr) -> bool {
        let mut hasher = xor_hash::XorHasher::default();
        peer_ip.hash(&mut hasher);
        let hash = hasher.finish();
        let mut count = 0usize;
        let mut threshold = 0usize;
        for bucket_num in 0..Self::BUCKET_NUM {
            // each ip will rotate differently around buckets
            let bucket_num_offset = (hash >> (64 - Self::BUCKET_NUM_BITS)) as usize;
            let bucket_num = (bucket_num ^ bucket_num_offset) % Self::BUCKET_NUM;
            let bucket_idx = (hash >> (bucket_num * Self::BUCKET_SIZE_BITS)) as u8 as usize;
            debug_assert!(bucket_idx <= u8::MAX as usize);
            let bucket_array_offset = bucket_num * Self::BUCKET_SIZE + bucket_idx;
            count += self.buckets[bucket_array_offset].load(Ordering::Relaxed) as usize;
            threshold += self.threshold / Self::BUCKET_NUM + 1;
            if count < threshold {
                // only one write, in a (not really) random position of a random bucket; should
                // help with cacheline exclusive access sharing between cpus
                self.buckets[bucket_array_offset].fetch_add(1, Ordering::Relaxed);
                return false;
            }
        }
        true
    }
}

impl FastPreRateLimiterInner {
    const BUCKET_SIZE_BITS: usize = 8;
    const BUCKET_SIZE: usize = 1 << Self::BUCKET_SIZE_BITS;
    const BUCKET_NUM_BITS: usize = 2;
    const BUCKET_NUM: usize = 1 << Self::BUCKET_NUM_BITS;

    fn new(threshold: usize) -> Self {
        Self {
            threshold,
            buckets: (0..Self::BUCKET_SIZE * Self::BUCKET_NUM)
                .map(|_| Default::default())
                .collect(),
        }
    }
    fn tick(&self, tick_i: usize) {
        let bucket = tick_i % Self::BUCKET_NUM;
        for i in 0..Self::BUCKET_SIZE {
            self.buckets[bucket * Self::BUCKET_SIZE + i].store(0, Ordering::Relaxed);
        }
    }
}

#[derive(Clone)]
pub struct FastPreRateLimiter {
    inner: Arc<FastPreRateLimiterInner>,
}

impl FastPreRateLimiter {
    pub fn new(threshold: usize, window_secs: u64) -> Self {
        let s = Self {
            inner: Arc::new(FastPreRateLimiterInner::new(threshold)),
        };

        s.start_timer_thread(window_secs);

        s
    }

    pub fn rate_limit(&self, peer_ip: IpAddr) -> bool {
        self.inner.rate_limit(peer_ip)
    }
}

impl FastPreRateLimiter {
    pub fn start_timer_thread(&self, window_secs: u64) {
        let s = Arc::downgrade(&self.inner);
        let tick = (window_secs / FastPreRateLimiterInner::BUCKET_NUM as u64) + 1;
        let mut tick_i = 0;
        std::thread::spawn(move || {
            while let Some(s) = s.upgrade() {
                std::thread::sleep(Duration::from_secs(tick));
                s.tick(tick_i);
                tick_i += 1;
            }
        });
    }
}
