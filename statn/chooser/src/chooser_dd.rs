use anyhow::Result;
use std::fs::File;
use std::io::Write;

use crate::criteria::{criterion, CriterionType};
use crate::drawdown::{drawdown_quantiles, find_quantile};
use crate::market_data::{align_dates, convert_to_log_prices, load_markets};
use crate::random::Rng;
use crate::sort::qsortd;

const N_CRITERIA: usize = 3;

pub fn run_chooser_dd(file_list: &str, is_n: usize, oos1_n: usize) -> Result<()> {
    if is_n < 2 || oos1_n < 1 {
        anyhow::bail!("Invalid parameters: IS_n must be >= 2 and OOS1_n must be >= 1");
    }

    // User-configurable parameters
    let bootstrap_reps = 2000;
    let quantile_reps = 10000;
    let n_trades = 252; // One year if daily prices

    // Open report file
    let mut fp_report = File::create("CHOOSER.LOG")?;
    writeln!(
        fp_report,
        "CHOOSER_DD  log with IS_n={}  OOS1_n={}",
        is_n, oos1_n
    )?;

    // Load markets
    println!("\nLoading markets...");
    let mut markets = load_markets(file_list)?;
    let n_markets = markets.len();

    // Align dates
    println!("\nAligning dates...");
    let n_cases = align_dates(&mut markets);

    writeln!(
        fp_report,
        "\n\nMerged database has {} records from date {} to {}",
        n_cases,
        markets[0].dates[0],
        markets[0].dates[n_cases - 1]
    )?;

    // Convert to log prices
    convert_to_log_prices(&mut markets);

    // Print return of each market over OOS2 period
    writeln!(
        fp_report,
        "\n\n25200 * mean return of each market in OOS2 period..."
    )?;
    let mut sum = 0.0;
    for market in &markets {
        let ret = 25200.0
            * (market.close[n_cases - 1] - market.close[is_n + oos1_n - 1])
            / (n_cases - is_n - oos1_n) as f64;
        sum += ret;
        writeln!(fp_report, "{:>15} {:9.4}", market.name, ret)?;
    }
    writeln!(fp_report, "Mean = {:9.4}", sum / n_markets as f64)?;

    // Allocate memory for OOS1 and OOS2
    let mut oos1 = vec![0.0; N_CRITERIA * n_cases];
    let mut oos2 = vec![0.0; n_cases];

    // Allocate drawdown work arrays
    let mut bootsample = vec![0.0; n_cases];
    let mut quantile_sample = vec![0.0; n_trades];
    let mut work = vec![0.0; quantile_reps];
    let mut q001 = vec![0.0; bootstrap_reps];
    let mut q01 = vec![0.0; bootstrap_reps];
    let mut q05 = vec![0.0; bootstrap_reps];
    let mut q10 = vec![0.0; bootstrap_reps];

    // Initialize
    let mut crit_count = [0usize; N_CRITERIA];

    let mut is_start = 0;
    let mut oos1_start = is_n;
    let mut oos1_end = is_n;
    let oos2_start = is_n + oos1_n;
    let mut oos2_end = is_n + oos1_n;

    // Main loop traversing market history
    println!("\n\nComputing trades...");

    loop {
        // Evaluate all performance criteria for all markets
        for icrit in 0..N_CRITERIA {
            let crit_type = CriterionType::from_index(icrit).unwrap();
            let mut best_crit = -1.0e60;
            let mut ibest = 0;

            for (imarket, market) in markets.iter().enumerate() {
                let crit = criterion(crit_type, &market.close[is_start..is_start + is_n]);
                if crit > best_crit {
                    best_crit = crit;
                    ibest = imarket;
                }
            }

            oos1[icrit * n_cases + oos1_end] =
                markets[ibest].close[oos1_end] - markets[ibest].close[oos1_end - 1];
        }

        if oos1_end >= n_cases - 1 {
            break; // Hit end of data
        }

        // Advance window: first half
        is_start += 1;
        oos1_end += 1;

        if oos1_end - oos1_start < oos1_n {
            continue; // Still filling OOS1
        }

        // Find best criterion in OOS1
        let mut best_crit = -1.0e60;
        let mut ibestcrit = 0;

        for icrit in 0..N_CRITERIA {
            let mut crit = 0.0;
            for i in oos1_start..oos1_end {
                crit += oos1[icrit * n_cases + i];
            }
            if crit > best_crit {
                best_crit = crit;
                ibestcrit = icrit;
            }
        }

        crit_count[ibestcrit] += 1;

        // Use best criterion to select market
        let crit_type = CriterionType::from_index(ibestcrit).unwrap();
        best_crit = -1.0e60;
        let mut ibest = 0;

        for (imarket, market) in markets.iter().enumerate() {
            let crit = criterion(crit_type, &market.close[oos2_end - is_n..oos2_end]);
            if crit > best_crit {
                best_crit = crit;
                ibest = imarket;
            }
        }

        // Record OOS2 return
        oos2[oos2_end] = markets[ibest].close[oos2_end] - markets[ibest].close[oos2_end - 1];
        oos1_start += 1;
        oos2_end += 1;
    }

    // Compute criterion performance
    let mut crit_perf = [0.0; N_CRITERIA];
    for i in 0..N_CRITERIA {
        let mut sum = 0.0;
        for j in oos2_start..oos2_end {
            sum += oos1[i * n_cases + j];
        }
        crit_perf[i] = 25200.0 * sum / (oos2_end - oos2_start) as f64;
    }

    // Compute final performance
    let mut sum = 0.0;
    for i in oos2_start..oos2_end {
        sum += oos2[i];
    }
    let final_perf = 25200.0 * sum / (oos2_end - oos2_start) as f64;

    // Print summary
    writeln!(
        fp_report,
        "\n\n25200 * mean log return of each criterion, and pct times chosen"
    )?;

    let total_count: usize = crit_count.iter().sum();

    for i in 0..N_CRITERIA {
        let crit_type = CriterionType::from_index(i).unwrap();
        writeln!(
            fp_report,
            "{:>15} {:9.4}  Chosen {:.1} pct",
            crit_type.name(),
            crit_perf[i],
            100.0 * crit_count[i] as f64 / total_count as f64
        )?;
    }

    writeln!(
        fp_report,
        "\n\n25200 * mean return of final system = {:.4}",
        final_perf
    )?;

    // Compute and print drawdown information
    let n = oos2_end - oos2_start;
    let divisor = bootstrap_reps / 10;

    println!("\n\nDoing bootstrap");

    let mut rng = Rng::new();

    for iboot in 0..bootstrap_reps {
        if iboot % divisor == 0 {
            print!(".");
            std::io::stdout().flush().ok();
        }

        // Collect bootstrap sample from entire OOS set
        for i in 0..n {
            let k = (rng.unifrand() * n as f64) as usize;
            let k = if k >= n { n - 1 } else { k };
            bootsample[i] = oos2[k + oos2_start];
        }

        // Compute four statistics
        let (q001_val, q01_val, q05_val, q10_val) = drawdown_quantiles(
            n,
            n_trades,
            &bootsample[..n],
            quantile_reps,
            &mut quantile_sample,
            &mut work,
            &mut rng,
        );

        q001[iboot] = q001_val;
        q01[iboot] = q01_val;
        q05[iboot] = q05_val;
        q10[iboot] = q10_val;
    }

    // Sort for CDF and find quantiles
    qsortd(&mut q001);
    qsortd(&mut q01);
    qsortd(&mut q05);
    qsortd(&mut q10);

    // Print for user
    writeln!(fp_report, "\n\nDrawdown approximate bounds.")?;
    writeln!(
        fp_report,
        "Rows are drawdown probability, columns are confidence in bounds."
    )?;
    writeln!(
        fp_report,
        "          0.5       0.6       0.7       0.8       0.9       0.95"
    )?;

    writeln!(
        fp_report,
        "0.001  {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}",
        find_quantile(bootstrap_reps, &q001, 0.5),
        find_quantile(bootstrap_reps, &q001, 0.6),
        find_quantile(bootstrap_reps, &q001, 0.7),
        find_quantile(bootstrap_reps, &q001, 0.8),
        find_quantile(bootstrap_reps, &q001, 0.9),
        find_quantile(bootstrap_reps, &q001, 0.95)
    )?;

    writeln!(
        fp_report,
        "0.01   {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}",
        find_quantile(bootstrap_reps, &q01, 0.5),
        find_quantile(bootstrap_reps, &q01, 0.6),
        find_quantile(bootstrap_reps, &q01, 0.7),
        find_quantile(bootstrap_reps, &q01, 0.8),
        find_quantile(bootstrap_reps, &q01, 0.9),
        find_quantile(bootstrap_reps, &q01, 0.95)
    )?;

    writeln!(
        fp_report,
        "0.05   {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}",
        find_quantile(bootstrap_reps, &q05, 0.5),
        find_quantile(bootstrap_reps, &q05, 0.6),
        find_quantile(bootstrap_reps, &q05, 0.7),
        find_quantile(bootstrap_reps, &q05, 0.8),
        find_quantile(bootstrap_reps, &q05, 0.9),
        find_quantile(bootstrap_reps, &q05, 0.95)
    )?;

    writeln!(
        fp_report,
        "0.10   {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}",
        find_quantile(bootstrap_reps, &q10, 0.5),
        find_quantile(bootstrap_reps, &q10, 0.6),
        find_quantile(bootstrap_reps, &q10, 0.7),
        find_quantile(bootstrap_reps, &q10, 0.8),
        find_quantile(bootstrap_reps, &q10, 0.9),
        find_quantile(bootstrap_reps, &q10, 0.95)
    )?;

    println!("\n\nResults written to CHOOSER.LOG");

    Ok(())
}
