use rand::SeedableRng;
use std::sync::atomic::AtomicU64;

pub trait Rngator: Sync {
    fn rng(&self) -> Box<dyn rand::RngCore>;
}

pub struct ThreadRngator {}

impl Rngator for ThreadRngator {
    fn rng(&self) -> Box<dyn rand::RngCore> {
        Box::new(rand::thread_rng())
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
    fn rng(&self) -> Box<dyn rand::RngCore> {
        let seed = self.seed.fetch_add(1, std::sync::atomic::Ordering::Release);
        Box::new(rand_pcg::Pcg64::seed_from_u64(seed))
    }
}
