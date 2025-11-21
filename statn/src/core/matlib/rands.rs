use std::f64::consts::PI;

/// Generate a uniform random number in [0, 1)
pub fn unifrand() -> f64 {
    rand::random::<f64>()
}

/// Generate a standard normal random variable using Box-Muller method
pub fn normal() -> f64 {
    loop {
        let x1 = unifrand();
        if x1 > 0.0 {
            let x1 = (-2.0 * x1.ln()).sqrt();
            let x2 = (2.0 * PI * unifrand()).cos();
            return x1 * x2;
        }
    }
}

/// Generate a pair of standard normal random variables using Box-Muller method
pub fn normal_pair() -> (f64, f64) {
    loop {
        let u1 = unifrand();
        if u1 > 0.0 {
            let u1 = (-2.0 * u1.ln()).sqrt();
            let u2 = 2.0 * PI * unifrand();
            return (u1 * u2.sin(), u1 * u2.cos());
        }
    }
}

/// Generate a Gamma random variable with parameter v/2
pub fn gamma(v: i32) -> f64 {
    match v {
        1 => {
            // Chi-square with 1 df is 2 * gamma(0.5)
            let x = normal();
            0.5 * x * x
        }
        2 => {
            // Gamma(1) is exponential(1)
            loop {
                let x = unifrand();
                if x > 0.0 {
                    return -x.ln();
                }
            }
        }
        _ => {
            // Valid for all real a > 1 (a = v/2)
            let vm1 = 0.5 * v as f64 - 1.0;
            let root = (v as f64 - 1.0).sqrt();

            loop {
                let y = (PI * unifrand()).tan();
                let x = root * y + vm1;
                if x > 0.0 {
                    let z = (1.0 + y * y) * (vm1 * (x / vm1).ln() - root * y).exp();
                    if unifrand() <= z {
                        return x;
                    }
                }
            }
        }
    }
}

/// Generate a Beta random variable with parameters v1/2 and v2/2
pub fn beta(v1: i32, v2: i32) -> f64 {
    let x1 = gamma(v1);
    let x2 = gamma(v2);
    x1 / (x1 + x2)
}

/// Generate a random point uniformly distributed on an n-sphere surface
pub fn rand_sphere(nvars: usize) -> Vec<f64> {
    let mut x = vec![0.0; nvars];
    let mut length = 0.0;

    // Efficiently generate pairs
    for i in 0..(nvars / 2) {
        let (x1, x2) = normal_pair();
        x[2 * i] = x1;
        x[2 * i + 1] = x2;
        length += x1 * x1 + x2 * x2;
    }

    // If odd, get the last one
    if nvars % 2 == 1 {
        let x_last = normal();
        x[nvars - 1] = x_last;
        length += x_last * x_last;
    }

    let length = 1.0 / length.sqrt();
    for val in x.iter_mut() {
        *val *= length;
    }

    x
}

/// Generate a random vector following the n-variate Cauchy density with specified scale
pub fn cauchy(n: usize, scale: f64) -> Vec<f64> {
    if n == 1 {
        let temp = PI * unifrand() - 0.5 * PI;
        return vec![scale * (0.99999999 * temp).tan()];
    }

    let mut x = rand_sphere(n);
    let temp = beta(n as i32, 1);

    let scale_factor = if temp < 1.0 {
        scale * (temp / (1.0 - temp)).sqrt()
    } else {
        1.0e10
    };

    for val in x.iter_mut() {
        *val *= scale_factor;
    }

    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unifrand() {
        for _ in 0..1000 {
            let u = unifrand();
            assert!(u >= 0.0 && u < 1.0);
        }
    }

    #[test]
    fn test_normal() {
        for _ in 0..1000 {
            let n = normal();
            assert!(n.is_finite());
        }
    }

    #[test]
    fn test_normal_pair() {
        for _ in 0..1000 {
            let (x1, x2) = normal_pair();
            assert!(x1.is_finite());
            assert!(x2.is_finite());
        }
    }

    #[test]
    fn test_gamma() {
        for v in &[1, 2, 3, 4, 5] {
            for _ in 0..100 {
                let g = gamma(*v);
                assert!(g > 0.0);
            }
        }
    }

    #[test]
    fn test_beta() {
        for _ in 0..1000 {
            let b = beta(2, 3);
            assert!(b >= 0.0 && b <= 1.0);
        }
    }

    #[test]
    fn test_rand_sphere() {
        for nvars in &[2, 3, 5, 10] {
            let x = rand_sphere(*nvars);
            assert_eq!(x.len(), *nvars);

            // Check it's on unit sphere
            let norm_sq: f64 = x.iter().map(|v| v * v).sum();
            assert!((norm_sq - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_cauchy() {
        for n in &[1, 2, 3, 5] {
            let x = cauchy(*n, 1.0);
            assert_eq!(x.len(), *n);
            for val in x.iter() {
                assert!(val.is_finite());
            }
        }
    }
}