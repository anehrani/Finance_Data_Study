
pub fn boot_conf_pctile<F>(
    n: usize,
    x: &[f64],
    user_t: F,
    nboot: usize,
) -> (f64, f64, f64, f64, f64, f64)
where
    F: Fn(usize, &[f64]) -> f64,
{
    let mut work2 = Vec::with_capacity(nboot);
    let mut rng = rand::thread_rng();
    use rand::Rng;

    for _ in 0..nboot {
        let mut xwork = Vec::with_capacity(n);
        for _ in 0..n {
            let k = rng.gen_range(0..n);
            xwork.push(x[k]);
        }
        work2.push(user_t(n, &xwork));
    }

    work2.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let get_percentile = |p: f64| -> f64 {
        let k = (p * (nboot as f64 + 1.0)) as isize - 1;
        let idx = k.max(0) as usize;
        if idx < nboot {
            work2[idx]
        } else {
            work2[nboot - 1]
        }
    };

    let low2p5 = get_percentile(0.025);
    let high2p5 = get_percentile(1.0 - 0.025); // nboot-1-k logic in C++ roughly maps to this
    
    let low5 = get_percentile(0.05);
    let high5 = get_percentile(1.0 - 0.05);

    let low10 = get_percentile(0.10);
    let high10 = get_percentile(1.0 - 0.10);

    (low2p5, high2p5, low5, high5, low10, high10)
}

pub fn boot_conf_bca<F>(
    n: usize,
    x: &[f64],
    user_t: F,
    nboot: usize,
) -> (f64, f64, f64, f64, f64, f64)
where
    F: Fn(usize, &[f64]) -> f64,
{
    use crate::stats::{inverse_normal_cdf, normal_cdf};
    use rand::Rng;

    let theta_hat = user_t(n, x);
    let mut z0_count = 0;
    let mut work2 = Vec::with_capacity(nboot);
    let mut rng = rand::thread_rng();

    for _ in 0..nboot {
        let mut xwork = Vec::with_capacity(n);
        for _ in 0..n {
            let k = rng.gen_range(0..n);
            xwork.push(x[k]);
        }
        let param = user_t(n, &xwork);
        work2.push(param);
        if param < theta_hat {
            z0_count += 1;
        }
    }

    if z0_count >= nboot {
        z0_count = nboot - 1;
    }
    if z0_count == 0 {
        z0_count = 1;
    }

    let z0 = inverse_normal_cdf(z0_count as f64 / nboot as f64);

    // Jackknife for accel
    let mut theta_dot = 0.0;
    let mut jack_params = Vec::with_capacity(n);
    
    // We need a mutable copy of x to simulate the swap logic, or just create new vectors
    // The C++ code swaps: x[i] = xlast; ... x[i] = xtemp; effectively removing x[i] and replacing with x[n-1]
    // Wait, C++ code:
    // xlast = x[n-1];
    // for (i=0; i<n; i++) {
    //    xtemp = x[i];
    //    x[i] = xlast; // Replace current with last
    //    param = user_t(n-1, x); // Compute on n-1 size? No, user_t takes n-1 but x is still size n?
    //    // Ah, user_t(n-1, x) uses first n-1 elements.
    //    // So if we put xlast at x[i], and use first n-1, we are effectively removing x[i] (which was at i)
    //    // and keeping x[n-1] (which is now at i).
    //    // But what about the original x[n-1]? It's at x[n-1].
    //    // So the set is {x[0]...x[i-1], x[n-1], x[i+1]...x[n-2], x[n-1]} ?
    //    // This seems like it duplicates x[n-1] if i < n-1.
    //    // And if i == n-1, x[n-1] = xlast (no change), so we just use first n-1.
    //    // This seems to be a specific way to do jackknife by replacing the dropped element with the last one, 
    //    // and then only using n-1 elements. Since order shouldn't matter for user_t (usually mean), this works.
    // }
    
    // In Rust, let's just create a vector without the i-th element.
    for i in 0..n {
        let mut subset = Vec::with_capacity(n - 1);
        for (j, val) in x.iter().enumerate() {
            if i != j {
                subset.push(*val);
            }
        }
        let param = user_t(n - 1, &subset);
        theta_dot += param;
        jack_params.push(param);
    }

    theta_dot /= n as f64;
    let mut numer = 0.0;
    let mut denom = 0.0;

    for val in &jack_params {
        let diff = theta_dot - val;
        let xtemp = diff * diff;
        denom += xtemp;
        numer += xtemp * diff;
    }

    denom = denom.sqrt();
    denom = denom * denom * denom;
    let accel = numer / (6.0 * denom + 1.0e-60);

    work2.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let calc_limits = |alpha: f64| -> (f64, f64) {
        let zlo = inverse_normal_cdf(alpha);
        let zhi = inverse_normal_cdf(1.0 - alpha);
        
        let alo = normal_cdf(z0 + (z0 + zlo) / (1.0 - accel * (z0 + zlo)));
        let ahi = normal_cdf(z0 + (z0 + zhi) / (1.0 - accel * (z0 + zhi)));
        
        let k_lo = (alo * (nboot as f64 + 1.0)) as isize - 1;
        let idx_lo = k_lo.max(0) as usize;
        let low = if idx_lo < nboot { work2[idx_lo] } else { work2[nboot - 1] };

        let k_hi = ((1.0 - ahi) * (nboot as f64 + 1.0)) as isize - 1;
        let idx_hi = k_hi.max(0) as usize;
        let high = if idx_hi < nboot { work2[nboot - 1 - idx_hi] } else { work2[0] }; // C++: work2[nboot-1-k]

        (low, high)
    };

    let (low2p5, high2p5) = calc_limits(0.025);
    let (low5, high5) = calc_limits(0.05);
    let (low10, high10) = calc_limits(0.10);

    (low2p5, high2p5, low5, high5, low10, high10)
}
