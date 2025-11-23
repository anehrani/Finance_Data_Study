use std::env;
use std::io::{self, Read};

// External functions already implemented in Rust
fn rand32m_seed(iseed: i32);
fn unifrand() -> f64;
fn normal() -> f64;
fn qsortd(istart: usize, istop: usize, x: &mut [f64]);
use stats::{normal_cdf, inverse_normal_cdf};

// Bootstrap confidence interval functions from previous conversion
fn boot_conf_pctile<F>(
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

    for rep in 0..nboot {
        for i in 0..n {
            let k = ((unifrand() * n as f64) as usize).min(n - 1);
            xwork[i] = x[k];
        }
        work2[rep] = user_t(&xwork);
    }

    qsortd(0, nboot - 1, &mut work2);

    let mut k = ((0.025 * (nboot + 1) as f64) as i32) - 1;
    if k < 0 {
        k = 0;
    }
    let low2p5 = work2[k as usize];
    let high2p5 = work2[nboot - 1 - k as usize];

    k = ((0.05 * (nboot + 1) as f64) as i32) - 1;
    if k < 0 {
        k = 0;
    }
    let low5 = work2[k as usize];
    let high5 = work2[nboot - 1 - k as usize];

    k = ((0.10 * (nboot + 1) as f64) as i32) - 1;
    if k < 0 {
        k = 0;
    }
    let low10 = work2[k as usize];
    let high10 = work2[nboot - 1 - k as usize];

    (low2p5, high2p5, low5, high5, low10, high10)
}

fn boot_conf_bca<F>(
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

    let theta_hat = user_t(x);
    let mut z0_count = 0;

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

    if z0_count >= nboot {
        z0_count = nboot - 1;
    }
    if z0_count <= 0 {
        z0_count = 1;
    }

    let z0 = inverse_normal_cdf(z0_count as f64 / nboot as f64);

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

    qsortd(0, nboot - 1, &mut work2);

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

    let (low2p5, high2p5) = compute_bounds(0.025);
    let (low5, high5) = compute_bounds(0.05);
    let (low10, high10) = compute_bounds(0.10);

    (low2p5, high2p5, low5, high5, low10, high10)
}

static mut USE_LOG: bool = true;

/// Compute the profit factor
fn param_pf(x: &[f64]) -> f64 {
    let mut numer = 1.0e-10;
    let mut denom = 1.0e-10;

    for &val in x {
        if val > 0.0 {
            numer += val;
        } else {
            denom -= val;
        }
    }

    let val = numer / denom;
    unsafe { if USE_LOG { val.ln() } else { val } }
}

/// Compute the Sharpe ratio (not normalized in any way)
fn param_sr(x: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mut numer = 0.0;

    for &val in x {
        numer += val;
    }
    numer /= n;

    let mut denom = 0.0;
    for &val in x {
        let diff = val - numer;
        denom += diff * diff;
    }
    denom = (denom / n).sqrt();

    if denom > 0.0 {
        numer / denom
    } else {
        1.0e30
    }
}

/// Count how many times a condition is true in an array
fn count_violations(
    bounds: &[(f64, f64)],
    true_val: f64,
    check_low: bool,
) -> usize {
    bounds
        .iter()
        .filter(|(low, high)| {
            if check_low {
                *low > true_val
            } else {
                *high < true_val
            }
        })
        .count()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let (nsamps, nboot, ntries, prob) = if args.len() == 5 {
        match (
            args[1].parse::<usize>(),
            args[2].parse::<usize>(),
            args[3].parse::<usize>(),
            args[4].parse::<f64>(),
        ) {
            (Ok(n), Ok(b), Ok(t), Ok(p)) => (n, b, t, p),
            _ => {
                eprintln!("Error parsing command line arguments");
                print_usage();
                return;
            }
        }
    } else {
        print_usage();
        return;
    };

    if nsamps == 0 || nboot == 0 || ntries == 0 || prob < 0.0 || prob >= 1.0 {
        eprintln!("Invalid parameters");
        print_usage();
        return;
    }

    let mut true_pf = prob / (1.0 - prob);
    unsafe {
        if USE_LOG {
            true_pf = true_pf.ln();
        }
    }

    let divisor = std::cmp::max(2, 10_000_000 / (nsamps * nboot));

    let mut x = vec![0.0; nsamps];
    let mut param = vec![0.0; ntries];

    let mut low2p5_1 = vec![0.0; ntries];
    let mut high2p5_1 = vec![0.0; ntries];
    let mut low5_1 = vec![0.0; ntries];
    let mut high5_1 = vec![0.0; ntries];
    let mut low10_1 = vec![0.0; ntries];
    let mut high10_1 = vec![0.0; ntries];

    let mut low2p5_2 = vec![0.0; ntries];
    let mut high2p5_2 = vec![0.0; ntries];
    let mut low5_2 = vec![0.0; ntries];
    let mut high5_2 = vec![0.0; ntries];
    let mut low10_2 = vec![0.0; ntries];
    let mut high10_2 = vec![0.0; ntries];

    let mut low2p5_3 = vec![0.0; ntries];
    let mut high2p5_3 = vec![0.0; ntries];
    let mut low5_3 = vec![0.0; ntries];
    let mut high5_3 = vec![0.0; ntries];
    let mut low10_3 = vec![0.0; ntries];
    let mut high10_3 = vec![0.0; ntries];

    let mut true_sum = 0.0;
    let mut true_sumsq = 0.0;

    // Profit factor trials
    println!("\n=== PROFIT FACTOR ANALYSIS ===");
    for itry in 0..ntries {
        if itry % divisor == 0 {
            println!("\n\nTry {}", itry);
        }

        rand32m_seed((itry + (itry << 16)) as i32);

        for i in 0..nsamps {
            x[i] = 0.01 + 0.002 * normal();
            if unifrand() > prob {
                x[i] = -x[i];
            }
            true_sum += x[i];
            true_sumsq += x[i] * x[i];
        }

        param[itry] = param_pf(&x);

        let (l2p5, h2p5, l5, h5, l10, h10) = boot_conf_pctile(&x, param_pf, nboot);
        low2p5_1[itry] = l2p5;
        high2p5_1[itry] = h2p5;
        low5_1[itry] = l5;
        high5_1[itry] = h5;
        low10_1[itry] = l10;
        high10_1[itry] = h10;

        let (l2p5, h2p5, l5, h5, l10, h10) = boot_conf_bca(&x, param_pf, nboot);
        low2p5_2[itry] = l2p5;
        high2p5_2[itry] = h2p5;
        low5_2[itry] = l5;
        high5_2[itry] = h5;
        low10_2[itry] = l10;
        high10_2[itry] = h10;

        low2p5_3[itry] = 2.0 * param[itry] - high2p5_1[itry];
        high2p5_3[itry] = 2.0 * param[itry] - low2p5_1[itry];
        low5_3[itry] = 2.0 * param[itry] - high5_1[itry];
        high5_3[itry] = 2.0 * param[itry] - low5_1[itry];
        low10_3[itry] = 2.0 * param[itry] - high10_1[itry];
        high10_3[itry] = 2.0 * param[itry] - low10_1[itry];

        if (itry % divisor == 1) || (itry == ntries - 1) {
            let ndone = itry + 1;
            let mean_param: f64 = param[..ndone].iter().sum::<f64>() / ndone as f64;

            unsafe {
                if USE_LOG {
                    println!("Mean log pf = {:.5} true = {:.5}", mean_param, true_pf);
                } else {
                    println!("Mean pf = {:.5} true = {:.5}", mean_param, true_pf);
                }
            }

            print_results(&low2p5_1[..ndone], &high2p5_1[..ndone],
                         &low5_1[..ndone], &high5_1[..ndone],
                         &low10_1[..ndone], &high10_1[..ndone],
                         true_pf, "Pctile");

            print_results(&low2p5_2[..ndone], &high2p5_2[..ndone],
                         &low5_2[..ndone], &high5_2[..ndone],
                         &low10_2[..ndone], &high10_2[..ndone],
                         true_pf, "BCa");

            print_results(&low2p5_3[..ndone], &high2p5_3[..ndone],
                         &low5_3[..ndone], &high5_3[..ndone],
                         &low10_3[..ndone], &high10_3[..ndone],
                         true_pf, "Pivot");
        }

        if (itry % 10) == 1 {
            if check_for_escape() {
                break;
            }
        }
    }

    // Sharpe ratio calculation
    true_sum /= (ntries * nsamps) as f64;
    true_sumsq /= (ntries * nsamps) as f64;
    true_sumsq = (true_sumsq - true_sum * true_sum).sqrt();
    let true_sr = true_sum / true_sumsq;

    println!("\n\n=== SHARPE RATIO ANALYSIS ===");
    for itry in 0..ntries {
        if itry % divisor == 0 {
            println!("\n\nTry {}", itry);
        }

        rand32m_seed((itry + (itry << 16)) as i32);

        for i in 0..nsamps {
            x[i] = 0.01 + 0.002 * normal();
            if unifrand() > prob {
                x[i] = -x[i];
            }
        }

        param[itry] = param_sr(&x);

        let (l2p5, h2p5, l5, h5, l10, h10) = boot_conf_pctile(&x, param_sr, nboot);
        low2p5_1[itry] = l2p5;
        high2p5_1[itry] = h2p5;
        low5_1[itry] = l5;
        high5_1[itry] = h5;
        low10_1[itry] = l10;
        high10_1[itry] = h10;

        let (l2p5, h2p5, l5, h5, l10, h10) = boot_conf_bca(&x, param_sr, nboot);
        low2p5_2[itry] = l2p5;
        high2p5_2[itry] = h2p5;
        low5_2[itry] = l5;
        high5_2[itry] = h5;
        low10_2[itry] = l10;
        high10_2[itry] = h10;

        low2p5_3[itry] = 2.0 * param[itry] - high2p5_1[itry];
        high2p5_3[itry] = 2.0 * param[itry] - low2p5_1[itry];
        low5_3[itry] = 2.0 * param[itry] - high5_1[itry];
        high5_3[itry] = 2.0 * param[itry] - low5_1[itry];
        low10_3[itry] = 2.0 * param[itry] - high10_1[itry];
        high10_3[itry] = 2.0 * param[itry] - low10_1[itry];

        if (itry % divisor == 1) || (itry == ntries - 1) {
            if itry == ntries - 1 {
                println!("\n\nFinal Sharpe ratio...");
            }
            let ndone = itry + 1;
            let mean_param: f64 = param[..ndone].iter().sum::<f64>() / ndone as f64;

            println!("Mean sr = {:.5}  true = {:.5}", mean_param, true_sr);

            print_results(&low2p5_1[..ndone], &high2p5_1[..ndone],
                         &low5_1[..ndone], &high5_1[..ndone],
                         &low10_1[..ndone], &high10_1[..ndone],
                         true_sr, "Pctile");

            print_results(&low2p5_2[..ndone], &high2p5_2[..ndone],
                         &low5_2[..ndone], &high5_2[..ndone],
                         &low10_2[..ndone], &high10_2[..ndone],
                         true_sr, "BCa");

            print_results(&low2p5_3[..ndone], &high2p5_3[..ndone],
                         &low5_3[..ndone], &high5_3[..ndone],
                         &low10_3[..ndone], &high10_3[..ndone],
                         true_sr, "Pivot");
        }

        if (itry % 10) == 1 {
            if check_for_escape() {
                break;
            }
        }
    }

    println!("\n\nnsamps={}  nboot={}  ntries={}  prob={:.3}", nsamps, nboot, ntries, prob);
    println!("Press any key to exit...");
    let _ = io::stdin().read(&mut [0; 1]);
}

fn print_usage() {
    eprintln!("Usage: boot_ratio nsamples nboot ntries prob");
    eprintln!("  nsamples - Number of price changes in market history");
    eprintln!("  nboot - Number of bootstrap replications");
    eprintln!("  ntries - Number of trials for generating summary");
    eprintln!("  prob - Probability that a trade will be a win");
}

fn print_results(
    low2p5: &[f64],
    high2p5: &[f64],
    low5: &[f64],
    high5: &[f64],
    low10: &[f64],
    high10: &[f64],
    true_val: f64,
    method: &str,
) {
    let n = low2p5.len() as f64;
    let low2p5_count = low2p5.iter().filter(|&&v| v > true_val).count() as f64;
    let high2p5_count = high2p5.iter().filter(|&&v| v < true_val).count() as f64;
    let low5_count = low5.iter().filter(|&&v| v > true_val).count() as f64;
    let high5_count = high5.iter().filter(|&&v| v < true_val).count() as f64;
    let low10_count = low10.iter().filter(|&&v| v > true_val).count() as f64;
    let high10_count = high10.iter().filter(|&&v| v < true_val).count() as f64;

    println!(
        "{:6} 2.5: ({:5.2} {:5.2})  5: ({:5.2} {:5.2})  10: ({:5.2} {:5.2})",
        method,
        100.0 * low2p5_count / n,
        100.0 * high2p5_count / n,
        100.0 * low5_count / n,
        100.0 * high5_count / n,
        100.0 * low10_count / n,
        100.0 * high10_count / n
    );
}

fn check_for_escape() -> bool {
    // In a real terminal application, you'd need to implement non-blocking input
    // For now, this is a placeholder that returns false
    false
}