use rand::SeedableRng;
use std::sync::atomic::AtomicU64;

pub trait Rngator: Sync {
    type R: rand::RngCore;
    fn rng(&self) -> Self::R;
}

pub struct ThreadRngator {}

impl Rngator for ThreadRngator {
    type R = rand::rngs::ThreadRng;
    fn rng(&self) -> rand::rngs::ThreadRng {
        rand::thread_rng()
    }
}

pub struct SeedableRngator {
    seed: AtomicU64,
}

impl SeedableRngator {
    pub fn new(seed: u64) -> SeedableRngator {
        SeedableRngator { seed: AtomicU64::new(seed) }
    }
}

impl Rngator for SeedableRngator {
    type R = rand_pcg::Pcg64;
    fn rng(&self) -> rand_pcg::Pcg64 {
        let seed = self.seed.fetch_add(1, std::sync::atomic::Ordering::Release);
        rand_pcg::Pcg64::seed_from_u64(seed)
    }
}
