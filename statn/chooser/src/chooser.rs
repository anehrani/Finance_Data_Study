use anyhow::Result;
use std::fs::File;
use std::io::Write;

use crate::criteria::{criterion, CriterionType};
use crate::market_data::{align_dates, convert_to_log_prices, load_markets};
use crate::permutation::{do_permute, prepare_permute};
use crate::random::Rng;

const N_CRITERIA: usize = 3;

pub fn run_chooser(
    file_list: &str,
    is_n: usize,
    oos1_n: usize,
    mut nreps: usize,
) -> Result<()> {
    if nreps < 1 {
        nreps = 1;
    }

    if is_n < 2 || oos1_n < 1 {
        anyhow::bail!("Invalid parameters: IS_n must be >= 2 and OOS1_n must be >= 1");
    }

    // Open report file
    let mut fp_report = File::create("CHOOSER.LOG")?;
    writeln!(
        fp_report,
        "CHOOSER log with IS_n={}  OOS1_n={}  Reps={}",
        is_n, oos1_n, nreps
    )?;

    // Load markets
    let mut markets = load_markets(file_list)?;
    let n_markets = markets.len();

    // Align dates
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

    // Allocate permutation work arrays if needed
    let mut permute_work: Option<Vec<Vec<f64>>> = if nreps > 1 {
        Some(vec![vec![0.0; n_cases]; n_markets])
    } else {
        None
    };

    // Initialize RNG
    let mut rng = Rng::new();

    // Prepare permutation if needed
    if let Some(ref mut work) = permute_work {
        let market_close: Vec<Vec<f64>> = markets.iter().map(|m| m.close.clone()).collect();
        prepare_permute(is_n, n_markets, 1, &market_close, work);
        prepare_permute(is_n + oos1_n, n_markets, is_n, &market_close, work);
        prepare_permute(n_cases, n_markets, is_n + oos1_n, &market_close, work);
    }

    // Monte-Carlo permutation loop
    println!("\n\nComputing");

    let mut crit_count = [0usize; N_CRITERIA];
    let mut crit_perf = [0.0; N_CRITERIA];
    let mut crit_pval = [1usize; N_CRITERIA];
    let mut final_perf = 0.0;
    let mut final_pval = 1usize;

    for irep in 0..nreps {
        // Permute after first replication
        if irep > 0 {
            if let Some(ref mut work) = permute_work {
                let mut market_close: Vec<Vec<f64>> =
                    markets.iter().map(|m| m.close.clone()).collect();
                do_permute(is_n, n_markets, 1, &mut market_close, work, &mut rng);
                do_permute(is_n + oos1_n, n_markets, is_n, &mut market_close, work, &mut rng);
                do_permute(
                    n_cases,
                    n_markets,
                    is_n + oos1_n,
                    &mut market_close,
                    work,
                    &mut rng,
                );
                // Update markets with permuted data
                for (i, market) in markets.iter_mut().enumerate() {
                    market.close = market_close[i].clone();
                }
            }
        }

        // Initialize indices
        let mut is_start = 0;
        let mut oos1_start = is_n;
        let mut oos1_end = is_n;
        let oos2_start = is_n + oos1_n;
        let mut oos2_end = is_n + oos1_n;

        print!(".");
        std::io::stdout().flush().ok();

        // Main loop traversing market history
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

            if irep == 0 {
                crit_count[ibestcrit] += 1;
            }

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
        for i in 0..N_CRITERIA {
            let mut sum = 0.0;
            for j in oos2_start..oos2_end {
                sum += oos1[i * n_cases + j];
            }
            let perf = 25200.0 * sum / (oos2_end - oos2_start) as f64;

            if irep == 0 {
                crit_perf[i] = perf;
            } else if perf >= crit_perf[i] {
                crit_pval[i] += 1;
            }
        }

        // Compute final performance
        let mut sum = 0.0;
        for i in oos2_start..oos2_end {
            sum += oos2[i];
        }
        let perf = 25200.0 * sum / (oos2_end - oos2_start) as f64;

        if irep == 0 {
            final_perf = perf;
        } else if perf >= final_perf {
            final_pval += 1;
        }
    }

    // Print summary
    if nreps > 1 {
        writeln!(
            fp_report,
            "\n\n25200 * mean return of each criterion, p-value, and percent of times chosen..."
        )?;
    } else {
        writeln!(
            fp_report,
            "\n\n25200 * mean return of each criterion, and percent of times chosen..."
        )?;
    }

    let total_count: usize = crit_count.iter().sum();

    for i in 0..N_CRITERIA {
        let crit_type = CriterionType::from_index(i).unwrap();
        if nreps > 1 {
            writeln!(
                fp_report,
                "{:>15} {:9.4}  p={:.3}  Chosen {:.1} pct",
                crit_type.name(),
                crit_perf[i],
                crit_pval[i] as f64 / nreps as f64,
                100.0 * crit_count[i] as f64 / total_count as f64
            )?;
        } else {
            writeln!(
                fp_report,
                "{:>15} {:9.4}  Chosen {:.1} pct",
                crit_type.name(),
                crit_perf[i],
                100.0 * crit_count[i] as f64 / total_count as f64
            )?;
        }
    }

    if nreps > 1 {
        writeln!(
            fp_report,
            "\n\n25200 * mean return of final system = {:.4}  p={:.3}",
            final_perf,
            final_pval as f64 / nreps as f64
        )?;
    } else {
        writeln!(
            fp_report,
            "\n\n25200 * mean return of final system = {:.4}",
            final_perf
        )?;
    }

    println!("\n\nResults written to CHOOSER.LOG");

    Ok(())
}
