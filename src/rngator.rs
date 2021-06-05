use rand::SeedableRng;

pub trait Rngator: Sync {
    type R: rand::RngCore;
    fn rng(&self, site_id: u64) -> Self::R;
}

pub struct ThreadRngator {}

impl Rngator for ThreadRngator {
    type R = rand::rngs::ThreadRng;
    fn rng(&self, _: u64) -> rand::rngs::ThreadRng {
        rand::thread_rng()
    }
}

pub struct SeedableRngator {
    seed: u64,
}

impl SeedableRngator {
    pub fn new(seed: u64) -> SeedableRngator {
        SeedableRngator { seed }
    }
}

impl Rngator for SeedableRngator {
    type R = rand_pcg::Pcg64;
    fn rng(&self, site_id: u64) -> rand_pcg::Pcg64 {
        rand_pcg::Pcg64::seed_from_u64(self.seed + site_id)
    }
}
