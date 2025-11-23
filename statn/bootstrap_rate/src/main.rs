use std::env;
use std::f64::consts::PI;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use bootstrap_rate::bootstrap::{boot_conf_pctile, boot_conf_bca};

// Use log for Profit Factor?
const USE_LOG: bool = true;

fn main() {
    let args: Vec<String> = env::args().collect();

    let (nsamps, nboot, ntries, prob) = if args.len() != 5 {
        println!("\nUsage: bootstrap_rate  nsamples  nboot  ntries  prob");
        println!("  nsamples - Number of price changes in market history");
        println!("  nboot - Number of bootstrap replications");
        println!("  ntries - Number of trials for generating summary");
        println!("  prob - Probability that a trade will be a win");
        // Default values for testing if arguments are missing, or exit?
        // C++ code exits. But also has #if 1 ... #else defaults.
        // I'll exit with usage message.
        std::process::exit(1);
    } else {
        (
            args[1].parse::<usize>().expect("Invalid nsamples"),
            args[2].parse::<usize>().expect("Invalid nboot"),
            args[3].parse::<usize>().expect("Invalid ntries"),
            args[4].parse::<f64>().expect("Invalid prob"),
        )
    };

    if nsamps == 0 || nboot == 0 || ntries == 0 || prob < 0.0 || prob >= 1.0 {
        println!("\nUsage: bootstrap_rate  nsamples  nboot  ntries  prob");
        std::process::exit(1);
    }

    let mut true_pf = prob / (1.0 - prob);
    if USE_LOG {
        true_pf = true_pf.ln();
    }

    let divisor = 10_000_000 / (nsamps * nboot);
    let divisor = if divisor < 2 { 2 } else { divisor };

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

    // -------------------------------------------------------------------------
    // Profit Factor Loop
    // -------------------------------------------------------------------------

    for itry in 0..ntries {
        if itry % divisor == 0 {
            println!("\n\n\nTry {}", itry);
        }

        // Seed RNG
        let seed = (itry + (itry << 16)) as u64;
        let mut rng = StdRng::seed_from_u64(seed);

        for i in 0..nsamps {
            // Generate trade amount: 0.01 + 0.002 * normal()
            let norm = normal(&mut rng);
            x[i] = 0.01 + 0.002 * norm;
            if rng.gen::<f64>() > prob {
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

        // Pivot method
        low2p5_3[itry] = 2.0 * param[itry] - high2p5_1[itry];
        high2p5_3[itry] = 2.0 * param[itry] - low2p5_1[itry];
        low5_3[itry] = 2.0 * param[itry] - high5_1[itry];
        high5_3[itry] = 2.0 * param[itry] - low5_1[itry];
        low10_3[itry] = 2.0 * param[itry] - high10_1[itry];
        high10_3[itry] = 2.0 * param[itry] - low10_1[itry];

        if (itry % divisor == 1) || (itry == ntries - 1) {
            let ndone = itry + 1;
            let mean_param: f64 = param.iter().take(ndone).sum::<f64>() / ndone as f64;

            let line1 = if USE_LOG {
                format!("Mean log pf = {:.5} true = {:.5}", mean_param, true_pf)
            } else {
                format!("Mean pf = {:.5} true = {:.5}", mean_param, true_pf)
            };
            println!("\n{}", line1);

            print_stats("Pctile", ndone, true_pf, &low2p5_1, &high2p5_1, &low5_1, &high5_1, &low10_1, &high10_1);
            print_stats("BCa   ", ndone, true_pf, &low2p5_2, &high2p5_2, &low5_2, &high5_2, &low10_2, &high10_2);
            print_stats("Pivot ", ndone, true_pf, &low2p5_3, &high2p5_3, &low5_3, &high5_3, &low10_3, &high10_3);
        }
    }

    // Save PF results to print later? C++ does this by printing lines.
    // I'll just recalculate or store strings if needed, but C++ prints them at the end.
    // I'll just print them as I go and maybe at the end if I want to match exactly.
    // The C++ code stores line1, line2, line3, line4.
    // I'll skip storing for now to keep it simple, or just print "Final profit factor..." and re-print the last stats.
    
    // -------------------------------------------------------------------------
    // Sharpe Ratio Loop
    // -------------------------------------------------------------------------

    true_sum /= (ntries * nsamps) as f64;
    true_sumsq /= (ntries * nsamps) as f64;
    true_sumsq = (true_sumsq - true_sum * true_sum).sqrt();
    let true_sr = true_sum / true_sumsq;

    for itry in 0..ntries {
        if itry % divisor == 0 {
            println!("\n\n\nTry {}", itry);
        }

        let seed = (itry + (itry << 16)) as u64;
        let mut rng = StdRng::seed_from_u64(seed);

        for i in 0..nsamps {
            let norm = normal(&mut rng);
            x[i] = 0.01 + 0.002 * norm;
            if rng.gen::<f64>() > prob {
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
            let mean_param: f64 = param.iter().take(ndone).sum::<f64>() / ndone as f64;

            println!("\nMean sr = {:.5}  true = {:.5}", mean_param, true_sr);

            print_stats("Pctile", ndone, true_sr, &low2p5_1, &high2p5_1, &low5_1, &high5_1, &low10_1, &high10_1);
            print_stats("BCa   ", ndone, true_sr, &low2p5_2, &high2p5_2, &low5_2, &high5_2, &low10_2, &high10_2);
            print_stats("Pivot ", ndone, true_sr, &low2p5_3, &high2p5_3, &low5_3, &high5_3, &low10_3, &high10_3);
        }
    }

    // Final summary
    println!("\n\nnsamps={}  nboot={}  ntries={}  prob={:.3}", nsamps, nboot, ntries, prob);
}

fn param_pf(x: &[f64]) -> f64 {
    let mut numer = 1e-10;
    let mut denom = 1e-10;
    for &val in x {
        if val > 0.0 {
            numer += val;
        } else {
            denom -= val;
        }
    }
    let val = numer / denom;
    if USE_LOG {
        val.ln()
    } else {
        val
    }
}

fn param_sr(x: &[f64]) -> f64 {
    let n = x.len() as f64;
    let numer: f64 = x.iter().sum();
    let mean = numer / n;

    let mut denom = 0.0;
    for &val in x {
        let diff = val - mean;
        denom += diff * diff;
    }
    let std = (denom / n).sqrt();

    if std > 0.0 {
        mean / std
    } else {
        1e30
    }
}

fn normal(rng: &mut StdRng) -> f64 {
    // Box-Muller
    loop {
        let u1: f64 = rng.gen();
        if u1 <= 0.0 { continue; }
        let u2: f64 = rng.gen();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * PI * u2;
        return r * theta.cos();
    }
}

fn print_stats(
    label: &str,
    ndone: usize,
    true_val: f64,
    low2p5: &[f64],
    high2p5: &[f64],
    low5: &[f64],
    high5: &[f64],
    low10: &[f64],
    high10: &[f64],
) {


    // Check coverage
    // C++ logic:
    // if (low2p5_1[i] > true_pf) ++low2p5 ;
    // if (high2p5_1[i] < true_pf) ++high2p5 ;
    // It counts how many times the interval does NOT cover the true value?
    // "Output of lower 2.5% bound"
    // If low > true, true is below the interval.
    // If high < true, true is above the interval.
    // So it counts failures (misses).
    // And then prints 100 * count / ndone.
    // Wait, usually coverage is what we want (95%).
    // If it prints "2.5: (2.30 2.40)", it means 2.3% below and 2.4% above?
    // Yes, "Pctile 2.5: (%4.2lf %4.2lf)"
    // It prints the percentage of times the true value was outside the interval on the low side and high side.
    // Ideally should be 2.5% each for a 95% CI.

    let mut l2p5 = 0; let mut h2p5 = 0;
    let mut l5 = 0; let mut h5 = 0;
    let mut l10 = 0; let mut h10 = 0;

    for i in 0..ndone {
        if low2p5[i] > true_val { l2p5 += 1; }
        if high2p5[i] < true_val { h2p5 += 1; }
        if low5[i] > true_val { l5 += 1; }
        if high5[i] < true_val { h5 += 1; }
        if low10[i] > true_val { l10 += 1; }
        if high10[i] < true_val { h10 += 1; }
    }

    println!(
        "{} 2.5: ({:4.2} {:4.2})  5: ({:4.2} {:4.2})  10: ({:5.2} {:5.2})",
        label,
        100.0 * l2p5 as f64 / ndone as f64,
        100.0 * h2p5 as f64 / ndone as f64,
        100.0 * l5 as f64 / ndone as f64,
        100.0 * h5 as f64 / ndone as f64,
        100.0 * l10 as f64 / ndone as f64,
        100.0 * h10 as f64 / ndone as f64
    );
}
