use rand::Rng;

#[allow(dead_code)]
pub fn unifrand() -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen()
}
