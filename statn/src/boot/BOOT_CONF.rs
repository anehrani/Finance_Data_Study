// External functions already implemented in Rust
fn unifrand() -> f64;
fn qsortd(first: usize, last: usize, data: &mut [f64]);
use stats::{normal_cdf, inverse_normal_cdf};

/// Compute confidence intervals using percentile method
pub fn boot_conf_pctile<F>(
    x: &[f64],
    user_t: F,
    nboot: usize,
) -> (f64, f64, f64, f64, f64, f64)
where
    F: Fn(&[f64]) -> f64,
{
    let n = x.len();
    let mut xwork = vec![0.0; n];
    let mut work2 = vec![0.0; nboot];

    // Generate bootstrap replications
    for rep in 0..nboot {
        for i in 0..n {
            let k = ((unifrand() * n as f64) as usize).min(n - 1);
            xwork[i] = x[k];
        }
        work2[rep] = user_t(&xwork);
    }

    // Sort in ascending order
    qsortd(0, nboot - 1, &mut work2);

    // Compute 2.5% bounds
    let mut k = ((0.025 * (nboot + 1) as f64) as i32) - 1;
    if k < 0 {
        k = 0;
    }
    let low2p5 = work2[k as usize];
    let high2p5 = work2[nboot - 1 - k as usize];

    // Compute 5% bounds
    k = ((0.05 * (nboot + 1) as f64) as i32) - 1;
    if k < 0 {
        k = 0;
    }
    let low5 = work2[k as usize];
    let high5 = work2[nboot - 1 - k as usize];

    // Compute 10% bounds
    k = ((0.10 * (nboot + 1) as f64) as i32) - 1;
    if k < 0 {
        k = 0;
    }
    let low10 = work2[k as usize];
    let high10 = work2[nboot - 1 - k as usize];

    (low2p5, high2p5, low5, high5, low10, high10)
}

/// Compute confidence intervals using BCa (bias-corrected and accelerated) method
pub fn boot_conf_bca<F>(
    x: &[f64],
    user_t: F,
    nboot: usize,
) -> (f64, f64, f64, f64, f64, f64)
where
    F: Fn(&[f64]) -> f64,
{
    let n = x.len();
    let mut xwork = vec![0.0; n];
    let mut work2 = vec![0.0; nboot];
    let mut x_modified = x.to_vec();

    // Parameter for full set
    let theta_hat = user_t(x);
    let mut z0_count = 0;

    // Generate bootstrap replications
    for rep in 0..nboot {
        for i in 0..n {
            let k = ((unifrand() * n as f64) as usize).min(n - 1);
            xwork[i] = x[k];
        }
        let param = user_t(&xwork);
        work2[rep] = param;
        if param < theta_hat {
            z0_count += 1;
        }
    }

    // Prevent edge cases
    if z0_count >= nboot {
        z0_count = nboot - 1;
    }
    if z0_count <= 0 {
        z0_count = 1;
    }

    let z0 = inverse_normal_cdf(z0_count as f64 / nboot as f64);

    // Jackknife for computing acceleration
    let xlast = x[n - 1];
    let mut theta_dot = 0.0;

    for i in 0..n {
        let xtemp = x_modified[i];
        x_modified[i] = xlast;
        let param = user_t(&x_modified[..n]);
        theta_dot += param;
        xwork[i] = param;
        x_modified[i] = xtemp;
    }

    // Compute acceleration factor
    theta_dot /= n as f64;
    let mut numer = 0.0;
    let mut denom = 0.0;

    for i in 0..n {
        let diff = theta_dot - xwork[i];
        let xtemp = diff * diff;
        denom += xtemp;
        numer += xtemp * diff;
    }

    denom = denom.sqrt();
    denom = denom * denom * denom;
    let accel = numer / (6.0 * denom + 1.0e-60);

    // Sort bootstrap replications
    qsortd(0, nboot - 1, &mut work2);

    // Helper function to compute bounds for a given confidence level
    let compute_bounds = |alpha: f64| -> (f64, f64) {
        let zlo = inverse_normal_cdf(alpha);
        let zhi = inverse_normal_cdf(1.0 - alpha);
        let alo = normal_cdf(z0 + (z0 + zlo) / (1.0 - accel * (z0 + zlo)));
        let ahi = normal_cdf(z0 + (z0 + zhi) / (1.0 - accel * (z0 + zhi)));

        let mut k = ((alo * (nboot + 1) as f64) as i32) - 1;
        if k < 0 {
            k = 0;
        }
        let low = work2[k as usize];

        k = (((1.0 - ahi) * (nboot + 1) as f64) as i32) - 1;
        if k < 0 {
            k = 0;
        }
        let high = work2[nboot - 1 - k as usize];

        (low, high)
    };

    // Compute 2.5%, 5%, and 10% bounds
    let (low2p5, high2p5) = compute_bounds(0.025);
    let (low5, high5) = compute_bounds(0.05);
    let (low10, high10) = compute_bounds(0.10);

    (low2p5, high2p5, low5, high5, low10, high10)
}