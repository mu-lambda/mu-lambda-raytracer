pub trait Rngator: Sync {
    fn rng(&self) -> Box<dyn rand::RngCore>;
}

pub struct ThreadRngator {}

impl Rngator for ThreadRngator {
    fn rng(&self) -> Box<dyn rand::RngCore> {
        Box::new(rand::thread_rng())
    }
}
