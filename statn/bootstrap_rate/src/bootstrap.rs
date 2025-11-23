use rand::Rng;
use stats::{inverse_normal_cdf, normal_cdf};

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
    let mut rng = rand::thread_rng();
    let mut work2 = Vec::with_capacity(nboot);
    let mut xwork = vec![0.0; n];

    for _ in 0..nboot {
        for i in 0..n {
            let k = rng.gen_range(0..n);
            xwork[i] = x[k];
        }
        work2.push(user_t(&xwork));
    }

    work2.sort_by(|a, b| a.partial_cmp(b).unwrap());




                                           // 0.025 * (nboot+1) - 1.
                                           // if nboot=1000, k = 25 - 1 = 24.
                                           // high is nboot-1-24 = 999-24 = 975.
                                           // which is approx 0.975.
                                           // I'll use the exact logic from C++ to be safe.

    let get_low_high = |p: f64| -> (f64, f64) {
        let k = (p * (nboot as f64 + 1.0)) as isize - 1;
        let k = k.max(0) as usize;
        let low = work2[k];
        let high = work2[nboot - 1 - k];
        (low, high)
    };

    let (low2p5, high2p5) = get_low_high(0.025);
    let (low5, high5) = get_low_high(0.05);
    let (low10, high10) = get_low_high(0.10);

    (low2p5, high2p5, low5, high5, low10, high10)
}

/// Compute confidence intervals using BCa method
pub fn boot_conf_bca<F>(
    x: &[f64],
    user_t: F,
    nboot: usize,
) -> (f64, f64, f64, f64, f64, f64)
where
    F: Fn(&[f64]) -> f64,
{
    let n = x.len();
    let mut rng = rand::thread_rng();
    let mut work2 = Vec::with_capacity(nboot);
    let mut xwork = vec![0.0; n];

    let theta_hat = user_t(x);
    let mut z0_count = 0;

    for _ in 0..nboot {
        for i in 0..n {
            let k = rng.gen_range(0..n);
            xwork[i] = x[k];
        }
        let param = user_t(&xwork);
        work2.push(param);
        if param < theta_hat {
            z0_count += 1;
        }
    }

    if z0_count >= nboot {
        z0_count = nboot - 1;
    }
    if z0_count <= 0 {
        z0_count = 1;
    }

    let z0 = inverse_normal_cdf(z0_count as f64 / nboot as f64);

    // Jackknife for accel
    let mut theta_dot = 0.0;
    let mut jk_params = vec![0.0; n];
    // We need a mutable copy of x for jackknife as we remove one element
    // Actually C++ swaps elements.
    // Easier in Rust: create a new vector of size n-1.
    // Or just use a scratch buffer.
    // C++:
    // xlast = x[n-1]
    // for i=0..n:
    //   xtemp = x[i]
    //   x[i] = xlast
    //   param = user_t(n-1, x)
    //   x[i] = xtemp
    // This replaces the i-th element with the last element, effectively removing the i-th element (and duplicating the last one? No, it passes n-1 to user_t).
    // Ah, user_t takes (n, x).
    // So it uses the first n-1 elements.
    // If we swap x[i] with x[n-1], the first n-1 elements contain everything except x[i].
    // Yes.

    let mut x_jk = x.to_vec();
    let xlast = x_jk[n - 1];

    for i in 0..n {
        let xtemp = x_jk[i];
        x_jk[i] = xlast;
        // Calculate param on first n-1 elements
        let param = user_t(&x_jk[0..n - 1]);
        theta_dot += param;
        jk_params[i] = param;
        x_jk[i] = xtemp;
    }

    theta_dot /= n as f64;
    let mut numer = 0.0;
    let mut denom = 0.0;

    for i in 0..n {
        let diff = theta_dot - jk_params[i];
        let xtemp = diff * diff;
        denom += xtemp;
        numer += xtemp * diff;
    }

    denom = denom.sqrt();
    denom = denom * denom * denom;
    let accel = numer / (6.0 * denom + 1e-60);

    work2.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let calc_limits = |alpha: f64| -> (f64, f64) {
        let zlo = inverse_normal_cdf(alpha);
        let zhi = inverse_normal_cdf(1.0 - alpha);
        
        let alo = normal_cdf(z0 + (z0 + zlo) / (1.0 - accel * (z0 + zlo)));
        let ahi = normal_cdf(z0 + (z0 + zhi) / (1.0 - accel * (z0 + zhi)));
        
        let k_lo = (alo * (nboot as f64 + 1.0)) as isize - 1;
        let k_lo = k_lo.max(0) as usize;
        let low = work2[k_lo];
        

        // Wait, C++:
        // k = (int) ((1.0-ahi) * (nboot + 1)) - 1 ;
        // *high = work2[nboot-1-k] ;
        // If ahi is large (close to 1), (1-ahi) is small. k is small. nboot-1-k is large. Correct.
        
        let k_hi_idx = ((1.0 - ahi) * (nboot as f64 + 1.0)) as isize - 1;
        let k_hi_idx = k_hi_idx.max(0) as usize;
        let high = work2[nboot - 1 - k_hi_idx];
        
        (low, high)
    };

    let (low2p5, high2p5) = calc_limits(0.025);
    let (low5, high5) = calc_limits(0.05);
    let (low10, high10) = calc_limits(0.10);

    (low2p5, high2p5, low5, high5, low10, high10)
}
