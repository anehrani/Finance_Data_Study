use std::cell::RefCell;
use matlib::Mwc256;

// Re-export Mwc256 as Rng to maintain compatibility if needed, 
// or just use Mwc256 directly. For now, let's alias it.
pub type Rng = Mwc256;

thread_local! {
    static RNG: RefCell<Rng> = RefCell::new(Rng::new());
}

/// Set the seed for the thread-local RNG
pub fn set_seed(seed: i32) {
    let seed = seed as u32;
    RNG.with(|rng| {
        *rng.borrow_mut() = Rng::with_seed(seed);
    });
}

/// Generate a random f64 in [0, 1) using the thread-local RNG
pub fn unifrand() -> f64 {
    RNG.with(|rng| rng.borrow_mut().unifrand())
}

/// Generate a standard normal random variable using Box-Muller method
pub fn normal() -> f64 {
    RNG.with(|rng| rng.borrow_mut().normal())
}

