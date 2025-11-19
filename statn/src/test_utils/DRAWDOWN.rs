use rand::Rng;
use std::fs::File;
use std::io::Write;

const PI: f64 = std::f64::consts::PI;
const POP_MULT: usize = 1000;

/// Generate a standard normal random variable using Box-Muller method
fn normal(rng: &mut rand::rngs::ThreadRng) -> f64 {
    loop {
        let x1 = rng.gen::<f64>();
        if x1 <= 0.0 {
            continue;
        }
        let x1 = (-2.0 * x1.ln()).sqrt();
        let x2 = (2.0 * PI * rng.gen::<f64>()).cos();
        return x1 * x2;
    }
}

/// Generate a set of trades
fn get_trades(
    n_changes: usize,
    n_trades: usize,
    win_prob: f64,
    make_changes: bool,
    changes: &mut Vec<f64>,
    trades: &mut Vec<f64>,
    rng: &mut rand::rngs::ThreadRng,
) {
    if make_changes {
        for i in 0..n_changes {
            let mut change = normal(rng);
            if rng.gen::<f64>() < win_prob {
                change = change.abs();
            } else {
                change = -change.abs();
            }
            changes[i] = change;
        }
    }

    for itrade in 0..n_trades {
        let k = ((rng.gen::<f64>() * n_changes as f64) as usize).min(n_changes - 1);
        trades[itrade] = changes[k];
    }
}

/// Compute mean return
fn mean_return(trades: &[f64]) -> f64 {
    let sum: f64 = trades.iter().sum();
    sum / trades.len() as f64
}

/// Compute drawdown
fn drawdown(trades: &[f64]) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }

    let mut cumulative = trades[0];
    let mut max_price = trades[0];
    let mut dd = 0.0;

    for i in 1..trades.len() {
        cumulative += trades[i];
        if cumulative > max_price {
            max_price = cumulative;
        } else {
            let loss = max_price - cumulative;
            if loss > dd {
                dd = loss;
            }
        }
    }

    dd
}

/// Compute four drawdown quantiles
fn drawdown_quantiles(
    n_changes: usize,
    n_trades: usize,
    b_changes: &[f64],
    nboot: usize,
    bootsample: &mut Vec<f64>,
    work: &mut Vec<f64>,
    rng: &mut rand::rngs::ThreadRng,
) -> (f64, f64, f64, f64) {
    for iboot in 0..nboot {
        for i in 0..n_trades {
            let k = ((rng.gen::<f64>() * n_changes as f64) as usize).min(n_changes - 1);
            bootsample[i] = b_changes[k];
        }
        work[iboot] = drawdown(bootsample);
    }

    work.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let q001 = find_quantile(work, 0.999);
    let q01 = find_quantile(work, 0.99);
    let q05 = find_quantile(work, 0.95);
    let q10 = find_quantile(work, 0.9);

    (q001, q01, q05, q10)
}

/// Find a quantile
fn find_quantile(data: &[f64], frac: f64) -> f64 {
    let k = ((frac * (data.len() + 1) as f64) as usize).saturating_sub(1);
    data[k]
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Parse command line arguments
    if args.len() != 8 {
        eprintln!("Usage: drawdown Nchanges Ntrades WinProb BoundConf BootstrapReps QuantileReps TestReps");
        eprintln!("  Nchanges - Number of price changes");
        eprintln!("  Ntrades - Number of trades");
        eprintln!("  WinProb - Probability of winning");
        eprintln!("  BoundConf - Confidence (typically .5-.999) in correct dd bound");
        eprintln!("  BootstrapReps - Number of bootstrap reps");
        eprintln!("  QuantileReps - Number of bootstrap reps for finding drawdown quantiles");
        eprintln!("  TestReps - Number of testing reps for this study");
        std::process::exit(1);
    }

    let n_changes: usize = args[1].parse().expect("Invalid Nchanges");
    let n_trades: usize = args[2].parse().expect("Invalid Ntrades");
    let win_prob: f64 = args[3].parse().expect("Invalid WinProb");
    let bound_conf: f64 = args[4].parse().expect("Invalid BoundConf");
    let bootstrap_reps: usize = args[5].parse().expect("Invalid BootstrapReps");
    let quantile_reps: usize = args[6].parse().expect("Invalid QuantileReps");
    let test_reps: usize = args[7].parse().expect("Invalid TestReps");

    // Validation
    assert!(n_changes >= 2, "Nchanges must be at least 2");
    assert!(n_trades >= 2, "Ntrades must be at least 2");
    assert!(n_trades <= n_changes, "Ntrades must not exceed Nchanges");
    assert!(win_prob >= 0.0 && win_prob <= 1.0, "Winning probability must be 0-1");
    assert!(bootstrap_reps >= 10, "BootstrapReps must be at least 10");
    assert!(quantile_reps >= 10, "QuantileReps must be at least 10");
    assert!(test_reps >= 1, "TestReps must be at least 1");

    let mut fp = File::create("DRAWDOWN.LOG")?;

    writeln!(fp, "\nChanges = {}", n_changes)?;
    writeln!(fp, "Trades = {}", n_trades)?;
    writeln!(fp, "Win probability = {:.4}", win_prob)?;
    writeln!(fp, "DD bound confidence = {:.4}", bound_conf)?;
    writeln!(fp, "Bootstrap reps = {}", bootstrap_reps)?;
    writeln!(fp, "Quantile reps = {}", quantile_reps)?;
    writeln!(fp, "Test reps = {}", test_reps)?;

    // Allocate memory
    let mut changes = vec![0.0; n_changes];
    let mut bootsample = vec![0.0; n_trades];
    let mut trades = vec![0.0; n_changes];
    let mut incorrect_meanrets = vec![0.0; bootstrap_reps];
    let mut incorrect_drawdowns = vec![0.0; bootstrap_reps];
    let mut correct_q001 = vec![0.0; bootstrap_reps];
    let mut correct_q01 = vec![0.0; bootstrap_reps];
    let mut correct_q05 = vec![0.0; bootstrap_reps];
    let mut correct_q10 = vec![0.0; bootstrap_reps];
    let mut work = vec![0.0; quantile_reps];

    // Initialize counters
    let mut count_incorrect_meanret_001 = 0;
    let mut count_incorrect_meanret_01 = 0;
    let mut count_incorrect_meanret_05 = 0;
    let mut count_incorrect_meanret_10 = 0;
    let mut count_incorrect_drawdown_001 = 0;
    let mut count_incorrect_drawdown_01 = 0;
    let mut count_incorrect_drawdown_05 = 0;
    let mut count_incorrect_drawdown_10 = 0;
    let mut count_correct_001 = 0;
    let mut count_correct_01 = 0;
    let mut count_correct_05 = 0;
    let mut count_correct_10 = 0;

    let mut rng = rand::thread_rng();

    // Outer (test) loop
    for itest in 1..=test_reps {
        // Incorrect method test
        for iboot in 0..bootstrap_reps {
            let make_changes = iboot == 0;
            get_trades(
                n_changes,
                n_trades,
                win_prob,
                make_changes,
                &mut changes,
                &mut trades,
                &mut rng,
            );
            incorrect_meanrets[iboot] = mean_return(&trades[..n_trades]);
            incorrect_drawdowns[iboot] = drawdown(&trades[..n_trades]);
        }

        // Sort and find quantiles for mean returns
        incorrect_meanrets.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let incorrect_meanret_001 = find_quantile(&incorrect_meanrets, 0.001);
        let incorrect_meanret_01 = find_quantile(&incorrect_meanrets, 0.01);
        let incorrect_meanret_05 = find_quantile(&incorrect_meanrets, 0.05);
        let incorrect_meanret_10 = find_quantile(&incorrect_meanrets, 0.1);

        // Sort and find quantiles for drawdowns
        incorrect_drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let incorrect_drawdown_001 = find_quantile(&incorrect_drawdowns, 0.999);
        let incorrect_drawdown_01 = find_quantile(&incorrect_drawdowns, 0.99);
        let incorrect_drawdown_05 = find_quantile(&incorrect_drawdowns, 0.95);
        let incorrect_drawdown_10 = find_quantile(&incorrect_drawdowns, 0.9);

        // Correct method test
        for iboot in 0..bootstrap_reps {
            let make_changes = iboot == 0;
            get_trades(
                n_changes,
                n_changes,
                win_prob,
                make_changes,
                &mut changes,
                &mut trades,
                &mut rng,
            );
            let (q001, q01, q05, q10) = drawdown_quantiles(
                n_changes,
                n_trades,
                &trades,
                quantile_reps,
                &mut bootsample,
                &mut work,
                &mut rng,
            );
            correct_q001[iboot] = q001;
            correct_q01[iboot] = q01;
            correct_q05[iboot] = q05;
            correct_q10[iboot] = q10;
        }

        // Sort for CDF and find quantiles
        correct_q001.sort_by(|a, b| a.partial_cmp(b).unwrap());
        correct_q01.sort_by(|a, b| a.partial_cmp(b).unwrap());
        correct_q05.sort_by(|a, b| a.partial_cmp(b).unwrap());
        correct_q10.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let correct_q001_bound = find_quantile(&correct_q001, 1.0 - (1.0 - bound_conf) / 2.0);
        let correct_q01_bound = find_quantile(&correct_q01, 1.0 - (1.0 - bound_conf) / 2.0);
        let correct_q05_bound = find_quantile(&correct_q05, bound_conf);
        let correct_q10_bound = find_quantile(&correct_q10, bound_conf);

        // Population test
        for _ipop in 0..POP_MULT {
            for i in 0..n_trades {
                let mut trade = normal(&mut rng);
                if rng.gen::<f64>() < win_prob {
                    trade = trade.abs();
                } else {
                    trade = -trade.abs();
                }
                trades[i] = trade;
            }

            let crit_mean = mean_return(&trades[..n_trades]);
            if crit_mean < incorrect_meanret_001 {
                count_incorrect_meanret_001 += 1;
            }
            if crit_mean < incorrect_meanret_01 {
                count_incorrect_meanret_01 += 1;
            }
            if crit_mean < incorrect_meanret_05 {
                count_incorrect_meanret_05 += 1;
            }
            if crit_mean < incorrect_meanret_10 {
                count_incorrect_meanret_10 += 1;
            }

            let crit_dd = drawdown(&trades[..n_trades]);
            if crit_dd > incorrect_drawdown_001 {
                count_incorrect_drawdown_001 += 1;
            }
            if crit_dd > incorrect_drawdown_01 {
                count_incorrect_drawdown_01 += 1;
            }
            if crit_dd > incorrect_drawdown_05 {
                count_incorrect_drawdown_05 += 1;
            }
            if crit_dd > incorrect_drawdown_10 {
                count_incorrect_drawdown_10 += 1;
            }

            if crit_dd > correct_q001_bound {
                count_correct_001 += 1;
            }
            if crit_dd > correct_q01_bound {
                count_correct_01 += 1;
            }
            if crit_dd > correct_q05_bound {
                count_correct_05 += 1;
            }
            if crit_dd > correct_q10_bound {
                count_correct_10 += 1;
            }
        }

        // Print to console
        println!("\n{}", itest);
        println!("Mean return");
        println!("  Actual    Incorrect");
        println!(
            "   0.001   {:8.5}",
            count_incorrect_meanret_001 as f64 / (POP_MULT * itest) as f64
        );
        println!(
            "   0.01    {:8.5}",
            count_incorrect_meanret_01 as f64 / (POP_MULT * itest) as f64
        );
        println!(
            "   0.05    {:8.5}",
            count_incorrect_meanret_05 as f64 / (POP_MULT * itest) as f64
        );
        println!(
            "   0.1     {:8.5}",
            count_incorrect_meanret_10 as f64 / (POP_MULT * itest) as f64
        );

        println!("\nDrawdown");
        println!("  Actual    Incorrect  Correct");
        println!(
            "   0.001   {:8.5}  {:8.5}",
            count_incorrect_drawdown_001 as f64 / (POP_MULT * itest) as f64,
            count_correct_001 as f64 / (POP_MULT * itest) as f64
        );
        println!(
            "   0.01    {:8.5}  {:8.5}",
            count_incorrect_drawdown_01 as f64 / (POP_MULT * itest) as f64,
            count_correct_01 as f64 / (POP_MULT * itest) as f64
        );
        println!(
            "   0.05    {:8.5}  {:8.5}",
            count_incorrect_drawdown_05 as f64 / (POP_MULT * itest) as f64,
            count_correct_05 as f64 / (POP_MULT * itest) as f64
        );
        println!(
            "   0.1     {:8.5}  {:8.5}",
            count_incorrect_drawdown_10 as f64 / (POP_MULT * itest) as f64,
            count_correct_10 as f64 / (POP_MULT * itest) as f64
        );

        // Write to file periodically
        if itest % 100 == 0 || itest == test_reps {
            writeln!(fp, "\n\n")?;
            writeln!(fp, "Mean return worse (Ratio)")?;
            writeln!(fp, "  Actual       Incorrect")?;

            let ratio_001 =
                count_incorrect_meanret_001 as f64 / (POP_MULT * itest) as f64 / 0.001;
            let actual_001 = count_incorrect_meanret_001 as f64 / (POP_MULT * itest) as f64;
            writeln!(fp, "   0.001   {:8.5} ({:6.2})", actual_001, ratio_001)?;

            let ratio_01 = count_incorrect_meanret_01 as f64 / (POP_MULT * itest) as f64 / 0.01;
            let actual_01 = count_incorrect_meanret_01 as f64 / (POP_MULT * itest) as f64;
            writeln!(fp, "   0.01    {:8.5} ({:6.2})", actual_01, ratio_01)?;

            let ratio_05 = count_incorrect_meanret_05 as f64 / (POP_MULT * itest) as f64 / 0.05;
            let actual_05 = count_incorrect_meanret_05 as f64 / (POP_MULT * itest) as f64;
            writeln!(fp, "   0.05    {:8.5} ({:6.2})", actual_05, ratio_05)?;

            let ratio_10 = count_incorrect_meanret_10 as f64 / (POP_MULT * itest) as f64 / 0.1;
            let actual_10 = count_incorrect_meanret_10 as f64 / (POP_MULT * itest) as f64;
            writeln!(fp, "   0.1     {:8.5} ({:6.2})", actual_10, ratio_10)?;

            writeln!(fp, "\nDrawdown worse (Ratio)")?;
            writeln!(fp, "  Actual     Incorrect          Correct")?;

            let incorrect_actual_001 =
                count_incorrect_drawdown_001 as f64 / (POP_MULT * itest) as f64;
            let incorrect_ratio_001 = incorrect_actual_001 / 0.001;
            let correct_actual_001 = count_correct_001 as f64 / (POP_MULT * itest) as f64;
            let correct_ratio_001 = correct_actual_001 / 0.001;
            writeln!(
                fp,
                "   0.001   {:8.5} ({:6.2})  {:8.5} ({:6.2})",
                incorrect_actual_001, incorrect_ratio_001, correct_actual_001, correct_ratio_001
            )?;

            let incorrect_actual_01 =
                count_incorrect_drawdown_01 as f64 / (POP_MULT * itest) as f64;
            let incorrect_ratio_01 = incorrect_actual_01 / 0.01;
            let correct_actual_01 = count_correct_01 as f64 / (POP_MULT * itest) as f64;
            let correct_ratio_01 = correct_actual_01 / 0.01;
            writeln!(
                fp,
                "   0.01    {:8.5} ({:6.2})  {:8.5} ({:6.2})",
                incorrect_actual_01, incorrect_ratio_01, correct_actual_01, correct_ratio_01
            )?;

            let incorrect_actual_05 =
                count_incorrect_drawdown_05 as f64 / (POP_MULT * itest) as f64;
            let incorrect_ratio_05 = incorrect_actual_05 / 0.05;
            let correct_actual_05 = count_correct_05 as f64 / (POP_MULT * itest) as f64;
            let correct_ratio_05 = correct_actual_05 / 0.05;
            writeln!(
                fp,
                "   0.05    {:8.5} ({:6.2})  {:8.5} ({:6.2})",
                incorrect_actual_05, incorrect_ratio_05, correct_actual_05, correct_ratio_05
            )?;

            let incorrect_actual_10 =
                count_incorrect_drawdown_10 as f64 / (POP_MULT * itest) as f64;
            let incorrect_ratio_10 = incorrect_actual_10 / 0.1;
            let correct_actual_10 = count_correct_10 as f64 / (POP_MULT * itest) as f64;
            let correct_ratio_10 = correct_actual_10 / 0.1;
            writeln!(
                fp,
                "   0.1     {:8.5} ({:6.2})  {:8.5} ({:6.2})",
                incorrect_actual_10, incorrect_ratio_10, correct_actual_10, correct_ratio_10
            )?;
        }
    }

    Ok(())
}