use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::f64;

// External functions (assumed to be already implemented in Rust)
fn unifrand() -> f64;
fn qsortd(first: usize, last: usize, data: &mut [f64]);

const MAX_MARKETS: usize = 1024;
const MAX_NAME_LENGTH: usize = 16;
const BLOCK_SIZE: usize = 4096;
const MAX_CRITERIA: usize = 16;

/// Compute drawdown
/// This assumes that the trades are log of equity changes.
/// It returns percent drawdown.
fn drawdown(trades: &[f64]) -> f64 {
    if trades.is_empty() {
        return 0.0;
    }

    let mut cumulative = trades[0];
    let mut max_price = trades[0];
    let mut dd = 0.0;

    for &trade in &trades[1..] {
        cumulative += trade;
        if cumulative > max_price {
            max_price = cumulative;
        } else {
            let loss = max_price - cumulative;
            if loss > dd {
                dd = loss;
            }
        }
    }

    100.0 * (1.0 - (-dd).exp())
}

/// Compute four drawdown quantiles
fn drawdown_quantiles(
    n_changes: usize,
    n_trades: usize,
    b_changes: &[f64],
    nboot: usize,
    quantsample: &mut [f64],
    work: &mut [f64],
) -> (f64, f64, f64, f64) {
    for iboot in 0..nboot {
        for i in 0..n_trades {
            let k = ((unifrand() * n_changes as f64) as usize).min(n_changes - 1);
            quantsample[i] = b_changes[k];
        }
        work[iboot] = drawdown(quantsample);
    }

    qsortd(0, nboot - 1, work);

    let get_quantile = |fraction: f64| -> f64 {
        let k = ((fraction * (nboot + 1) as f64) as i32 - 1).max(0) as usize;
        work[k]
    };

    (
        get_quantile(0.999),
        get_quantile(0.99),
        get_quantile(0.95),
        get_quantile(0.90),
    )
}

/// Find a quantile
fn find_quantile(data: &[f64], frac: f64) -> f64 {
    let n = data.len();
    let k = ((frac * (n + 1) as f64) as i32 - 1).max(0) as usize;
    data[k]
}

/// Criterion function: total return (assumes 'prices' are actually log prices)
fn total_return(prices: &[f64]) -> f64 {
    if prices.is_empty() {
        return 0.0;
    }
    prices[prices.len() - 1] - prices[0]
}

/// Criterion function: raw Sharpe ratio (assumes 'prices' are actually log prices)
fn sharpe_ratio(prices: &[f64]) -> f64 {
    if prices.len() < 2 {
        return 0.0;
    }

    let n = prices.len() as f64;
    let mean = (prices[prices.len() - 1] - prices[0]) / (n - 1.0);

    let mut var = 1.0e-60;
    for i in 1..prices.len() {
        let diff = (prices[i] - prices[i - 1]) - mean;
        var += diff * diff;
    }

    mean / (var / (n - 1.0)).sqrt()
}

/// Criterion function: profit factor (assumes 'prices' are actually log prices)
fn profit_factor(prices: &[f64]) -> f64 {
    let mut win_sum = 1.0e-60;
    let mut lose_sum = 1.0e-60;

    for i in 1..prices.len() {
        let ret = prices[i] - prices[i - 1];
        if ret > 0.0 {
            win_sum += ret;
        } else {
            lose_sum -= ret;
        }
    }

    win_sum / lose_sum
}

/// Master criterion function
fn criterion(which: usize, prices: &[f64]) -> f64 {
    match which {
        0 => total_return(prices),
        1 => sharpe_ratio(prices),
        2 => profit_factor(prices),
        _ => -1.0e60,
    }
}

/// Parse a line to extract OHLC data
fn parse_market_line(line: &str) -> Option<(i32, f64, f64, f64, f64)> {
    let fields: Vec<&str> = line
        .split(|c: char| c == ' ' || c == ',' || c == '\t' || c == '/')
        .filter(|s| !s.is_empty())
        .collect();

    if fields.len() < 5 {
        return None;
    }

    let date = fields[0].parse::<i32>().ok()?;
    let open = fields[1].parse::<f64>().ok()?;
    let high = fields[2].parse::<f64>().ok()?;
    let low = fields[3].parse::<f64>().ok()?;
    let close = fields[4].parse::<f64>().ok()?;

    Some((date, open, high, low, close))
}

/// Extract market name from file path
fn extract_market_name(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    let file_stem = path.file_stem()?.to_str()?;
    
    if file_stem.len() > MAX_NAME_LENGTH - 1 {
        return None;
    }
    
    Some(file_stem.to_string())
}

/// Main program
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        println!("USAGE: CHOOSER FileList IS_n OOS1_n");
        println!("  FileList - Text file containing list of competing market history files");
        println!("  IS_n - N of market history records for each selection criterion to analyze");
        println!("  OOS1_n - N of OOS records for choosing best criterion");
        std::process::exit(0);
    }

    let file_list_name = &args[1];
    let is_n = args[2].parse::<usize>().expect("IS_n must be a number");
    let oos1_n = args[3].parse::<usize>().expect("OOS1_n must be a number");

    if is_n < 2 || oos1_n < 1 {
        println!("USAGE: CHOOSER FileList IS_n OOS1_n");
        println!("  FileList - Text file containing list of competing market history files");
        println!("  IS_n - N of market history records for each selection criterion to analyze");
        println!("  OOS1_n - N of OOS records for choosing best criterion");
        std::process::exit(0);
    }

    let n_criteria = 3;
    let bootstrap_reps = 2000;
    let quantile_reps = 10000;
    let n_trades = 252; // One year if daily prices

    // Open report file
    let mut fp_report = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("CHOOSER.LOG")
    {
        Ok(f) => f,
        Err(_) => {
            eprintln!("ERROR... Cannot open CHOOSER.LOG for writing");
            std::process::exit(1);
        }
    };

    let _ = writeln!(
        fp_report,
        "CHOOSER_DD log with IS_n={} OOS1_n={}",
        is_n, oos1_n
    );

    // Read market list and data
    let file_list = match std::fs::read_to_string(file_list_name) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("ERROR... Cannot open list file {}", file_list_name);
            std::process::exit(1);
        }
    };

    let mut market_names: Vec<String> = Vec::new();
    let mut market_dates: Vec<Vec<i32>> = Vec::new();
    let mut market_closes: Vec<Vec<f64>> = Vec::new();

    for line in file_list.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let market_file = trimmed;
        let market_name = match extract_market_name(market_file) {
            Some(name) => name,
            Err(_) => {
                eprintln!("ERROR... Invalid market file name ({})", market_file);
                continue;
            }
        };

        let market_data = match std::fs::read_to_string(market_file) {
            Ok(content) => content,
            Err(_) => {
                eprintln!("ERROR... Cannot open market file {}", market_file);
                continue;
            }
        };

        println!("Reading market file {}...", market_file);

        let mut dates = Vec::new();
        let mut closes = Vec::new();
        let mut prior_date = 0;

        for line in market_data.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parsed = match parse_market_line(trimmed) {
                Some(p) => p,
                None => continue,
            };

            let (full_date, _open, high, low, close) = parsed;

            // Validate date
            let year = full_date / 10000;
            let month = (full_date % 10000) / 100;
            let day = full_date % 100;

            if month < 1 || month > 12 || day < 1 || day > 31 || year < 1800 || year > 2030 {
                eprintln!(
                    "ERROR... Invalid date {} in market file {} ",
                    full_date, market_file
                );
                std::process::exit(1);
            }

            if full_date <= prior_date {
                eprintln!(
                    "ERROR... Date failed to increase in market file {}",
                    market_file
                );
                std::process::exit(1);
            }

            prior_date = full_date;

            dates.push(full_date);
            closes.push(close);
        }

        if !dates.is_empty() {
            let _ = writeln!(
                fp_report,
                "Market file {} had {} records from date {} to {}",
                market_file,
                dates.len(),
                dates[0],
                dates[dates.len() - 1]
            );

            market_names.push(market_name);
            market_dates.push(dates);
            market_closes.push(closes);
        }
    }

    let n_markets = market_names.len();

    if n_markets == 0 {
        eprintln!("ERROR... No markets loaded");
        std::process::exit(1);
    }

    // Align dates across all markets
    println!("\nAligning dates...");

    let mut market_indices = vec![0; n_markets];
    let mut grand_index = 0;
    let mut aligned_dates = Vec::new();
    let mut aligned_closes = vec![Vec::new(); n_markets];

    loop {
        // Find max date at current index
        let mut max_date = 0;
        for i in 0..n_markets {
            if market_indices[i] < market_dates[i].len() {
                let date = market_dates[i][market_indices[i]];
                if date > max_date {
                    max_date = date;
                }
            }
        }

        // Advance all markets until they reach or pass max date
        let mut all_same_date = true;
        let mut any_finished = false;

        for i in 0..n_markets {
            while market_indices[i] < market_dates[i].len()
                && market_dates[i][market_indices[i]] < max_date
            {
                market_indices[i] += 1;
            }

            if market_indices[i] >= market_dates[i].len() {
                any_finished = true;
                break;
            }

            if market_dates[i][market_indices[i]] != max_date {
                all_same_date = false;
            }
        }

        if any_finished {
            break;
        }

        if all_same_date {
            aligned_dates.push(max_date);
            for i in 0..n_markets {
                aligned_closes[i].push(market_closes[i][market_indices[i]]);
                market_indices[i] += 1;
            }
            grand_index += 1;
        }
    }

    let n_cases = grand_index;

    let _ = writeln!(
        fp_report,
        "\nMerged database has {} records from date {} to {}",
        n_cases, aligned_dates[0], aligned_dates[n_cases - 1]
    );

    // Convert all closing prices to log prices
    for imarket in 0..n_markets {
        for i in 0..n_cases {
            aligned_closes[imarket][i] = aligned_closes[imarket][i].ln();
        }
    }

    // Print return of each market over the OOS2 period
    let oos2_start = is_n + oos1_n;

    let _ = writeln!(
        fp_report,
        "\n25200 * mean return of each market in OOS2 period..."
    );
    let mut sum = 0.0;
    for i in 0..n_markets {
        let ret = 25200.0
            * (aligned_closes[i][n_cases - 1] - aligned_closes[i][oos2_start - 1])
            / (n_cases - oos2_start) as f64;
        sum += ret;
        let _ = writeln!(fp_report, "{:15} {:9.4}", market_names[i], ret);
    }
    let _ = writeln!(fp_report, "Mean = {:9.4}", sum / n_markets as f64);

    // Allocate memory for OOS1, OOS2, and drawdown stuff
    let mut oos1 = vec![0.0; n_criteria * n_cases];
    let mut oos2 = vec![0.0; n_cases];
    let mut bootsample = vec![0.0; n_cases];
    let mut quantile_sample = vec![0.0; n_trades];
    let mut work = vec![0.0; quantile_reps];
    let mut q001 = vec![0.0; bootstrap_reps];
    let mut q01 = vec![0.0; bootstrap_reps];
    let mut q05 = vec![0.0; bootstrap_reps];
    let mut q10 = vec![0.0; bootstrap_reps];

    // Initialize
    let mut crit_count = vec![0; n_criteria];

    let mut is_start = 0;
    let mut oos1_start = is_n;
    let mut oos1_end = is_n;
    let mut oos2_end = oos2_start;

    // Main outermost loop traverses market history
    println!("\nComputing trades...");

    loop {
        // Evaluate all performance criteria for all markets
        for icrit in 0..n_criteria {
            let mut best_crit = -1.0e60;
            let mut ibest = 0;

            for imarket in 0..n_markets {
                let prices = &aligned_closes[imarket][is_start..is_start + is_n];
                let crit = criterion(icrit, prices);

                if crit > best_crit {
                    best_crit = crit;
                    ibest = imarket;
                }
            }

            if oos1_end < n_cases {
                oos1[icrit * n_cases + oos1_end] =
                    aligned_closes[ibest][oos1_end] - aligned_closes[ibest][oos1_end - 1];
            }
        }

        if oos1_end >= n_cases - 1 {
            break;
        }

        is_start += 1;
        oos1_end += 1;

        if oos1_end - oos1_start < oos1_n {
            continue;
        }

        // Find the best criterion in OOS1
        let mut best_crit = -1.0e60;
        let mut ibestcrit = 0;

        for icrit in 0..n_criteria {
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

        // Use recently best criterion to select market
        best_crit = -1.0e60;
        let mut ibest = 0;

        for imarket in 0..n_markets {
            let start_idx = if oos2_end >= is_n {
                oos2_end - is_n
            } else {
                0
            };
            let end_idx = oos2_end.min(aligned_closes[imarket].len());
            
            if start_idx < end_idx {
                let prices = &aligned_closes[imarket][start_idx..end_idx];
                let crit = criterion(ibestcrit, prices);

                if crit > best_crit {
                    best_crit = crit;
                    ibest = imarket;
                }
            }
        }

        if oos2_end < n_cases {
            oos2[oos2_end] =
                aligned_closes[ibest][oos2_end] - aligned_closes[ibest][oos2_end - 1];
        }

        oos1_start += 1;
        oos2_end += 1;
    }

    // Compute and save mean market performance of each criterion
    let mut crit_perf = vec![0.0; n_criteria];

    for i in 0..n_criteria {
        let mut sum = 0.0;
        for j in oos2_start..oos2_end {
            sum += oos1[i * n_cases + j];
        }
        let perf = 25200.0 * sum / (oos2_end - oos2_start) as f64;
        crit_perf[i] = perf;
    }

    // Compute and save final return
    let mut sum = 0.0;
    for i in oos2_start..oos2_end {
        sum += oos2[i];
    }
    let final_perf = 25200.0 * sum / (oos2_end - oos2_start) as f64;

    // Print summary information
    let _ = writeln!(
        fp_report,
        "\n25200 * mean log return of each criterion, and pct times chosen"
    );

    let mut total_count = 0;
    for i in 0..n_criteria {
        total_count += crit_count[i];
    }

    for i in 0..n_criteria {
        let criterion_name = match i {
            0 => "Total return",
            1 => "Sharpe ratio",
            2 => "Profit factor",
            _ => "ERROR",
        };

        let pct = if total_count > 0 {
            100.0 * crit_count[i] as f64 / total_count as f64
        } else {
            0.0
        };

        let _ = writeln!(
            fp_report,
            "{:15} {:9.4}  Chosen {:.1}%",
            criterion_name, crit_perf[i], pct
        );
    }

    let _ = writeln!(
        fp_report,
        "\n25200 * mean return of final system = {:.4}",
        final_perf
    );

    // Compute and print drawdown information
    let n = oos2_end - oos2_start;
    let divisor = (bootstrap_reps / 10).max(1);

    println!("\nDoing bootstrap");
    for iboot in 0..bootstrap_reps {
        if iboot % divisor == 0 {
            print!(".");
            let _ = std::io::stdout().flush();
        }

        for i in 0..n {
            let k = ((unifrand() * n as f64) as usize).min(n - 1);
            bootsample[i] = oos2[k + oos2_start];
        }

        let (q001_val, q01_val, q05_val, q10_val) =
            drawdown_quantiles(n, n_trades, &bootsample, quantile_reps, &mut quantile_sample, &mut work);

        q001[iboot] = q001_val;
        q01[iboot] = q01_val;
        q05[iboot] = q05_val;
        q10[iboot] = q10_val;
    }

    // Sort for CDF and find quantiles
    qsortd(0, bootstrap_reps - 1, &mut q001);
    qsortd(0, bootstrap_reps - 1, &mut q01);
    qsortd(0, bootstrap_reps - 1, &mut q05);
    qsortd(0, bootstrap_reps - 1, &mut q10);

    // Print for user
    let _ = writeln!(fp_report, "\nDrawdown approximate bounds.");
    let _ = writeln!(
        fp_report,
        "Rows are drawdown probability, columns are confidence in bounds."
    );
    let _ = writeln!(fp_report, "          0.5       0.6       0.7       0.8       0.9       0.95");

    let _ = writeln!(
        fp_report,
        "0.001  {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}",
        find_quantile(&q001, 0.5),
        find_quantile(&q001, 0.6),
        find_quantile(&q001, 0.7),
        find_quantile(&q001, 0.8),
        find_quantile(&q001, 0.9),
        find_quantile(&q001, 0.95)
    );

    let _ = writeln!(
        fp_report,
        "0.01   {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}",
        find_quantile(&q01, 0.5),
        find_quantile(&q01, 0.6),
        find_quantile(&q01, 0.7),
        find_quantile(&q01, 0.8),
        find_quantile(&q01, 0.9),
        find_quantile(&q01, 0.95)
    );

    let _ = writeln!(
        fp_report,
        "0.05   {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}",
        find_quantile(&q05, 0.5),
        find_quantile(&q05, 0.6),
        find_quantile(&q05, 0.7),
        find_quantile(&q05, 0.8),
        find_quantile(&q05, 0.9),
        find_quantile(&q05, 0.95)
    );

    let _ = writeln!(
        fp_report,
        "0.10   {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}  {:8.3}",
        find_quantile(&q10, 0.5),
        find_quantile(&q10, 0.6),
        find_quantile(&q10, 0.7),
        find_quantile(&q10, 0.8),
        find_quantile(&q10, 0.9),
        find_quantile(&q10, 0.95)
    );

    println!("\nDone!");
}