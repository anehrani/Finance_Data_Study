use std::env;
use std::process;

use ::drawdown::*;

const POP_MULT: usize = 1000;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 8 {
        eprintln!("\nUsage: {} Nchanges Ntrades WinProb BoundConf BootstrapReps QuantileReps TestReps", args[0]);
        eprintln!("  Nchanges - Number of price changes");
        eprintln!("  Ntrades - Number of trades");
        eprintln!("  WinProb - Probability of winning");
        eprintln!("  BoundConf - Confidence (typically .5-.999) in correct dd bound");
        eprintln!("  BootstrapReps - Number of bootstrap reps");
        eprintln!("  QuantileReps - Number of bootstrap reps for finding drawdown quantiles");
        eprintln!("  TestReps - Number of testing reps for this study");
        process::exit(1);
    }

    let n_changes: usize = args[1].parse().expect("Invalid Nchanges");
    let n_trades: usize = args[2].parse().expect("Invalid Ntrades");
    let win_prob: f64 = args[3].parse().expect("Invalid WinProb");
    let bound_conf: f64 = args[4].parse().expect("Invalid BoundConf");
    let bootstrap_reps: usize = args[5].parse().expect("Invalid BootstrapReps");
    let quantile_reps: usize = args[6].parse().expect("Invalid QuantileReps");
    let test_reps: usize = args[7].parse().expect("Invalid TestReps");

    // Validate parameters
    if n_changes < 2 {
        eprintln!("\nERROR... Nchanges must be at least 2");
        process::exit(1);
    }
    if n_trades < 2 {
        eprintln!("\nERROR... Ntrades must be at least 2");
        process::exit(1);
    }
    if n_trades > n_changes {
        eprintln!("\nERROR... Ntrades must not exceed Nchanges");
        process::exit(1);
    }
    if !(0.0..=1.0).contains(&win_prob) {
        eprintln!("\nERROR... Winning probability must be 0-1");
        process::exit(1);
    }
    if bootstrap_reps < 10 {
        eprintln!("\nERROR... BootstrapReps must be at least 10");
        process::exit(1);
    }
    if quantile_reps < 10 {
        eprintln!("\nERROR... QuantileReps must be at least 10");
        process::exit(1);
    }
    if test_reps < 1 {
        eprintln!("\nERROR... TestReps must be at least 1");
        process::exit(1);
    }

    // Open output buffer
    let mut buffer = String::new();

    use std::fmt::Write;
    writeln!(buffer, "\nChanges = {}", n_changes).unwrap();
    writeln!(buffer, "Trades = {}", n_trades).unwrap();
    writeln!(buffer, "Win probability = {:.4}", win_prob).unwrap();
    writeln!(buffer, "DD bound confidence = {:.4}", bound_conf).unwrap();
    writeln!(buffer, "Bootstrap reps = {}", bootstrap_reps).unwrap();
    writeln!(buffer, "Quantile reps = {}", quantile_reps).unwrap();
    writeln!(buffer, "Test reps = {}", test_reps).unwrap();

    // Allocate memory
    let mut changes = Vec::with_capacity(n_changes);
    let mut bootsample = Vec::with_capacity(n_trades);
    let mut trades = Vec::with_capacity(n_changes);
    let mut incorrect_meanrets = Vec::with_capacity(bootstrap_reps);
    let mut incorrect_drawdowns = Vec::with_capacity(bootstrap_reps);
    let mut correct_q001 = Vec::with_capacity(bootstrap_reps);
    let mut correct_q01 = Vec::with_capacity(bootstrap_reps);
    let mut correct_q05 = Vec::with_capacity(bootstrap_reps);
    let mut correct_q10 = Vec::with_capacity(bootstrap_reps);
    let mut work = Vec::with_capacity(quantile_reps);

    // Counters
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

    // Main test loop
    for itest in 1..=test_reps {
        // Incorrect method test
        incorrect_meanrets.clear();
        incorrect_drawdowns.clear();

        for iboot in 0..bootstrap_reps {
            let make_changes = iboot == 0;
            get_trades(n_changes, n_trades, win_prob, make_changes, &mut changes, &mut trades);
            incorrect_meanrets.push(mean_return(&trades));
            incorrect_drawdowns.push(calc_drawdown(&trades));
        }

        // Sort and find quantiles
        incorrect_meanrets.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let incorrect_meanret_001 = find_quantile(&incorrect_meanrets, 0.001);
        let incorrect_meanret_01 = find_quantile(&incorrect_meanrets, 0.01);
        let incorrect_meanret_05 = find_quantile(&incorrect_meanrets, 0.05);
        let incorrect_meanret_10 = find_quantile(&incorrect_meanrets, 0.1);

        incorrect_drawdowns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let incorrect_drawdown_001 = find_quantile(&incorrect_drawdowns, 0.999);
        let incorrect_drawdown_01 = find_quantile(&incorrect_drawdowns, 0.99);
        let incorrect_drawdown_05 = find_quantile(&incorrect_drawdowns, 0.95);
        let incorrect_drawdown_10 = find_quantile(&incorrect_drawdowns, 0.9);

        // Correct method test
        correct_q001.clear();
        correct_q01.clear();
        correct_q05.clear();
        correct_q10.clear();

        for iboot in 0..bootstrap_reps {
            let make_changes = iboot == 0;
            get_trades(n_changes, n_changes, win_prob, make_changes, &mut changes, &mut trades);
            let (q001, q01, q05, q10) = drawdown_quantiles(
                n_changes,
                n_trades,
                &trades,
                quantile_reps,
                &mut bootsample,
                &mut work,
            );
            correct_q001.push(q001);
            correct_q01.push(q01);
            correct_q05.push(q05);
            correct_q10.push(q10);
        }

        // Sort and find bounds
        correct_q001.sort_by(|a, b| a.partial_cmp(b).unwrap());
        correct_q01.sort_by(|a, b| a.partial_cmp(b).unwrap());
        correct_q05.sort_by(|a, b| a.partial_cmp(b).unwrap());
        correct_q10.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let correct_q001_bound = find_quantile(&correct_q001, 1.0 - (1.0 - bound_conf) / 2.0);
        let correct_q01_bound = find_quantile(&correct_q01, 1.0 - (1.0 - bound_conf) / 2.0);
        let correct_q05_bound = find_quantile(&correct_q05, bound_conf);
        let correct_q10_bound = find_quantile(&correct_q10, bound_conf);

        // Population test
        for _ in 0..POP_MULT {
            trades.clear();
            for _ in 0..n_trades {
                let mut val = normal();
                if unifrand() < win_prob {
                    val = val.abs();
                } else {
                    val = -val.abs();
                }
                trades.push(val);
            }

            // Test mean return
            let crit = mean_return(&trades);
            if crit < incorrect_meanret_001 {
                count_incorrect_meanret_001 += 1;
            }
            if crit < incorrect_meanret_01 {
                count_incorrect_meanret_01 += 1;
            }
            if crit < incorrect_meanret_05 {
                count_incorrect_meanret_05 += 1;
            }
            if crit < incorrect_meanret_10 {
                count_incorrect_meanret_10 += 1;
            }

            // Test drawdown (incorrect method)
            let crit = calc_drawdown(&trades);
            if crit > incorrect_drawdown_001 {
                count_incorrect_drawdown_001 += 1;
            }
            if crit > incorrect_drawdown_01 {
                count_incorrect_drawdown_01 += 1;
            }
            if crit > incorrect_drawdown_05 {
                count_incorrect_drawdown_05 += 1;
            }
            if crit > incorrect_drawdown_10 {
                count_incorrect_drawdown_10 += 1;
            }

            // Test drawdown (correct method)
            if crit > correct_q001_bound {
                count_correct_001 += 1;
            }
            if crit > correct_q01_bound {
                count_correct_01 += 1;
            }
            if crit > correct_q05_bound {
                count_correct_05 += 1;
            }
            if crit > correct_q10_bound {
                count_correct_10 += 1;
            }
        }

        // Print progress to screen
        println!("\n\n{}", itest);
        println!("Mean return");
        println!("  Actual    Incorrect");
        println!("   0.001   {:.5}", count_incorrect_meanret_001 as f64 / (POP_MULT * itest) as f64);
        println!("   0.01    {:.5}", count_incorrect_meanret_01 as f64 / (POP_MULT * itest) as f64);
        println!("   0.05    {:.5}", count_incorrect_meanret_05 as f64 / (POP_MULT * itest) as f64);
        println!("   0.1     {:.5}", count_incorrect_meanret_10 as f64 / (POP_MULT * itest) as f64);

        println!("\nDrawdown");
        println!("  Actual    Incorrect  Correct");
        println!("   0.001   {:.5}  {:.5}",
                 count_incorrect_drawdown_001 as f64 / (POP_MULT * itest) as f64,
                 count_correct_001 as f64 / (POP_MULT * itest) as f64);
        println!("   0.01    {:.5}  {:.5}",
                 count_incorrect_drawdown_01 as f64 / (POP_MULT * itest) as f64,
                 count_correct_01 as f64 / (POP_MULT * itest) as f64);
        println!("   0.05    {:.5}  {:.5}",
                 count_incorrect_drawdown_05 as f64 / (POP_MULT * itest) as f64,
                 count_correct_05 as f64 / (POP_MULT * itest) as f64);
        println!("   0.1     {:.5}  {:.5}",
                 count_incorrect_drawdown_10 as f64 / (POP_MULT * itest) as f64,
                 count_correct_10 as f64 / (POP_MULT * itest) as f64);

        // Write results to buffer
        if itest % 100 == 0 || itest == test_reps {
            writeln!(buffer, "\n\n").unwrap();
            writeln!(buffer, "\nMean return worse (Ratio)").unwrap();
            writeln!(buffer, "  Actual       Incorrect").unwrap();
            writeln!(buffer, "   0.001   {:.5} ({:.2})",
                     count_incorrect_meanret_001 as f64 / (POP_MULT * itest) as f64,
                     (count_incorrect_meanret_001 as f64 / (POP_MULT * itest) as f64) / 0.001).unwrap();
            writeln!(buffer, "   0.01    {:.5} ({:.2})",
                     count_incorrect_meanret_01 as f64 / (POP_MULT * itest) as f64,
                     (count_incorrect_meanret_01 as f64 / (POP_MULT * itest) as f64) / 0.01).unwrap();
            writeln!(buffer, "   0.05    {:.5} ({:.2})",
                     count_incorrect_meanret_05 as f64 / (POP_MULT * itest) as f64,
                     (count_incorrect_meanret_05 as f64 / (POP_MULT * itest) as f64) / 0.05).unwrap();
            writeln!(buffer, "   0.1     {:.5} ({:.2})",
                     count_incorrect_meanret_10 as f64 / (POP_MULT * itest) as f64,
                     (count_incorrect_meanret_10 as f64 / (POP_MULT * itest) as f64) / 0.1).unwrap();

            writeln!(buffer, "\nDrawdown worse (Ratio)").unwrap();
            writeln!(buffer, "  Actual     Incorrect          Correct").unwrap();
            writeln!(buffer, "   0.001   {:.5} ({:.2})  {:.5} ({:.2})",
                     count_incorrect_drawdown_001 as f64 / (POP_MULT * itest) as f64,
                     (count_incorrect_drawdown_001 as f64 / (POP_MULT * itest) as f64) / 0.001,
                     count_correct_001 as f64 / (POP_MULT * itest) as f64,
                     (count_correct_001 as f64 / (POP_MULT * itest) as f64) / 0.001).unwrap();
            writeln!(buffer, "   0.01    {:.5} ({:.2})  {:.5} ({:.2})",
                     count_incorrect_drawdown_01 as f64 / (POP_MULT * itest) as f64,
                     (count_incorrect_drawdown_01 as f64 / (POP_MULT * itest) as f64) / 0.01,
                     count_correct_01 as f64 / (POP_MULT * itest) as f64,
                     (count_correct_01 as f64 / (POP_MULT * itest) as f64) / 0.01).unwrap();
            writeln!(buffer, "   0.05    {:.5} ({:.2})  {:.5} ({:.2})",
                     count_incorrect_drawdown_05 as f64 / (POP_MULT * itest) as f64,
                     (count_incorrect_drawdown_05 as f64 / (POP_MULT * itest) as f64) / 0.05,
                     count_correct_05 as f64 / (POP_MULT * itest) as f64,
                     (count_correct_05 as f64 / (POP_MULT * itest) as f64) / 0.05).unwrap();
            writeln!(buffer, "   0.1     {:.5} ({:.2})  {:.5} ({:.2})",
                     count_incorrect_drawdown_10 as f64 / (POP_MULT * itest) as f64,
                     (count_incorrect_drawdown_10 as f64 / (POP_MULT * itest) as f64) / 0.1,
                     count_correct_10 as f64 / (POP_MULT * itest) as f64,
                     (count_correct_10 as f64 / (POP_MULT * itest) as f64) / 0.10).unwrap();
            
            // Write to file (overwrite with current buffer)
            statn::core::io::write::write_file("DRAWDOWN.LOG", &buffer).expect("Failed to write DRAWDOWN.LOG");
        }
    }

    println!("\nResults written to DRAWDOWN.LOG");

}
