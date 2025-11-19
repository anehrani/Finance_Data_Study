use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process;

const MKTBUF: usize = 128; // Alloc for market info in chunks of this many records

/// Criterion function for CSCV - calculates mean of returns
fn criter(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    returns.iter().sum::<f64>() / returns.len() as f64
}

/// Computationally symmetric cross validation core routine
fn cscvcore(
    ncases: usize,
    n_systems: usize,
    n_blocks_input: usize,
    returns: &[f64],
    indices: &mut Vec<usize>,
    lengths: &mut Vec<usize>,
    flags: &mut Vec<u8>,
    work: &mut Vec<f64>,
    is_crits: &mut Vec<f64>,
    oos_crits: &mut Vec<f64>,
) -> f64 {
    let n_blocks = (n_blocks_input / 2) * 2;

    assert_eq!(
        returns.len(),
        n_systems * ncases,
        "Returns matrix size mismatch"
    );
    assert!(n_blocks > 0 && n_blocks % 2 == 0, "n_blocks must be even and > 0");

    indices.clear();
    indices.resize(n_blocks, 0);
    lengths.clear();
    lengths.resize(n_blocks, 0);
    flags.clear();
    flags.resize(n_blocks, 0);
    work.clear();
    work.resize(ncases, 0.0);
    is_crits.clear();
    is_crits.resize(n_systems, 0.0);
    oos_crits.clear();
    oos_crits.resize(n_systems, 0.0);

    // Find the starting index and length of each of the n_blocks submatrices
    let mut istart = 0;
    for i in 0..n_blocks {
        indices[i] = istart;
        lengths[i] = (ncases - istart) / (n_blocks - i);
        istart += lengths[i];
    }

    let mut nless = 0;

    // Identify the training set blocks
    for i in 0..(n_blocks / 2) {
        flags[i] = 1;
    }

    // Identify the test set blocks
    for i in (n_blocks / 2)..n_blocks {
        flags[i] = 0;
    }

    // Main loop processes all combinations of blocks
    let mut ncombo = 0;
    loop {
        // Compute training-set (IS) criterion for each candidate system
        for isys in 0..n_systems {
            let mut n = 0;

            for ic in 0..n_blocks {
                if flags[ic] == 1 {
                    for i in indices[ic]..(indices[ic] + lengths[ic]) {
                        work[n] = returns[isys * ncases + i];
                        n += 1;
                    }
                }
            }

            is_crits[isys] = criter(&work[0..n]);
        }

        // Compute (OOS) criterion for each candidate system
        for isys in 0..n_systems {
            let mut n = 0;

            for ic in 0..n_blocks {
                if flags[ic] == 0 {
                    for i in indices[ic]..(indices[ic] + lengths[ic]) {
                        work[n] = returns[isys * ncases + i];
                        n += 1;
                    }
                }
            }

            oos_crits[isys] = criter(&work[0..n]);
        }

        // Find the best system IS
        let mut best = f64::NEG_INFINITY;
        let mut ibest = 0;
        for isys in 0..n_systems {
            if is_crits[isys] > best {
                best = is_crits[isys];
                ibest = isys;
            }
        }

        best = oos_crits[ibest];
        let mut n = 0;
        for isys in 0..n_systems {
            if isys == ibest || best >= oos_crits[isys] {
                n += 1;
            }
        }

        let rel_rank = n as f64 / (n_systems as f64 + 1.0);

        if rel_rank <= 0.5 {
            nless += 1;
        }

        // Move to the next combination
        let mut found_next = false;
        let mut n = 0;
        for iradix in 0..(n_blocks - 1) {
            if flags[iradix] == 1 {
                n += 1;
                if flags[iradix + 1] == 0 {
                    flags[iradix] = 0;
                    flags[iradix + 1] = 1;

                    for i in 0..iradix {
                        n -= 1;
                        if n > 0 {
                            flags[i] = 1;
                        } else {
                            flags[i] = 0;
                        }
                    }

                    found_next = true;
                    break;
                }
            }
        }

        ncombo += 1;

        if !found_next {
            break;
        }
    }

    nless as f64 / ncombo as f64
}

/// Computes one-bar returns for all short-term and long-term lookbacks
/// of a primitive moving-average crossover system.
fn get_returns(prices: &[f64], max_lookback: usize, returns: &mut Vec<f64>) {
    returns.clear();

    for ilong in 2..=max_lookback {
        for ishort in 1..ilong {
            let mut short_sum = 0.0;
            let mut long_sum = 0.0;

            for i in (max_lookback - 1)..(prices.len() - 1) {
                if i == max_lookback - 1 {
                    // Calculate initial moving averages
                    for j in 0..ishort {
                        short_sum += prices[i - j];
                    }
                    long_sum = short_sum;
                    for j in ishort..ilong {
                        long_sum += prices[i - j];
                    }
                } else {
                    // Update moving averages
                    short_sum += prices[i] - prices[i - ishort];
                    long_sum += prices[i] - prices[i - ilong];
                }

                let short_mean = short_sum / ishort as f64;
                let long_mean = long_sum / ilong as f64;

                let ret = if short_mean > long_mean {
                    prices[i + 1] - prices[i]
                } else if short_mean < long_mean {
                    prices[i] - prices[i + 1]
                } else {
                    0.0
                };

                returns.push(ret);
            }
        }
    }

    let expected_len = max_lookback * (max_lookback - 1) / 2 * (prices.len() - max_lookback);
    assert_eq!(
        returns.len(),
        expected_len,
        "Returns vector length mismatch"
    );
}

/// Read market prices from file (format: YYYYMMDD Price)
fn read_market_file(filename: &str) -> Result<Vec<f64>, String> {
    let path = Path::new(filename);
    let file = File::open(path).map_err(|e| format!("Cannot open file {}: {}", filename, e))?;

    let reader = BufReader::new(file);
    let mut prices = Vec::new();

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|e| format!("Error reading line {}: {}", line_num + 1, e))?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Validate date format (first 8 characters should be digits)
        if line.len() < 9 {
            return Err(format!("Invalid line {} (too short)", line_num + 1));
        }

        for i in 0..8 {
            if !line.chars().nth(i).unwrap().is_ascii_digit() {
                return Err(format!("Invalid date on line {}", line_num + 1));
            }
        }

        // Parse the price (starting from column 9, skipping whitespace and delimiters)
        let price_str = line[9..].trim_start_matches(|c: char| c.is_whitespace() || c == ',');

        match price_str.parse::<f64>() {
            Ok(price) if price > 0.0 => {
                prices.push(price.ln()); // Store log price
            }
            Ok(_) => {
                return Err(format!("Invalid price on line {}: must be positive", line_num + 1));
            }
            Err(e) => {
                return Err(format!("Invalid price on line {}: {}", line_num + 1, e));
            }
        }
    }

    if prices.is_empty() {
        return Err("No prices read from file".to_string());
    }

    Ok(prices)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    if args.len() != 4 {
        eprintln!("\nUsage: cscv_mkt  n_blocks  max_lookback  filename");
        eprintln!("  n_blocks - number of blocks into which cases are partitioned");
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        process::exit(1);
    }

    let n_blocks: usize = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing n_blocks");
        process::exit(1);
    });

    let max_lookback: usize = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Error parsing max_lookback");
        process::exit(1);
    });

    let filename = &args[3];

    // Read market prices
    println!("Reading market file...");

    let prices = match read_market_file(filename) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("\n{}", e);
            process::exit(1);
        }
    };

    let nprices = prices.len();
    println!("Market price history read");

    // Initialize
    let n_returns = nprices - max_lookback;
    let n_systems = max_lookback * (max_lookback - 1) / 2;

    if nprices < 2 || n_blocks < 2 || max_lookback < 2 || n_returns < n_blocks {
        eprintln!("\nUsage: cscv_mkt  n_blocks  max_lookback  filename");
        eprintln!("  n_blocks - number of blocks into which cases are partitioned");
        eprintln!("  max_lookback - Maximum moving-average lookback");
        eprintln!("  filename - name of market file (YYYYMMDD Price)");
        process::exit(1);
    }

    println!(
        "\n\nnprices={}  n_blocks={}  max_lookback={}  n_systems={}  n_returns={}",
        nprices, n_blocks, max_lookback, n_systems, n_returns
    );

    // Compute returns
    let mut returns = Vec::new();
    get_returns(&prices, max_lookback, &mut returns);

    // Prepare work vectors
    let mut indices = Vec::new();
    let mut lengths = Vec::new();
    let mut flags = Vec::new();
    let mut work = Vec::new();
    let mut is_crits = Vec::new();
    let mut oos_crits = Vec::new();

    // Run CSCV core
    let prob = cscvcore(
        n_returns,
        n_systems,
        n_blocks,
        &returns,
        &mut indices,
        &mut lengths,
        &mut flags,
        &mut work,
        &mut is_crits,
        &mut oos_crits,
    );

    // Find return of grand best system
    let mut best_crit = f64::NEG_INFINITY;
    for i in 0..n_systems {
        let start = i * n_returns;
        let end = start + n_returns;
        let crit = criter(&returns[start..end]);
        if crit > best_crit {
            best_crit = crit;
        }
    }

    // Print results
    println!(
        "\n\nnprices={}  n_blocks={}  max_lookback={}  n_systems={}  n_returns={}",
        nprices, n_blocks, max_lookback, n_systems, n_returns
    );
    println!(
        "1000 * Grand criterion = {:.4}  Prob = {:.4}",
        1000.0 * best_crit,
        prob
    );

    println!("\nPress Enter to exit...");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_criter_basic() {
        let returns = vec![1.0, 2.0, 3.0, 4.0];
        let result = criter(&returns);
        assert!((result - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_get_returns_small() {
        let prices = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
        let mut returns = Vec::new();
        get_returns(&prices, 2, &mut returns);
        assert!(!returns.is_empty());
    }

    #[test]
    fn test_get_returns_length() {
        let prices = vec![0.0; 20];
        let max_lookback = 3;
        let mut returns = Vec::new();
        get_returns(&prices, max_lookback, &mut returns);

        let expected_len = max_lookback * (max_lookback - 1) / 2 * (prices.len() - max_lookback);
        assert_eq!(returns.len(), expected_len);
    }

    #[test]
    fn test_cscvcore_small() {
        let ncases = 12;
        let n_systems = 4;
        let n_blocks = 4;

        let returns = vec![0.1; n_systems * ncases];

        let mut indices = Vec::new();
        let mut lengths = Vec::new();
        let mut flags = Vec::new();
        let mut work = Vec::new();
        let mut is_crits = Vec::new();
        let mut oos_crits = Vec::new();

        let prob = cscvcore(
            ncases,
            n_systems,
            n_blocks,
            &returns,
            &mut indices,
            &mut lengths,
            &mut flags,
            &mut work,
            &mut is_crits,
            &mut oos_crits,
        );

        assert!(prob >= 0.0 && prob <= 1.0);
    }
}